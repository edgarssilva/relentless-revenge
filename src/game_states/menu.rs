use bevy::prelude::{App, Commands, Input, KeyCode, Plugin, Res};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet},
    state::NextState,
};

use crate::GameState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::MainMenu, setup_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(skip_menu)
                    .into(),
            );
    }
}

fn setup_menu() {}

fn skip_menu(keys: Res<Input<KeyCode>>, mut commands: Commands) {
    if keys.any_just_pressed([KeyCode::Space, KeyCode::Backslash]) {
        commands.insert_resource(NextState(GameState::InGame));
    }
}
