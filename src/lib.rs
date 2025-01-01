use std::time::Duration;

use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin};
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use logo::LogoPath;
pub use logo::{LOGO_PATH_DVD, LOGO_PATH_TTY};
use maze::MazePaths;
pub use maze::{
    MAZE_CEILING_PATH_BRICK, MAZE_CEILING_PATH_HEDGE, MAZE_WALL_PATH_BRICK, MAZE_WALL_PATH_HEDGE,
};
use rand::{distributions::Standard, prelude::Distribution, Rng};

mod assets;
mod bubbles;
mod common;
mod logo;
mod maze;

pub struct AppPlugin(pub Settings);

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            RatatuiPlugins::default(),
            RatatuiCameraPlugin,
        ))
        .init_resource::<Flags>();

        app.add_plugins((assets::plugin, common::plugin));

        let Settings {
            ref variant,
            ref background,
        } = self.0;

        app.insert_resource(ClearColor(Color::Srgba(*background)));

        match variant {
            SaverVariant::Logo(ref logo_path) => {
                app.insert_resource(LogoPath(logo_path.into()));
            }
            SaverVariant::Maze(ref maze_wall, ref maze_ceiling) => {
                app.insert_resource(MazePaths(maze_wall.into(), maze_ceiling.into()));
            }
            _ => {}
        }

        app.add_plugins(match variant {
            SaverVariant::Bubbles => bubbles::plugin,
            SaverVariant::Logo(_) => logo::plugin,
            SaverVariant::Maze(_, _) => maze::plugin,
        });
    }
}

#[derive(Resource, Default)]
pub struct Flags {
    _debug: bool,
    _msgs: Vec<String>,
}

pub struct Settings {
    pub variant: SaverVariant,
    pub background: Srgba,
}

pub enum SaverVariant {
    Bubbles,
    Logo(String),
    Maze(String, String),
}

impl Distribution<SaverVariant> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SaverVariant {
        match rng.gen_range(0..=2) {
            0 => SaverVariant::Bubbles,
            1 => SaverVariant::Logo(LOGO_PATH_TTY.into()),
            _ => SaverVariant::Maze(MAZE_WALL_PATH_BRICK.into(), MAZE_CEILING_PATH_BRICK.into()),
        }
    }
}
