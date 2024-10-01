use std::collections::BTreeMap;

use avian2d::math::PI;
use bevy::prelude::*;
use bevy_ratatui_render::RatatuiRenderContext;
use rand::{seq::SliceRandom, thread_rng};

#[derive(PartialEq, Debug)]
enum MazeDirection {
    North,
    East,
    South,
    West,
}

type MazeGraph = BTreeMap<(i32, i32), (bool, bool, bool, bool)>;

const MAZE_SIZE: i32 = 16;
const DIRECTION_LIST: &[MazeDirection] = &[
    MazeDirection::North,
    MazeDirection::East,
    MazeDirection::South,
    MazeDirection::West,
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

        let valid_candidates: Vec<&MazeDirection> = DIRECTION_LIST
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
            MazeDirection::North => {
                maze.entry(current).or_default().0 = true;
                maze.entry(next).or_default().2 = true;
            }
            MazeDirection::East => {
                maze.entry(current).or_default().1 = true;
                maze.entry(next).or_default().3 = true;
            }
            MazeDirection::South => {
                maze.entry(current).or_default().2 = true;
                maze.entry(next).or_default().0 = true;
            }
            MazeDirection::West => {
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
        let translation = Vec3::new(*x as f32, *y as f32, -0.75);

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
    let mut camera_transform = camera.single_mut();
    let target_vec = target_to_vec3(**target);

    let camera_target_dot = camera_transform
        .forward()
        .dot((target_vec - camera_transform.translation).normalize());

    if (camera_target_dot - 1.).abs() < 0.0001 {
        camera_transform.translation = camera_transform.translation.move_towards(target_vec, delta);
    } else {
        let target_dir = target_vec - camera_transform.translation;
        let cross = target_dir.cross(*camera_transform.forward());
        let dot = cross.dot(Vec3::Z);

        if dot > 0. {
            camera_transform.rotate_z(-delta);
        } else {
            camera_transform.rotate_z(delta);
        }
    }

    if camera_transform
        .translation
        .distance(target_to_vec3(**target))
        < 0.01
    {
        let node_edges = maze.get(&target).unwrap();
        let facing_direction = camera_direction(&camera_transform);
        let next_direction = next_valid_direction(&facing_direction, node_edges);
        **target = adjacent_node(**target, &next_direction);
    };
}

fn adjacent_node((x, y): (i32, i32), direction: &MazeDirection) -> (i32, i32) {
    match direction {
        MazeDirection::North => (x, y + 1),
        MazeDirection::East => (x + 1, y),
        MazeDirection::South => (x, y - 1),
        MazeDirection::West => (x - 1, y),
    }
}

fn camera_direction(camera_transform: &Transform) -> MazeDirection {
    let forward_vector = camera_transform.forward();

    if forward_vector.x.abs() > forward_vector.y.abs() {
        if forward_vector.x < 0. {
            MazeDirection::West
        } else {
            MazeDirection::East
        }
    } else if forward_vector.y < 0. {
        MazeDirection::South
    } else {
        MazeDirection::North
    }
}

fn next_valid_direction(
    facing: &MazeDirection,
    (north, east, south, west): &(bool, bool, bool, bool),
) -> MazeDirection {
    let mut direction_order = vec![
        MazeDirection::North,
        MazeDirection::East,
        MazeDirection::South,
        MazeDirection::West,
    ];

    while *facing != direction_order[1] {
        direction_order.rotate_left(1);
    }

    for direction in direction_order {
        if match direction {
            MazeDirection::North => *north,
            MazeDirection::East => *east,
            MazeDirection::South => *south,
            MazeDirection::West => *west,
        } {
            return direction;
        }
    }

    unreachable!();
}

fn target_to_vec3(target: (i32, i32)) -> Vec3 {
    Vec3::new((target).0 as f32, target.1 as f32, 0.)
}
