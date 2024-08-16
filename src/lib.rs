use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*, window::ExitCondition};
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_render::RatatuiRenderPlugin;
use logo::LogoPath;
pub use logo::{LOGO_PATH_DVD, LOGO_PATH_TTY};
use rand::{distributions::Standard, prelude::Distribution, Rng};

mod assets;
mod bubbles;
mod common;
mod logo;
mod maze;

pub struct AppPlugin(pub SaverVariant);

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    close_when_requested: false,
                })
                .disable::<LogPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            RatatuiPlugins::default(),
            RatatuiRenderPlugin::new("main", (256, 256)).autoresize(),
        ))
        .insert_resource(Msaa::Off)
        .init_resource::<Flags>();

        app.add_plugins((assets::plugin, common::plugin));

        if let SaverVariant::Logo(ref logo_path) = self.0 {
            app.insert_resource(LogoPath(logo_path.into()));
        }

        app.add_plugins(match self.0 {
            SaverVariant::Bubbles => bubbles::plugin,
            SaverVariant::Logo(_) => logo::plugin,
            SaverVariant::Maze => maze::plugin,
        });
    }
}

#[derive(Resource, Default)]
pub struct Flags {
    _debug: bool,
    _msgs: Vec<String>,
}

pub enum SaverVariant {
    Bubbles,
    Logo(String),
    Maze,
}

impl Distribution<SaverVariant> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SaverVariant {
        match rng.gen_range(0..=2) {
            0 => SaverVariant::Bubbles,
            1 => SaverVariant::Logo(LOGO_PATH_TTY.into()),
            _ => SaverVariant::Maze,
        }
    }
}
