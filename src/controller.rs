use crate::{
    movement::{
        direction::Direction,
        easing::{EaseFunction, EaseTo},
        movement::Velocity,
    },
    player::{Player, PlayerActions},
    state::State,
    stats::Stats,
};
use bevy::{
    math::Vec2,
    prelude::{Commands, Component, Entity, Query, Transform, With},
};
use bevy::{math::Vec3Swizzles, prelude::RemovedComponents};
use leafwing_input_manager::prelude::ActionState;

#[derive(Component)]
pub struct Controlled {
    pub move_to: Option<Vec2>,
}

pub fn move_player(
    mut query: Query<
        (
            &mut State,
            &mut Direction,
            &Stats,
            &ActionState<PlayerActions>,
            Entity,
        ),
        With<Player>,
    >,
    mut commands: Commands,
) {
    let (mut state, mut direction, stats, action_state, entity) = query.single_mut();

    if !(state.equals(State::IDLE) || state.equals(State::WALKING)) {
        return;
    }

    let mut dir = Vec2::ZERO;

    for action in PlayerActions::DIRECTIONS {
        if action_state.pressed(action) {
            if let Some(action_dir) = action.direction() {
                dir += action_dir.vec();
                direction.set(action_dir);
            }
        }
    }

    let dir = dir.normalize_or_zero() * stats.speed as f32;

    if dir.x == 0. && dir.y == 0. {
        state.set(State::IDLE);
        commands.entity(entity).remove::<Velocity>();
    } else {
        commands.entity(entity).insert(Velocity(dir));
        state.set(State::WALKING);
    }
}

pub fn dash_ability(
    mut query: Query<
        (
            &mut State,
            &mut Transform,
            &Direction,
            &ActionState<PlayerActions>,
            Entity,
        ),
        With<Player>,
    >,
    mut commands: Commands,
) {
    let (mut state, transform, direction, action_state, entity) = query.single_mut();

    let mut dir = Vec2::ZERO;

    for action in PlayerActions::DIRECTIONS {
        if action_state.pressed(action) {
            if let Some(action_dir) = action.direction() {
                dir += action_dir.vec();
            }
        }
    }

    if dir == Vec2::ZERO {
        dir = direction.vec();
    }

    if action_state.just_pressed(PlayerActions::Dash) {
        state.set(State::DASHING);

        //TODO: Add dash stats
        let new_pos = transform.translation.xy() + (dir.normalize() * 50.);
        commands
            .entity(entity)
            .insert(EaseTo::new(new_pos, EaseFunction::EaseOutExpo, 1.));
    }
}

pub fn finish_dash(
    mut query: Query<&mut State, With<Player>>,
    removals: RemovedComponents<EaseTo>,
) {
    for entity in removals.iter() {
        if let Ok(mut state) = query.get_mut(entity) {
            state.set(State::IDLE);
        }
    }
}
