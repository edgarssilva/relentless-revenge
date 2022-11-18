mod animation;
mod attack;
mod collision;
mod controller;
mod enemy;
mod helper;
mod level;
mod map;
mod menus;
mod movement;
mod player;
mod state;
mod stats;

use bevy::prelude::*;
use helper::KeyMaps;
use iyes_loopless::prelude::*;

use menus::{ingame::InGamePlugin, main_menu::MainMenuPlugin};
use stats::*;

pub const PLAYER_Z: f32 = 39.;
pub const MAP_Z: f32 = 36.;
pub const BACKGROUND_Z: f32 = 1.;
pub const DEBUG_Z: f32 = 100.;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    MainMenu,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_loopless_state(GameState::MainMenu)
        .add_plugin(MainMenuPlugin)
        .add_plugin(InGamePlugin)
        .insert_resource(ClearColor(Color::rgb(20. / 255., 0. / 255., 25. / 255.)))
        .insert_resource(KeyMaps::default())
        .run();
}
