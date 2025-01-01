use std::{collections::BTreeMap, f32::consts::PI};

use bevy::prelude::*;
use bevy_ratatui_camera::RatatuiCamera;
use rand::{seq::SliceRandom, thread_rng};

pub const MAZE_WALL_PATH_BRICK: &str = "embedded://ttysvr/../assets/maze_wall_brick.png";
pub const MAZE_WALL_PATH_HEDGE: &str = "embedded://ttysvr/../assets/maze_wall_hedge.png";
pub const MAZE_CEILING_PATH_BRICK: &str = "embedded://ttysvr/../assets/maze_ceiling_brick.png";
pub const MAZE_CEILING_PATH_HEDGE: &str = "embedded://ttysvr/../assets/maze_ceiling_hedge.png";

#[derive(PartialEq, Debug)]
enum MazeDirection {
    North,
    East,
    South,
    West,
}

type MazeGraph = BTreeMap<(i32, i32), (bool, bool, bool, bool)>;

const MAZE_SIZE: i32 = 12;
const MAZE_SCALE: f32 = 1.0;
const MAZE_WALK_SPEED: f32 = 0.4;
const MAZE_TURN_SPEED: f32 = 2.0;
const WALL_DIMENSIONS: Vec3 = Vec3::new(1.0, 0.01, 1.0);
const DIRECTION_LIST: &[MazeDirection] = &[
    MazeDirection::North,
    MazeDirection::East,
    MazeDirection::South,
    MazeDirection::West,
];

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Startup,
        (
            (maze_generation_system, maze_setup_system).chain(),
            lighting_setup_system,
        ),
    )
    .add_systems(Update, movement_system);
}

#[derive(Resource)]
pub struct MazePaths(pub String, pub String);

#[derive(Resource, Deref)]
struct Maze(MazeGraph);

#[derive(Resource, Deref, DerefMut)]
struct MazeTarget((i32, i32));

fn lighting_setup_system(mut ambient: ResMut<AmbientLight>) {
    ambient.brightness = 2000.0;
}

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
    maze: Res<Maze>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    paths: Res<MazePaths>,
) {
    let MazePaths(ref wall_path, ref ceiling_path) = *paths;

    let wall_mesh = meshes.add(Cuboid::from_size(Vec3::new(
        WALL_DIMENSIONS.x * MAZE_SCALE,
        WALL_DIMENSIONS.y * MAZE_SCALE,
        WALL_DIMENSIONS.z * MAZE_SCALE,
    )));

    let floor_ceiling_mesh = meshes.add(Cuboid::from_size(Vec3::new(MAZE_SCALE, MAZE_SCALE, 0.01)));

    let wall_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(wall_path)),
        reflectance: 0.0,
        ..default()
    });

    let floor_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("embedded://ttysvr/../assets/maze_floor.png")),
        reflectance: 0.0,
        ..default()
    });

    let ceiling_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(ceiling_path)),
        reflectance: 0.0,
        ..default()
    });

    for ((x, y), (north, east, south, west)) in maze.iter() {
        let translation = target_to_vec3((*x, *y));

        if !*north {
            commands.spawn((
                Transform::default().with_translation(translation + Vec3::Y * MAZE_SCALE * 0.5),
                Mesh3d(wall_mesh.clone()),
                MeshMaterial3d(wall_material.clone()),
            ));
        }

        if !*east {
            commands.spawn((
                Transform::default()
                    .with_translation(translation + Vec3::X * MAZE_SCALE * 0.5)
                    .with_rotation(Quat::from_rotation_z(PI / 2.)),
                Mesh3d(wall_mesh.clone()),
                MeshMaterial3d(wall_material.clone()),
            ));
        }

        if !*south && *y == 0 {
            commands.spawn((
                Transform::default()
                    .with_translation(translation - Vec3::Y * MAZE_SCALE * 0.5)
                    .with_rotation(Quat::from_rotation_z(PI)),
                Mesh3d(wall_mesh.clone()),
                MeshMaterial3d(wall_material.clone()),
            ));
        }

        if !*west && *x == 0 {
            commands.spawn((
                Transform::default()
                    .with_translation(translation - Vec3::X * MAZE_SCALE * 0.5)
                    .with_rotation(Quat::from_rotation_z(PI * 3. / 2.)),
                Mesh3d(wall_mesh.clone()),
                MeshMaterial3d(wall_material.clone()),
            ));
        }

        commands.spawn((
            Transform::default()
                .with_translation(translation - Vec3::Z * 0.5 * MAZE_SCALE * WALL_DIMENSIONS.z),
            Mesh3d(floor_ceiling_mesh.clone()),
            MeshMaterial3d(floor_material.clone()),
        ));

        commands.spawn((
            Transform::default()
                .with_translation(translation + Vec3::Z * 0.5 * MAZE_SCALE * WALL_DIMENSIONS.z),
            Mesh3d(floor_ceiling_mesh.clone()),
            MeshMaterial3d(ceiling_material.clone()),
        ));
    }

    commands
        .spawn((
            Msaa::Sample8,
            RatatuiCamera::autoresize().with_autoresize_fn(|(w, h)| (w * 4, h * 4)),
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: PI / 2.,
                ..default()
            }),
            Transform::default().looking_at(Vec3::Y, Vec3::Z),
        ))
        .with_children(|commands| {
            commands.spawn(PointLight {
                intensity: 10_000.,
                ..default()
            });
        });
}

fn movement_system(
    time: Res<Time>,
    maze: Res<Maze>,
    mut target: ResMut<MazeTarget>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let delta = time.delta_secs();
    let mut camera_transform = camera.single_mut();
    let target_vec = target_to_vec3(**target);

    let camera_target_dot = camera_transform
        .forward()
        .dot((target_vec - camera_transform.translation).normalize());

    if (camera_target_dot - 1.).abs() < 0.0001 {
        camera_transform.look_at(target_vec, Vec3::Z);
        camera_transform.translation = camera_transform
            .translation
            .move_towards(target_vec, delta * MAZE_SCALE * MAZE_WALK_SPEED);
    } else if camera_target_cross_dot(target_vec, &camera_transform) > 0. {
        camera_transform.rotate_z(-delta * MAZE_TURN_SPEED);
        if camera_target_cross_dot(target_vec, &camera_transform) < 0. {
            camera_transform.look_at(target_vec, Vec3::Z);
        }
    } else {
        camera_transform.rotate_z(delta * MAZE_TURN_SPEED);
        if camera_target_cross_dot(target_vec, &camera_transform) > 0. {
            camera_transform.look_at(target_vec, Vec3::Z);
        }
    }

    if camera_transform.translation.distance(target_vec) < 0.01 {
        camera_transform.translation = target_vec;
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
    Vec3::new(
        (target).0 as f32 * MAZE_SCALE,
        target.1 as f32 * MAZE_SCALE,
        0.,
    )
}

fn camera_target_cross_dot(target_vec: Vec3, camera_transform: &Transform) -> f32 {
    let target_dir = target_vec - camera_transform.translation;
    let cross = target_dir.cross(*camera_transform.forward());
    cross.dot(Vec3::Z)
}
