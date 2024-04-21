use bevy::{
    input::ButtonInput,
    prelude::{App, KeyCode, NextState, OnEnter, Plugin, Res, ResMut},
};

use crate::GameState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), (setup_menu, skip_menu));
    }
}

fn setup_menu() {}

fn skip_menu(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<NextState<GameState>>) {
    if keys.any_just_pressed([KeyCode::Space, KeyCode::Backslash]) {
        state.set(GameState::InGame);
    }
}
