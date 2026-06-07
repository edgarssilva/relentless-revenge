use bevy::prelude::*;

use game_states::{ingame::InGamePlugin, loading::LoadingPlugin, menu::MainMenuPlugin};
use helper::KeyMaps;
use stats::*;

mod animation;
mod attack;
mod boss;
mod collision;
mod controller;
mod effects;
mod enemy;
mod floor;
mod game_states;
mod helper;
mod layers;
mod manifest;
mod map;
mod movement;
mod player;
mod sorting;
mod state;
mod statistics;
mod stats;
mod ui;

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
        .insert_resource(ClearColor(Color::srgb(20. / 255., 0. / 255., 25. / 255.)))
        .insert_resource(KeyMaps::default())
        .init_state::<GameState>()
        .add_plugins(LoadingPlugin)
        .add_plugins(MainMenuPlugin)
        .add_plugins(InGamePlugin)
        .run();
}
