use bevy::prelude::{Bundle, Component};
use crate::stats::StatsBundle;

#[derive(Component)]
pub struct Boss;

#[derive(Bundle)]
pub struct BossBundle {
    boss: Boss,
    stats: StatsBundle,
}

