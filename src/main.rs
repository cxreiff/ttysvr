use std::{env, fmt::Display};

use bevy::app::App;
use clap::{Parser, Subcommand};
use ttysvr::{AppPlugin, SaverVariant, LOGO_PATH_DVD, LOGO_PATH_TTY};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    variant: Option<Variant>,

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
    Maze,
}

#[derive(Subcommand)]
pub enum LogoVariant {
    Dvd,
    Tty,
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::Bubbles => write!(f, "bubbles"),
            Variant::Logo { variant } => {
                if let Some(variant) = variant {
                    write!(f, "logo {}", variant)
                } else {
                    write!(f, "logo")
                }
            }
            Variant::Maze => write!(f, "maze"),
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

fn main() {
    let Cli {
        variant,
        init,
        cancel,
    } = Cli::parse();

    let saver_variant = match variant {
        Some(Variant::Bubbles) => SaverVariant::Bubbles,
        Some(Variant::Logo { ref variant }) => match variant {
            Some(LogoVariant::Dvd) => SaverVariant::Logo(LOGO_PATH_DVD.into()),
            Some(LogoVariant::Tty) => SaverVariant::Logo(LOGO_PATH_TTY.into()),
            None => SaverVariant::Logo(LOGO_PATH_TTY.into()),
        },
        Some(Variant::Maze) => SaverVariant::Maze,
        None => rand::random(),
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

    App::new().add_plugins(AppPlugin(saver_variant)).run();
}
