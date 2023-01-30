use crate::{
    attack::AttackPhase,
    movement::{
        direction::Direction,
        easing::{EaseFunction, EaseTo},
    },
    player::{Player, PlayerActions},
    state::State,
    stats::{Cooldown, MovementSpeed},
};
use bevy::{
    math::Vec2,
    prelude::{Commands, Component, Entity, Query, Res, Transform, With, Without},
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
            &MovementSpeed,
            &ActionState<PlayerActions>, // Entity,
        ),
        (With<Player>, Without<AttackPhase>), // Can't move while attacking
    >,
    time: Res<Time>,
) {
    if let Ok((mut state, mut direction, mut controlled, transform, mv_speed, action_state)) =
        query.get_single_mut()
    {
        // println!("state: {:?}", state);
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

        let dir = dir.normalize_or_zero() * mv_speed.speed as f32 * time.delta_seconds();

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
            &mut Cooldown,
            Entity,
        ),
        (With<Player>, Without<AttackPhase>), // Can't dash while attacking
    >,
    mut commands: Commands,
) {
    if let Ok((mut state, transform, direction, action_state, mut cooldown, entity)) =
        query.get_single_mut()
    {
        let mut dir = Vec2::ZERO;

        if state.equals(State::ATTACKING) || state.equals(State::DASHING) {
            return;
        }

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

        if action_state.just_pressed(PlayerActions::Dash) && cooldown.is_ready() {
            state.set(State::DASHING);
            cooldown.reset();

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
            if state.equals(State::DASHING) {
                state.set(State::IDLE);
            }
        }
    }
}

pub fn attack_ability(
    mut query: Query<(
        &mut State,
        &ActionState<PlayerActions>,
        &Transform,
        &Direction,
        &mut Cooldown,
        Entity,
    )>,
    mut commands: Commands,
) {
    if let Ok((mut state, action_state, transform, direction, mut cooldown, entity)) =
        query.get_single_mut()
    {
        if state.equals(State::DASHING) || state.equals(State::ATTACKING) {
            return;
        }

        if action_state.just_pressed(PlayerActions::Attack) && cooldown.is_ready() {
            state.set(State::ATTACKING);
            cooldown.reset();

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
