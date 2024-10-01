use std::collections::BTreeMap;

use avian2d::math::PI;
use bevy::prelude::*;
use bevy_ratatui_render::RatatuiRenderContext;
use rand::{seq::SliceRandom, thread_rng};

enum EdgeDirection {
    North,
    East,
    South,
    West,
}

type MazeGraph = BTreeMap<(i32, i32), (bool, bool, bool, bool)>;

const MAZE_SIZE: i32 = 16;
const DIRECTION_LIST: &[EdgeDirection] = &[
    EdgeDirection::North,
    EdgeDirection::East,
    EdgeDirection::South,
    EdgeDirection::West,
];

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, (maze_generation_system, maze_setup_system).chain())
        .add_systems(Update, movement_system);
}

#[derive(Resource, Deref)]
struct Maze(MazeGraph);

#[derive(Resource, Deref, DerefMut)]
struct MazeTarget((i32, i32));

fn maze_generation_system(mut commands: Commands) {
    let mut maze: MazeGraph = BTreeMap::new();
    let mut unresolved = vec![(0, 0)];
    let mut rng = thread_rng();

    loop {
        let Some(current) = unresolved.pop() else {
            break;
        };

        let valid_candidates: Vec<&EdgeDirection> = DIRECTION_LIST
            .iter()
            .filter(|direction| {
                let candidate = adjacent_node(current, direction);
                (0..MAZE_SIZE).contains(&candidate.0)
                    && (0..MAZE_SIZE).contains(&candidate.1)
                    && !maze.contains_key(&candidate)
            })
            .collect();

        let Some(next_direction) = valid_candidates.choose(&mut rng) else {
            continue;
        };

        let next = adjacent_node(current, next_direction);

        match next_direction {
            EdgeDirection::North => {
                maze.entry(current).or_default().0 = true;
                maze.entry(next).or_default().2 = true;
            }
            EdgeDirection::East => {
                maze.entry(current).or_default().1 = true;
                maze.entry(next).or_default().3 = true;
            }
            EdgeDirection::South => {
                maze.entry(current).or_default().2 = true;
                maze.entry(next).or_default().0 = true;
            }
            EdgeDirection::West => {
                maze.entry(current).or_default().3 = true;
                maze.entry(next).or_default().1 = true;
            }
        }

        unresolved.push(current);
        unresolved.push(next);
    }

    commands.insert_resource(Maze(maze));
    commands.insert_resource(MazeTarget((0, 0)));
}

fn maze_setup_system(
    mut commands: Commands,
    ratatui_render: Res<RatatuiRenderContext>,
    maze: Res<Maze>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::from_size(Vec3::new(1.0, 0.1, 1.0)));
    let material = materials.add(StandardMaterial::from_color(Color::hsl(0., 0.5, 0.5)));

    for ((x, y), (north, east, south, west)) in maze.iter() {
        let translation = Vec3::new(*x as f32, *y as f32, 0.);

        if !*north {
            let transform = Transform::default().with_translation(translation + Vec3::Y / 2.);
            commands.spawn(PbrBundle {
                transform,
                mesh: mesh.clone(),
                material: material.clone(),
                ..default()
            });
        }

        if !*east {
            let transform = Transform::default()
                .with_translation(translation + Vec3::X / 2.)
                .with_rotation(Quat::from_rotation_z(PI / 2.));
            commands.spawn(PbrBundle {
                transform,
                mesh: mesh.clone(),
                material: material.clone(),
                ..default()
            });
        }

        if !*south {
            let transform = Transform::default()
                .with_translation(translation - Vec3::Y / 2.)
                .with_rotation(Quat::from_rotation_z(PI));
            commands.spawn(PbrBundle {
                transform,
                mesh: mesh.clone(),
                material: material.clone(),
                ..default()
            });
        }

        if !*west {
            let transform = Transform::default()
                .with_translation(translation - Vec3::X / 2.)
                .with_rotation(Quat::from_rotation_z(PI * 3. / 2.));
            commands.spawn(PbrBundle {
                transform,
                mesh: mesh.clone(),
                material: material.clone(),
                ..default()
            });
        }
    }

    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                target: ratatui_render.target("main").unwrap_or_default(),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., 0.))
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Z),
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
}

fn movement_system(
    time: Res<Time>,
    maze: Res<Maze>,
    mut target: ResMut<MazeTarget>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let delta = time.delta_seconds();
    let mut camera = camera.single_mut();
    let target_vec = Vec3::new((**target).0 as f32, target.1 as f32, 0.);

    if camera.translation.distance(target_vec) < 0.01 {
        let (north, east, south, west) = maze.get(&target).unwrap();

        let _view_direction = rotation_to_edge_direction(camera.rotation);

        if *north {
            **target = adjacent_node(**target, &EdgeDirection::North)
        } else if *east {
            **target = adjacent_node(**target, &EdgeDirection::East)
        } else if *south {
            **target = adjacent_node(**target, &EdgeDirection::South)
        } else if *west {
            **target = adjacent_node(**target, &EdgeDirection::West)
        }
    };

    camera.translation = camera.translation.move_towards(target_vec, delta);
}

fn adjacent_node((x, y): (i32, i32), direction: &EdgeDirection) -> (i32, i32) {
    match direction {
        EdgeDirection::North => (x, y + 1),
        EdgeDirection::East => (x + 1, y),
        EdgeDirection::South => (x, y - 1),
        EdgeDirection::West => (x - 1, y),
    }
}

fn rotation_to_edge_direction(rotation: Quat) -> EdgeDirection {
    if (0.0..(PI / 2.)).contains(&rotation.z) {
        EdgeDirection::North
    } else if ((PI / 2.)..PI).contains(&rotation.z) {
        EdgeDirection::East
    } else if (PI..(PI * 3. / 2.)).contains(&rotation.z) {
        EdgeDirection::South
    } else {
        EdgeDirection::West
    }
}
