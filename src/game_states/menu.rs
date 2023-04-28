use bevy::prelude::{
    App, Input, IntoSystemAppConfig, IntoSystemConfig, KeyCode, NextState, OnEnter, Plugin, Res, ResMut,
};

use bevy::ecs::schedule::OnUpdate;

use crate::GameState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_menu.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(skip_menu.in_set(OnUpdate(GameState::MainMenu)));
    }
}

fn setup_menu() {}

fn skip_menu(keys: Res<Input<KeyCode>>, mut state: ResMut<NextState<GameState>>) {
    if keys.any_just_pressed([KeyCode::Space, KeyCode::Backslash]) {
        state.set(GameState::InGame);
    }
}
