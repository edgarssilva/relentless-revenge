use crate::stats::StatsBundle;
use bevy::prelude::{Bundle, Component};

#[derive(Component)]
pub struct Boss;

#[derive(Bundle)]
pub struct BossBundle {
    boss: Boss,
    stats: StatsBundle,
}
