use bevy::prelude::*;

use game_states::{ingame::InGamePlugin, loading::LoadingPlugin, menu::MainMenuPlugin};
use helper::KeyMaps;
use stats::*;

mod animation;
mod attack;
mod boss;
mod collision;
mod controller;
mod enemy;
mod game_states;
mod helper;
mod level;
mod map;
mod metadata;
mod movement;
mod player;
mod state;
mod stats;
mod ui;

pub const PLAYER_Z: f32 = 39.;
pub const MAP_Z: f32 = 36.;
pub const BACKGROUND_Z: f32 = 1.;
pub const DEBUG_Z: f32 = 100.;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    MainMenu,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::rgb(20. / 255., 0. / 255., 25. / 255.)))
        .insert_resource(KeyMaps::default())
        .add_state::<GameState>()
        .add_plugin(LoadingPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(InGamePlugin)
        .run();
}
