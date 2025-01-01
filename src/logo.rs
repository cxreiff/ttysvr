use bevy::prelude::*;
use bevy_ratatui::event::ResizeEvent;
use bevy_ratatui_camera::RatatuiCamera;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub const LOGO_PATH_DVD: &str = "embedded://ttysvr/../assets/dvd_logo.png";
pub const LOGO_PATH_TTY: &str = "embedded://ttysvr/../assets/tty_logo.png";

const STARTING_DIMENSIONS: (u32, u32) = (256, 256);
const ORTHO_SCALING: f32 = 0.5;
const LOGO_RADIUS: f32 = 32.;
const LOGO_SPEED: f32 = 24.;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LogoVisibleRegion>()
        .add_systems(Startup, logo_setup_system)
        .add_systems(Update, (handle_resize_system, logo_movement_system));
}

#[derive(Component, Deref, DerefMut)]
struct Logo {
    #[deref]
    velocity: Vec2,
}

#[derive(Resource, Deref)]
pub struct LogoPath(pub String);

#[derive(Resource, Deref, DerefMut, Default)]
struct LogoVisibleRegion(Vec2);

fn logo_setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut visible_region: ResMut<LogoVisibleRegion>,
    logo_path: Res<LogoPath>,
) {
    commands.spawn((
        RatatuiCamera::autoresize().with_dimensions(STARTING_DIMENSIONS),
        Camera2d,
        OrthographicProjection {
            scale: ORTHO_SCALING,
            ..OrthographicProjection::default_2d()
        },
    ));

    **visible_region = get_visible_region(STARTING_DIMENSIONS);
    let image = asset_server.load(&**logo_path);
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467712);
    let region = Rectangle::from_size(**visible_region * 0.5 - LOGO_RADIUS * 2.);

    commands.spawn((
        Logo {
            velocity: Vec2::new(LOGO_SPEED, -LOGO_SPEED),
        },
        Sprite {
            image,
            color: Color::hsl(0., 1., 0.6),
            custom_size: Some(Vec2::splat(LOGO_RADIUS * 2.)),
            ..default()
        },
        Transform::from_translation(region.sample_interior(&mut rng).extend(0.)),
    ));
}

fn handle_resize_system(
    mut resize_events: EventReader<ResizeEvent>,
    mut visible_region: ResMut<LogoVisibleRegion>,
) {
    for resize in resize_events.read() {
        let (width, height) = (resize.width * 2, resize.height * 4);
        **visible_region = get_visible_region((width as u32, height as u32));
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

        transform.translation += velocity.extend(0.) * time.delta_secs();

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

fn get_visible_region((width, height): (u32, u32)) -> Vec2 {
    let terminal_dimensions = Vec2::new(width as f32, height as f32);
    terminal_dimensions * ORTHO_SCALING
}
