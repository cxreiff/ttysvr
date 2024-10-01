use bevy::asset::embedded_asset;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    let prefix = "src/";
    embedded_asset!(app, prefix, "../assets/bubble.png");
    embedded_asset!(app, prefix, "../assets/dvd_logo.png");
    embedded_asset!(app, prefix, "../assets/tty_logo.png");
    embedded_asset!(app, prefix, "../assets/maze_wall.png");
    embedded_asset!(app, prefix, "../assets/maze_floor.png");
    embedded_asset!(app, prefix, "../assets/maze_ceiling.png");
}
