use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*, window::ExitCondition};
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_render::RatatuiRenderPlugin;
use rand::{distributions::Standard, prelude::Distribution, Rng};
use ratatui::crossterm::terminal;

mod bubbles;
mod common;
mod maze;
mod pipes;

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
        ))
        .insert_resource(Msaa::Off)
        .init_resource::<Flags>();

        let (cols, rows) = terminal::size().expect("Failed to retrieve terminal dimensions.");

        app.add_plugins(RatatuiRenderPlugin::new(
            "main",
            (cols as u32, rows as u32 * 2),
        ));

        app.add_plugins(common::plugin);

        app.add_plugins(match self.0 {
            SaverVariant::Maze => maze::plugin,
            SaverVariant::Pipes => pipes::plugin,
            SaverVariant::Bubbles => bubbles::plugin,
        });
    }
}

#[derive(Resource, Default)]
pub struct Flags {
    _debug: bool,
}

pub enum SaverVariant {
    Maze,
    Pipes,
    Bubbles,
}

impl Distribution<SaverVariant> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SaverVariant {
        match rng.gen_range(0..=2) {
            0 => SaverVariant::Maze,
            1 => SaverVariant::Pipes,
            _ => SaverVariant::Bubbles,
        }
    }
}
