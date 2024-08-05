use std::{env, fmt::Display};

use bevy::app::App;
use clap::{Parser, Subcommand};
use ttysvr::{AppPlugin, SaverVariant};

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
    Maze,
    Pipes,
    Bubbles,
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::Maze => write!(f, "maze"),
            Variant::Pipes => write!(f, "pipes"),
            Variant::Bubbles => write!(f, "bubbles"),
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
        Some(Variant::Maze) => SaverVariant::Maze,
        Some(Variant::Pipes) => SaverVariant::Pipes,
        Some(Variant::Bubbles) => SaverVariant::Bubbles,
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
