use bevy::{
    math::Vec2,
    prelude::{Commands, Component, Entity, Query, Res, Transform, With, Without},
    time::{Time, Timer},
};
use bevy::{math::Vec3Swizzles, prelude::RemovedComponents};
use leafwing_input_manager::prelude::ActionState;

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

#[derive(Component)]
pub struct Controlled {
    pub move_to: Option<Vec2>,
}

#[derive(Component)]
pub struct Combo {
    pub current: u32,
    pub max: u32,
    pub timer: Timer,
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
        if !(state.equals(State::Idle) || state.equals(State::Walking)) {
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
            state.set(State::Idle);
            controlled.move_to = None;
        } else {
            state.set(State::Walking);
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

        if matches!(*state, State::Attacking(_)) || state.equals(State::Dashing) {
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
            state.set(State::Dashing);
            cooldown.reset();

            //TODO: Add dash stats
            let new_pos = transform.translation.xy() + (dir.normalize() * 50.);
            if let Some(mut ec) = commands.get_entity(entity) {
                ec.insert(EaseTo::new(new_pos, EaseFunction::EaseOutQuad, 0.5));
            }
        }
    }
}

pub fn finish_dash(
    mut query: Query<&mut State, With<Player>>,
    mut removals: RemovedComponents<EaseTo>,
) {
    for entity in removals.iter() {
        if let Ok(mut state) = query.get_mut(entity) {
            if state.equals(State::Dashing) {
                state.set(State::Idle);
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
        Option<&mut Combo>,
        Entity,
    )>,
    mut commands: Commands,
) {
    if let Ok((mut state, action_state, transform, direction, mut cooldown, combo, entity)) =
        query.get_single_mut()
    {
        if state.equals(State::Dashing) || matches!(*state, State::Attacking(_)) {
            return;
        }

        if action_state.just_pressed(PlayerActions::Attack) && cooldown.is_ready() {
            if let Some(mut combo) = combo {
                combo.current += 1;
                combo.timer.reset();
                if combo.current > combo.max {
                    combo.current = 0;
                }
                state.set(State::Attacking(combo.current));
            } else {
                state.set(State::Attacking(0));
                commands.entity(entity).insert(Combo {
                    current: 0,
                    max: 2,
                    timer: Timer::from_seconds(1., bevy::time::TimerMode::Once),
                });
            }

            cooldown.reset();

            //TODO: Add attack dash stats
            let new_pos = transform.translation.xy() + (direction.vec().normalize() * 5.);

            if let Some(mut ec) = commands.get_entity(entity) {
                ec.insert(AttackPhase {
                    charge: Timer::from_seconds(0.05, bevy::time::TimerMode::Once),
                    attack: Timer::from_seconds(0.25, bevy::time::TimerMode::Once),
                    recover: Timer::from_seconds(0.1, bevy::time::TimerMode::Once),
                })
                .insert(EaseTo::new(new_pos, EaseFunction::EaseOutQuad, 0.55));
            }
        }
    }
}

pub fn combo_system(
    mut query: Query<(&mut Combo, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut combo, entity) in query.iter_mut() {
        combo.timer.tick(time.delta());

        if combo.timer.finished() {
            commands.entity(entity).remove::<Combo>();
        }
    }
}
