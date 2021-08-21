use bevy::prelude::{Commands, DespawnRecursiveExt, Entity, Query};

pub struct Stats {
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
}

impl Stats {
    pub fn new(health: u32, damage: u32, speed: u32) -> Self {
        Stats {
            health,
            damage,
            speed,
        }
    }
}

pub fn death_system(mut commands: Commands, query: Query<(Entity, &Stats)>) {
    for (entity, stats) in query.iter() {
        if stats.health <= 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
