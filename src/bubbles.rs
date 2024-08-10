use avian2d::{
    math::{Scalar, Vector},
    prelude::{Collider, Friction, Gravity, LinearVelocity, LockedAxes, RigidBody},
    PhysicsPlugins,
};
use bevy::prelude::*;
use bevy_ratatui::event::ResizeEvent;
use bevy_ratatui_render::RatatuiRenderContext;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

const ORTHO_SCALING: f32 = 0.5;
const BUBBLE_RATE: f32 = 0.33;
const BUBBLE_MAX_SPEED: f32 = 24.;
const BUBBLE_RADIUS: f32 = 9.;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default().with_length_unit(128.))
        .insert_resource(Gravity(Vector::ZERO))
        .init_resource::<BubbleVisibleRegion>()
        .init_resource::<BubbleAmount>()
        .add_systems(Startup, bubbles_setup_system)
        .add_systems(
            Update,
            (
                bubbles_spawn_system,
                handle_resize_system,
                bubble_movement_system,
                bubble_color_system,
            ),
        );
}

#[derive(Component)]
pub struct Bubble {
    target: Vec2,
    timer: Timer,
}

#[derive(Resource, Deref, DerefMut)]
pub struct BubbleRng(ChaCha8Rng);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct BubbleAmount(u32);

#[derive(Resource, Deref)]
pub struct BubbleSprite(Handle<Image>);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct BubbleVisibleRegion(Vec2);

#[derive(Bundle)]
pub struct BubbleBundle {
    bubble: Bubble,
    sprite: SpriteBundle,
    rigidbody: RigidBody,
    collider: Collider,
    locked_axes: LockedAxes,
    friction: Friction,
}

impl BubbleBundle {
    fn new(rng: &mut BubbleRng, sprite: &BubbleSprite, region: &Rectangle) -> Self {
        Self {
            bubble: Bubble {
                target: region.sample_interior(&mut rng.0),
                timer: Timer::from_seconds(3., TimerMode::Repeating),
            },
            sprite: SpriteBundle {
                transform: Transform::from_translation(
                    region.sample_interior(&mut rng.0).extend(0.),
                ),
                texture: (**sprite).clone(),
                sprite: Sprite {
                    color: Color::hsl(rng.gen_range(0.0..360.0), 1.0, 0.8),
                    custom_size: Some(Vec2::splat(BUBBLE_RADIUS * 2.)),
                    ..default()
                },
                ..default()
            },
            rigidbody: RigidBody::Dynamic,
            collider: Collider::circle(BUBBLE_RADIUS as Scalar),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            friction: Friction::new(0.0),
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct BubbleTimer(Timer);

impl Default for BubbleTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(BUBBLE_RATE, TimerMode::Repeating))
    }
}

fn bubbles_setup_system(
    mut commands: Commands,
    ratatui_render: Res<RatatuiRenderContext>,
    asset_server: Res<AssetServer>,
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = ORTHO_SCALING;
    camera.camera.target = ratatui_render.target("main").unwrap_or_default();
    commands.spawn(camera);

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0., 0., -5.),
        point_light: PointLight {
            intensity: 10_000.,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    let rng = ChaCha8Rng::seed_from_u64(19878367467712);
    commands.insert_resource(BubbleRng(rng));
    commands.insert_resource(BubbleSprite(
        asset_server.load("embedded://ttysvr/../assets/bubble.png"),
    ));
}

#[allow(clippy::too_many_arguments)]
fn bubbles_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut rng: ResMut<BubbleRng>,
    sprite: Res<BubbleSprite>,
    visible_region: Res<BubbleVisibleRegion>,
    spawn_amount: ResMut<BubbleAmount>,
    mut timer: Local<BubbleTimer>,
    mut count: Local<u32>,
) {
    timer.tick(time.delta());
    if timer.finished() && *count < **spawn_amount {
        *count += 1;
        commands.spawn(BubbleBundle::new(
            &mut rng,
            &sprite,
            &Rectangle::from_size(**visible_region - BUBBLE_RADIUS * 2.),
        ));
    }
}

fn handle_resize_system(
    mut resize_events: EventReader<ResizeEvent>,
    mut visible_region: ResMut<BubbleVisibleRegion>,
    mut spawn_amount: ResMut<BubbleAmount>,
    ratatui_render: Res<RatatuiRenderContext>,
) {
    for _ in resize_events.read() {
        let (width, height) = ratatui_render.dimensions("main").unwrap();
        let terminal_dimensions = Vec2::new(width as f32, height as f32);
        **visible_region = terminal_dimensions * ORTHO_SCALING;
        **spawn_amount = ((visible_region.x * visible_region.y) / 777.) as u32;
    }
}

fn bubble_movement_system(
    time: Res<Time>,
    mut bubbles: Query<(&mut Transform, &mut LinearVelocity, &mut Bubble)>,
    visible_region: Res<BubbleVisibleRegion>,
    mut rng: ResMut<BubbleRng>,
) {
    for (mut transform, mut velocity, mut bubble) in &mut bubbles {
        let visible_area = Rectangle::from_size(**visible_region - BUBBLE_RADIUS * 1.95);
        let visible_half = visible_area.half_size.extend(0.);

        let diff = (bubble.target.extend(0.) - transform.translation).xy();

        let next_point = visible_area.sample_interior(&mut rng.0);

        bubble.timer.tick(time.delta());
        if diff.length() < 30.0 || bubble.timer.finished() {
            bubble.target = next_point * 2.;
            bubble.timer.reset();
        }

        bubble.target = bubble
            .target
            .move_towards(next_point, time.delta_seconds() * 10.);

        **velocity += diff * 0.01;
        **velocity = velocity.clamp(
            -Vec2::splat(BUBBLE_MAX_SPEED),
            Vec2::splat(BUBBLE_MAX_SPEED),
        );
        transform.translation = transform.translation.clamp(-visible_half, visible_half);
    }
}

fn bubble_color_system(time: Res<Time>, mut bubbles: Query<&mut Sprite, With<Bubble>>) {
    for mut sprite in &mut bubbles {
        let new_hue = (sprite.color.hue() + time.delta_seconds() * 10.) % 360.;
        sprite.color.set_hue(new_hue);
    }
}
