use crate::{
    direction::Direction,
    movement::easing::{EaseFunction, EaseTo},
    state::State,
    stats::Stats,
    KeyMaps,
};
use bevy::{
    math::{Vec2, Vec3Swizzles},
    prelude::{
        Commands, Component, Entity, Input, KeyCode, Query, Res, Time, Transform, With, Without,
    },
};

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component)]
pub struct Controlled {
    pub move_to: Option<Vec2>,
}

//Player Movement
pub fn player_controller(
    mut commands: Commands,
    mut query: Query<
        (
            &mut Controlled,
            &mut Transform,
            Option<&Stats>,
            &mut Direction,
            &mut State,
            Entity,
        ),
        (With<PlayerControlled>, Without<EaseTo>),
    >,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mapping: Res<KeyMaps>,
) {
    for (mut controlled, transform, stats, mut direction, mut state, entity) in query.iter_mut() {
        let mut dir = Vec2::ZERO;

        if keys.pressed(mapping.walk_up) {
            dir += Vec2::Y;
            direction.set(Direction::NORTH);
        }

        if keys.pressed(mapping.walk_left) {
            dir -= Vec2::X;
            direction.set(Direction::WEST);
        }

        if keys.pressed(mapping.walk_down) {
            dir -= Vec2::Y;
            direction.set(Direction::SOUTH);
        }

        if keys.pressed(mapping.walk_right) {
            dir += Vec2::X;
            direction.set(Direction::EAST);
        }

        if keys.just_pressed(mapping.dash) {
            let dash_dir = if dir == Vec2::ZERO {
                direction.vec()
            } else {
                dir
            };

            state.set(State::DASHING);

            //TODO: Add dash stats
            let new_pos = transform.translation.xy() + (dash_dir * 50.);
            commands
                .entity(entity)
                .insert(EaseTo::new(new_pos, EaseFunction::EaseOutExpo, 1.));
            controlled.move_to = None;
            return;
        }

        let speed: u32 = if let Some(stats) = stats {
            stats.speed
        } else {
            45 /*TODO: Check default movement speed*/
        };

        let dir = dir.normalize_or_zero() * speed as f32 * time.delta_seconds();

        if dir.x == 0. && dir.y == 0. {
            if state.equals(State::WALKING) {
                state.set(State::IDLE);
            }
            controlled.move_to = None;
        } else {
            controlled.move_to = Some(transform.translation.xy() + dir);
            state.set(State::WALKING);
        }
    }
}
