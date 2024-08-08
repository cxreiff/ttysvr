use avian2d::{
    math::{Scalar, Vector},
    prelude::{Collider, Gravity, RigidBody},
    PhysicsPlugins,
};
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ratatui::event::ResizeEvent;
use bevy_ratatui_render::RatatuiRenderContext;
use rand::{seq::IteratorRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

const ORTHO_SCALING: f32 = 32.;
const BUBBLE_RATE: f32 = 1.;
const BUBBLE_MAX: u32 = 16;
const BUBBLE_RADIUS: f32 = 0.75;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vector::ZERO))
        .init_resource::<BubbleVisibleRegion>()
        .add_systems(Startup, bubbles_setup_system)
        .add_systems(Update, (bubbles_spawn_system, handle_resize_system));
}

#[derive(Component)]
pub struct Bubble;

#[derive(Resource, Deref, DerefMut)]
pub struct BubbleRng(ChaCha8Rng);

#[derive(Resource, Deref)]
pub struct BubbleMaterials(Vec<Handle<StandardMaterial>>);

#[derive(Resource, Deref)]
pub struct BubbleMesh(Handle<Mesh>);

#[derive(Resource, Deref, Default)]
pub struct BubbleVisibleRegion(Vec2);

#[derive(Bundle)]
pub struct BubbleBundle {
    bubble: Bubble,
    pbr: PbrBundle,
    rigidbody: RigidBody,
    collider: Collider,
}

impl BubbleBundle {
    fn new(
        rng: &mut BubbleRng,
        materials: &BubbleMaterials,
        mesh: &BubbleMesh,
        region: &Rectangle,
    ) -> Self {
        Self {
            bubble: Bubble,
            pbr: PbrBundle {
                transform: Transform::from_translation(
                    region.sample_interior(&mut rng.0).extend(0.),
                ),
                material: materials
                    .iter()
                    .choose(&mut rng.0)
                    .expect("bubble materials were not generated.")
                    .clone(),
                mesh: mesh.0.clone(),
                ..default()
            },
            rigidbody: RigidBody::Dynamic,
            collider: Collider::circle(BUBBLE_RADIUS as Scalar),
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

#[derive(Component)]
pub struct BubbleTopWall;

#[derive(Component)]
pub struct BubbleBottomWall;

#[derive(Component)]
pub struct BubbleLeftWall;

#[derive(Component)]
pub struct BubbleRightWall;

fn bubbles_setup_system(
    mut commands: Commands,
    ratatui_render: Res<RatatuiRenderContext>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        camera: Camera {
            target: ratatui_render.target("main").unwrap_or_default(),
            ..default()
        },
        projection: Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize(ORTHO_SCALING),
            ..default()
        }),
        transform: Transform::from_xyz(0., 0., 5.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0., 0., -5.),
        point_light: PointLight {
            intensity: 10_000.,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    let mut rng = ChaCha8Rng::seed_from_u64(19878367467712);
    commands.insert_resource(BubbleMaterials(
        (0..8)
            .map(|_| {
                materials.add(StandardMaterial {
                    base_color: Color::hsl(rng.gen_range(0.0..360.0), 1.0, 0.8),
                    specular_transmission: 0.9,
                    diffuse_transmission: 1.0,
                    thickness: 1.8,
                    ior: 1.5,
                    perceptual_roughness: 0.12,
                    ..default()
                })
            })
            .collect(),
    ));
    commands.insert_resource(BubbleRng(rng));
    commands.insert_resource(BubbleMesh(meshes.add(Sphere::new(BUBBLE_RADIUS))));

    let wall_bundle = (
        PbrBundle {
            mesh: meshes.add(Cuboid::from_size(Vec3::new(1., 1., 1.))),
            material: materials.add(StandardMaterial::default()),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(0.95, 0.95),
    );

    commands.spawn((wall_bundle.clone(), BubbleBottomWall));
    commands.spawn((wall_bundle.clone(), BubbleTopWall));
    commands.spawn((wall_bundle.clone(), BubbleLeftWall));
    commands.spawn((wall_bundle.clone(), BubbleRightWall));
}

#[allow(clippy::too_many_arguments)]
fn bubbles_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut rng: ResMut<BubbleRng>,
    materials: Res<BubbleMaterials>,
    mesh: Res<BubbleMesh>,
    visible_region: Res<BubbleVisibleRegion>,
    mut timer: Local<BubbleTimer>,
    mut count: Local<u32>,
) {
    timer.tick(time.delta());
    if timer.finished() && *count < BUBBLE_MAX {
        *count += 1;
        commands.spawn(BubbleBundle::new(
            &mut rng,
            &materials,
            &mesh,
            &Rectangle::from_size(**visible_region - BUBBLE_RADIUS * 2.),
        ));
    }
}

type BottomWallQuery = (
    With<BubbleBottomWall>,
    Without<BubbleTopWall>,
    Without<BubbleLeftWall>,
    Without<BubbleRightWall>,
);
type TopWallQuery = (
    With<BubbleTopWall>,
    Without<BubbleBottomWall>,
    Without<BubbleLeftWall>,
    Without<BubbleRightWall>,
);
type LeftWallQuery = (
    With<BubbleLeftWall>,
    Without<BubbleBottomWall>,
    Without<BubbleTopWall>,
    Without<BubbleRightWall>,
);
type RightWallQuery = (
    With<BubbleRightWall>,
    Without<BubbleBottomWall>,
    Without<BubbleTopWall>,
    Without<BubbleLeftWall>,
);
fn handle_resize_system(
    mut resize_events: EventReader<ResizeEvent>,
    mut visible_region: ResMut<BubbleVisibleRegion>,
    ratatui_render: Res<RatatuiRenderContext>,
    mut bottom_wall: Query<&mut Transform, BottomWallQuery>,
    mut top_wall: Query<&mut Transform, TopWallQuery>,
    mut left_wall: Query<&mut Transform, LeftWallQuery>,
    mut right_wall: Query<&mut Transform, RightWallQuery>,
) {
    for _ in resize_events.read() {
        let (width, height) = ratatui_render.dimensions("main").unwrap();
        let terminal_dimensions = Vec2::new(width as f32, height as f32);
        visible_region.0 = terminal_dimensions / ORTHO_SCALING;

        bottom_wall.single_mut().translation = Vec3::new(0.0, -visible_region.y / 2., 0.0);
        bottom_wall.single_mut().scale = Vec3::new(visible_region.x, 0.5, 1.0);

        top_wall.single_mut().translation = Vec3::new(0.0, visible_region.y / 2., 0.0);
        top_wall.single_mut().scale = Vec3::new(visible_region.x, 0.5, 1.0);

        left_wall.single_mut().translation = Vec3::new(-visible_region.x / 2., 0.0, 0.0);
        left_wall.single_mut().scale = Vec3::new(0.5, visible_region.y, 1.0);

        right_wall.single_mut().translation = Vec3::new(visible_region.x / 2., 0.0, 0.0);
        right_wall.single_mut().scale = Vec3::new(0.5, visible_region.y, 1.0);
    }
}
