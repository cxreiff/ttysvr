use std::{env, fmt::Display};

use bevy::app::App;
use bevy::color::Srgba;
use clap::{Parser, Subcommand};
use ttysvr::{
    AppPlugin, CommonSettings, SaverVariant, Settings, LOGO_PATH_DVD, LOGO_PATH_TTY,
    MAZE_CEILING_PATH_BRICK, MAZE_CEILING_PATH_HEDGE, MAZE_WALL_PATH_BRICK, MAZE_WALL_PATH_HEDGE,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    variant: Option<Variant>,

    #[arg(long, help = "Changes the clear color to the given hex coded color.")]
    background_color: Option<String>,

    #[arg(
        short,
        long,
        global = true,
        name = "DELAY",
        help = "Prints command for initiating ttysvr in DELAY seconds."
    )]
    init: Option<u32>,

    #[arg(
        short,
        long,
        global = true,
        help = "Prints command for cancelling ttysvr in current shell."
    )]
    cancel: bool,
}

#[derive(Subcommand)]
pub enum Variant {
    Bubbles,
    Logo {
        #[command(subcommand)]
        variant: Option<LogoVariant>,
    },
    Maze {
        #[command(subcommand)]
        variant: Option<MazeVariant>,
    },
}

#[derive(Subcommand)]
pub enum LogoVariant {
    Dvd,
    Tty,
}

#[derive(Subcommand)]
pub enum MazeVariant {
    Brick,
    Hedge,
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::Bubbles => write!(f, "bubbles"),
            Variant::Logo { variant } => {
                if let Some(variant) = variant {
                    write!(f, "logo {variant}")
                } else {
                    write!(f, "logo")
                }
            }
            Variant::Maze { variant } => {
                if let Some(variant) = variant {
                    write!(f, "maze {variant}")
                } else {
                    write!(f, "maze")
                }
            }
        }
    }
}

impl Display for LogoVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogoVariant::Dvd => write!(f, "dvd"),
            LogoVariant::Tty => write!(f, "tty"),
        }
    }
}

impl Display for MazeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MazeVariant::Brick => write!(f, "brick"),
            MazeVariant::Hedge => write!(f, "hedge"),
        }
    }
}

fn main() {
    let Cli {
        variant,
        init,
        cancel,
        background_color,
    } = Cli::parse();

    let mut settings = Settings {
        saver: match variant {
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
        },
        common: {
            let mut common_settings = CommonSettings::default();
            if let Some(color) = background_color.as_deref().and_then(|c| Srgba::hex(c).ok()) {
                common_settings.clear_color = color.into()
            } else if background_color.is_some() {
                println!("Invalid hex color")
            }

            common_settings
        },
    };

    if let Some(delay) = init {
        let executable_string = env::args().next().unwrap_or("ttysvr".into());
        let variant_string = match variant {
            None => "".into(),
            Some(variant) => format!(" {variant}"),
        };

        #[rustfmt::skip]
        println!(
"
TMOUT={delay}; trap \"{executable_string}{variant_string}; zle reset-prompt\" ALRM

# WRAP THIS COMMAND IN EVAL WITH BACKTICKS (ZSH ONLY)
# EXAMPLE: eval `ttysvr{variant_string} --init {delay}`
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

    App::new().add_plugins(AppPlugin(settings)).run();
}
