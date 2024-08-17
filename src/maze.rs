use avian2d::math::PI;
use bevy::prelude::*;
use bevy_ratatui_render::RatatuiRenderContext;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, maze_setup_system)
        .add_systems(Update, rotate_system);
}

fn maze_setup_system(
    mut commands: Commands,
    ratatui_render: Res<RatatuiRenderContext>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                target: ratatui_render.target("main").unwrap_or_default(),
                ..default()
            },
            transform: Transform::from_translation(Vec3::ZERO)
                .looking_at(Vec3::new(0., 0., 1.), Vec3::Y),
            ..default()
        })
        .with_children(|commands| {
            commands.spawn(PointLightBundle {
                point_light: PointLight {
                    intensity: 100_000.,
                    ..default()
                },
                ..default()
            });
        });

    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(0., 0., 5.),
        mesh: meshes.add(Cuboid::from_size(Vec3::new(5., 2., 0.1))),
        material: materials.add(StandardMaterial::from_color(Color::hsl(0., 0.5, 0.5))),
        ..default()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(5., 0., 0.).with_rotation(Quat::from_rotation_y(PI / 2.)),
        mesh: meshes.add(Cuboid::from_size(Vec3::new(5., 2., 0.1))),
        material: materials.add(StandardMaterial::from_color(Color::hsl(0., 0.5, 0.5))),
        ..default()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(-5., 0., 0.).with_rotation(Quat::from_rotation_y(-PI / 2.)),
        mesh: meshes.add(Cuboid::from_size(Vec3::new(5., 2., 0.1))),
        material: materials.add(StandardMaterial::from_color(Color::hsl(0., 0.5, 0.5))),
        ..default()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(0., 0., -5.).with_rotation(Quat::from_rotation_y(PI)),
        mesh: meshes.add(Cuboid::from_size(Vec3::new(5., 2., 0.1))),
        material: materials.add(StandardMaterial::from_color(Color::hsl(0., 0.5, 0.5))),
        ..default()
    });
}

fn rotate_system(time: Res<Time>, mut camera: Query<&mut Transform, With<Camera>>) {
    camera.single_mut().rotate_y(time.delta_seconds());
}
