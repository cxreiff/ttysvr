use std::{env, fmt::Display};

use bevy::app::App;
use clap::{Parser, Subcommand};
use ttysvr::{AppPlugin, SaverVariant};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    variant: Variant,

    #[arg(short, long, global = true, name = "DELAY")]
    init: Option<u32>,
}

#[derive(Subcommand)]
pub enum Variant {
    Maze,
    Pipes,
    Shuffle,
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::Maze => write!(f, "maze"),
            Variant::Pipes => write!(f, "pipes"),
            Variant::Shuffle => write!(f, "shuffle"),
        }
    }
}

fn main() {
    let Cli { variant, init } = Cli::parse();

    let saver_variant = match variant {
        Variant::Maze => SaverVariant::Maze,
        Variant::Pipes => SaverVariant::Pipes,
        Variant::Shuffle => rand::random(),
    };

    if let Some(delay) = init {
        let executable = env::args().next().unwrap_or("ttysvr".into());

        #[rustfmt::skip]
        println!(
"
TMOUT={delay}; trap \"{executable} {variant}; zle reset-prompt\" ALRM

# WRAP THIS COMMAND IN EVAL WITH BACKTICKS (ZSH ONLY)
# EXAMPLE: eval `ttysvr {variant} --init {delay}`
"
        );
        return;
    };

    App::new().add_plugins(AppPlugin(saver_variant)).run();
}
