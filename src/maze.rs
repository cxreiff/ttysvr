use bevy::prelude::*;
use bevy_ratatui_render::RatatuiRenderContext;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, maze_setup_system);
}

fn maze_setup_system(
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
        transform: Transform::from_xyz(0., 5., 0.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.))),
        material: materials.add(StandardMaterial::from_color(Color::hsl(220., 0.8, 0.5))),
        ..default()
    });
}
