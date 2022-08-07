use bevy::{
    input::Input,
    prelude::{Commands, Component, Entity, KeyCode, Query, Res, With, Without},
    render::camera::Camera,
};

use crate::{
    controller::PlayerControlled,
    direction::Direction,
    helper::{KeyMaps, Shake},
    state::State,
    stats::Stats,
};

#[derive(Component)]
pub struct MeleeSensor {
    pub dir: Direction,
    pub targets: Vec<Entity>,
}

impl MeleeSensor {
    pub fn from(dir: Direction) -> Self {
        Self {
            dir,
            targets: Vec::new(),
        }
    }
}

pub fn attack_system(
    mut player_query: Query<(&Stats, &Direction, &mut State), With<PlayerControlled>>,
    mut stats_query: Query<&mut Stats, Without<PlayerControlled>>,
    sensors_query: Query<&MeleeSensor>,
    keys: Res<Input<KeyCode>>,
    keymaps: Res<KeyMaps>,
    camera_query: Query<Entity, With<Camera>>,
    mut commands: Commands,
) {
    if !keys.just_pressed(keymaps.attack) {
        return;
    }

    if let Ok((attacker_stats, direction, mut state)) = player_query.get_single_mut() {
        if !attacker_stats.can_attack() {
            return;
        }

        state.set(State::ATTACKING);

        for sensor in sensors_query
            .iter()
            .filter(|sensor| sensor.dir == *direction)
        {
            for &attacked_entity in sensor.targets.iter() {
                if let Ok(mut attacked_stats) = stats_query.get_mut(attacked_entity) {
                    if attacked_stats.health < attacker_stats.damage {
                        attacked_stats.health = 0;
                    } else {
                        attacked_stats.health -= attacker_stats.damage;
                    }
                    if let Ok(camera) = camera_query.get_single() {
                        commands.entity(camera).insert(Shake {
                            duration: 0.25,
                            strength: 7.5,
                        });
                    }
                }
            }
        }
    }
}
