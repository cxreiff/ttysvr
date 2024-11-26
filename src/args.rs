use std::fmt::Display;

use bevy::color::Srgba;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub variant: Option<Variant>,

    #[arg(
        short,
        long = "bg",
        global = true,
        name = "HEX COLOR",
        help = "Set screensaver background to provided HEX COLOR, if applicable to variant."
    )]
    pub background: Option<ColorPreference>,

    #[arg(
        short,
        long,
        global = true,
        name = "DELAY",
        help = "Prints command for initiating ttysvr in DELAY seconds."
    )]
    pub init: Option<u32>,

    #[arg(
        short,
        long,
        global = true,
        help = "Prints command for cancelling ttysvr in current shell."
    )]
    pub cancel: bool,
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

#[derive(Clone)]
pub struct ColorPreference(pub Srgba);

impl From<String> for ColorPreference {
    fn from(value: String) -> Self {
        if let Ok(hex_color) = Srgba::hex(value) {
            return ColorPreference(hex_color);
        }

        ColorPreference(Srgba::NONE)
    }
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

impl Display for ColorPreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}
