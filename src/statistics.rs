use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use bevy::utils::Duration;
use bevy_persistent::prelude::*;
use serde::{Deserialize, Serialize};

use crate::player::Player;

#[derive(Resource, Serialize, Deserialize, Debug, Clone, Default)]
pub struct Statistics {
    pub kills: u32,
    pub deaths: u32,
    pub dashes: u32,
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub max_xp: u32,
    pub max_level: u32,
    pub revenge_time: f32,
    pub play_time: f32,
    pub game_count: u32,
}

pub fn auto_save(
    time: Res<Time>,
    statistics: ResMut<Persistent<Statistics>>,
    mut timer: Local<Timer>,
) {
    timer.set_duration(Duration::from_secs_f32(25.));
    timer.set_mode(TimerMode::Repeating);
    timer.tick(time.delta());

    if timer.just_finished() {
        statistics.persist();
    }
}

pub fn statistics(
    mut statistics: ResMut<Persistent<Statistics>>,
    query: Query<&Player>,
    time: Res<Time>,
) {
    for _ in query.iter() {
        statistics.play_time += time.delta().as_secs_f32();
    }
}
