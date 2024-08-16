use bevy::prelude::*;
use bevy_ratatui::event::ResizeEvent;
use bevy_ratatui_render::RatatuiRenderContext;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub const LOGO_PATH_DVD: &str = "embedded://ttysvr/../assets/dvd_logo.png";
pub const LOGO_PATH_TTY: &str = "embedded://ttysvr/../assets/tty_logo.png";

const ORTHO_SCALING: f32 = 0.5;
const LOGO_RADIUS: f32 = 32.;
const LOGO_SPEED: f32 = 24.;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LogoVisibleRegion>()
        .add_systems(Startup, logo_setup_system)
        .add_systems(Update, (handle_resize_system, logo_movement_system));
}

#[derive(Component, Deref, DerefMut)]
pub struct Logo {
    #[deref]
    velocity: Vec2,
}

#[derive(Resource, Deref)]
pub struct LogoPath(pub String);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct LogoVisibleRegion(Vec2);

#[derive(Bundle)]
pub struct LogoBundle {
    logo: Logo,
    sprite: SpriteBundle,
}

impl LogoBundle {
    fn new(rng: &mut ChaCha8Rng, texture: Handle<Image>, region: Rectangle) -> Self {
        Self {
            logo: Logo {
                velocity: Vec2::new(LOGO_SPEED, -LOGO_SPEED),
            },
            sprite: SpriteBundle {
                transform: Transform::from_translation(region.sample_interior(rng).extend(0.)),
                texture,
                sprite: Sprite {
                    color: Color::hsl(0., 1.0, 0.6),
                    custom_size: Some(Vec2::splat(LOGO_RADIUS * 2.)),
                    ..default()
                },
                ..default()
            },
        }
    }
}

fn logo_setup_system(
    mut commands: Commands,
    ratatui_render: Res<RatatuiRenderContext>,
    asset_server: Res<AssetServer>,
    mut visible_region: ResMut<LogoVisibleRegion>,
    logo_path: Res<LogoPath>,
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = ORTHO_SCALING;
    camera.camera.target = ratatui_render.target("main").unwrap_or_default();
    commands.spawn(camera);

    **visible_region = get_visible_region(&ratatui_render);
    let texture = asset_server.load(&**logo_path);
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467712);
    commands.spawn(LogoBundle::new(
        &mut rng,
        texture,
        Rectangle::from_size(**visible_region * 0.5 - LOGO_RADIUS * 2.),
    ));
}

fn handle_resize_system(
    mut resize_events: EventReader<ResizeEvent>,
    mut visible_region: ResMut<LogoVisibleRegion>,
    ratatui_render: Res<RatatuiRenderContext>,
) {
    for _ in resize_events.read() {
        **visible_region = get_visible_region(&ratatui_render);
    }
}

fn logo_movement_system(
    time: Res<Time>,
    mut logo: Query<(&mut Transform, &mut Sprite, &mut Logo)>,
    visible_region: Res<LogoVisibleRegion>,
) {
    for (mut transform, mut sprite, mut velocity) in &mut logo {
        let visible_area = Rectangle::from_size(**visible_region - LOGO_RADIUS * 1.95);
        let visible_half = visible_area.half_size.extend(0.);

        transform.translation += velocity.extend(0.) * time.delta_seconds();

        if (transform.translation.x < -visible_half.x && velocity.x < 0.)
            || (transform.translation.x > visible_half.x && velocity.x > 0.)
        {
            velocity.x *= -1.;
            let next_hue = (sprite.color.hue() + 68.) % 360.;
            sprite.color.set_hue(next_hue);
        }

        if (transform.translation.y < -visible_half.y - LOGO_RADIUS * 0.55 && velocity.y < 0.)
            || (transform.translation.y > visible_half.y + LOGO_RADIUS * 0.55 && velocity.y > 0.)
        {
            velocity.y *= -1.;
            let next_hue = (sprite.color.hue() + 68.) % 360.;
            sprite.color.set_hue(next_hue);
        }
    }
}

fn get_visible_region(ratatui_render: &RatatuiRenderContext) -> Vec2 {
    let (width, height) = ratatui_render.dimensions("main").unwrap();
    let terminal_dimensions = Vec2::new(width as f32, height as f32);
    terminal_dimensions * ORTHO_SCALING
}
