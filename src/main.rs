use std::env;

use args::{Args, LogoVariant, MazeVariant, Variant};
use bevy::{app::App, color::Srgba};
use clap::Parser;
use ttysvr::{
    AppPlugin, SaverVariant, Settings, LOGO_PATH_DVD, LOGO_PATH_TTY, MAZE_CEILING_PATH_BRICK,
    MAZE_CEILING_PATH_HEDGE, MAZE_WALL_PATH_BRICK, MAZE_WALL_PATH_HEDGE,
};

mod args;

fn main() {
    let Args {
        variant,
        background,
        init,
        cancel,
    } = Args::parse();

    if let Some(delay) = init {
        let executable_string = env::args().next().unwrap_or("ttysvr".into());
        let variant_string = match variant {
            Some(variant) => format!(" {variant}"),
            None => "".into(),
        };
        let background_string = match background {
            Some(background) => format!(" --bg={background}"),
            None => "".into(),
        };

        #[rustfmt::skip]
        println!(
"
TMOUT={delay}; trap \"{executable_string}{variant_string}{background_string}; zle reset-prompt\" ALRM

# WRAP THIS COMMAND IN EVAL WITH BACKTICKS (ZSH ONLY)
# EXAMPLE: eval `ttysvr{variant_string}{background_string} --init {delay}`
"
        );
        return;
    };

    if cancel {
        #[rustfmt::skip]
        println!(
"
TMOUT=0

# WRAP THIS COMMAND IN EVAL WITH BACKTICKS (ZSH ONLY)
# EXAMPLE: eval `ttysvr --cancel`
"
        );
        return;
    }

    let saver_variant = match variant {
        Some(Variant::Bubbles) => SaverVariant::Bubbles,
        Some(Variant::Logo { ref variant }) => match variant {
            Some(LogoVariant::Dvd) | None => SaverVariant::Logo(LOGO_PATH_DVD.into()),
            Some(LogoVariant::Tty) => SaverVariant::Logo(LOGO_PATH_TTY.into()),
        },
        Some(Variant::Maze { ref variant }) => match variant {
            Some(MazeVariant::Brick) | None => {
                SaverVariant::Maze(MAZE_WALL_PATH_BRICK.into(), MAZE_CEILING_PATH_BRICK.into())
            }
            Some(MazeVariant::Hedge) => {
                SaverVariant::Maze(MAZE_WALL_PATH_HEDGE.into(), MAZE_CEILING_PATH_HEDGE.into())
            }
        },
        None => rand::random(),
    };

    let settings = Settings {
        variant: saver_variant,
        background: background.map_or(Srgba::NONE, |bg| bg.0),
    };

    App::new().add_plugins(AppPlugin(settings)).run();
}
