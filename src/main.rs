use std::fmt::Display;

use bevy::app::App;
use clap::{Parser, Subcommand};
use ttysvr::{AppPlugin, SaverVariant};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand)]
pub enum Subcommands {
    Run {
        #[command(subcommand)]
        variant: Variant,
    },
    Init,
}

#[derive(Clone, Subcommand)]
pub enum Variant {
    Maze,
    Pipes,
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::Maze { .. } => write!(f, "maze"),
            Variant::Pipes { .. } => write!(f, "pipes"),
        }
    }
}

fn main() {
    let Cli { subcommand } = Cli::parse();

    match subcommand {
        Subcommands::Run { variant } => {
            let saver_variant = match variant {
                Variant::Maze => SaverVariant::Maze,
                Variant::Pipes => SaverVariant::Pipes,
            };

            App::new().add_plugins(AppPlugin(saver_variant)).run();
        }
        Subcommands::Init => {
            #[rustfmt::skip]
            println!(
"
# ttysvr
#
# Append this command's output to .zshrc
# e.g. `ttysaver init >> ~/.zshrc && source ~/.zshrc`
#
# call with `svr [variant] [seconds]`
# e.g. `svr maze 1000` for maze screensaver after 1000 seconds.
#
svr() {{ TMOUT=$2; trap \"cargo run -- run $1; zle reset-prompt\" ALRM }}
svr_off() {{ TMOUT=0 }}
# ttysvr end
"
            );
        }
    }
}
