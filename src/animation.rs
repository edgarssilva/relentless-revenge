use crate::{movement::direction::Direction, state::State, GameState};
use bevy_spritesheet_animation::prelude::*;

use bevy::{
    prelude::{in_state, App, Component, IntoSystemConfigs, Plugin, Query, Update},
    utils::HashMap,
};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpritesheetAnimationPlugin);
        app.add_systems(
            Update,
            (animation_state).run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component, PartialEq, Debug, Clone)]
pub struct Animations(pub HashMap<String, AnimationId>);

#[derive(Component, PartialEq, Debug, Clone)] //And statefull
pub struct DirectionalAnimations(pub HashMap<State, HashMap<Direction, AnimationId>>);

pub fn animation_state(
    mut query: Query<(
        &DirectionalAnimations,
        &mut SpritesheetAnimation,
        &State,
        &Direction,
    )>,
) {
    for (animations, mut spritesheet, state, direction) in query.iter_mut() {
        if let Some(state_animations) = animations.0.get(state) {
            if let Some(anim) = state_animations.get(direction) {
                if spritesheet.animation_id != *anim {
                    spritesheet.switch(*anim);
                }
            }
        }
    }
}
