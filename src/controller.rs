use crate::{
    attack::AttackPhase,
    movement::{
        direction::Direction,
        easing::{EaseFunction, EaseTo},
    },
    player::{Player, PlayerActions},
    state::State,
    stats::Stats,
};
use bevy::{
    math::Vec2,
    prelude::{Commands, Component, Entity, Query, Res, Transform, With},
    time::{Time, Timer},
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
            &mut Controlled,
            &Transform,
            &Stats,
            &ActionState<PlayerActions>, // Entity,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    if let Ok((mut state, mut direction, mut controlled, transform, stats, action_state)) =
        query.get_single_mut()
    {
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

        let dir = dir.normalize_or_zero() * stats.speed as f32 * time.delta_seconds();

        if dir.x == 0. && dir.y == 0. {
            state.set(State::IDLE);
            controlled.move_to = None;
        } else {
            state.set(State::WALKING);
            let move_to = transform.translation.xy() + dir;
            controlled.move_to = Some(move_to);
        }
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
    if let Ok((mut state, transform, direction, action_state, entity)) = query.get_single_mut() {
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
                .insert(EaseTo::new(new_pos, EaseFunction::EaseOutQuad, 0.5));
        }
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

pub fn attack_ability(
    mut query: Query<(
        &mut State,
        &mut Stats,
        &ActionState<PlayerActions>,
        &Transform,
        &Direction,
        Entity,
    )>,
    mut commands: Commands,
) {
    if let Ok((mut state, mut stats, action_state, transform, direction, entity)) =
        query.get_single_mut()
    {
        if state.equals(State::DASHING) {
            return;
        }

        if action_state.just_pressed(PlayerActions::Attack) && stats.can_attack() {
            state.set(State::ATTACKING);
            stats.reset_attack_timer();

            //TODO: Add attack dash stats
            let new_pos = transform.translation.xy() + (direction.vec().normalize() * 5.);

            commands
                .entity(entity)
                .insert(AttackPhase {
                    charge: Timer::from_seconds(0.05, bevy::time::TimerMode::Once),
                    attack: Timer::from_seconds(0.25, bevy::time::TimerMode::Once),
                    recover: Timer::from_seconds(0.25, bevy::time::TimerMode::Once),
                })
                .insert(EaseTo::new(new_pos, EaseFunction::EaseOutExpo, 0.55));
        }
    }
}
