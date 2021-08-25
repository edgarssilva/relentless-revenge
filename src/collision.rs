use bevy::prelude::{Entity, EventReader, Query};
use heron::{CollisionData, CollisionEvent, PhysicsLayer};

#[derive(PhysicsLayer)]
pub enum Layers {
    Player,
    Enemy,
    Attack,
}

pub fn melee_collisions(
    mut events: EventReader<CollisionEvent>,
    mut query: Query<&mut crate::MeleeSensor>, //TODO: Abstract this so any entity can have a list of collided entities
) {
    events.iter().for_each(|e| match e {
        CollisionEvent::Started(d1, d2) => {
            //Enemy entered players attack sensor
            if let Some((sensor, enemy)) = enemy_on_sensor(d1, d2) {
                if let Ok(mut melee_sensor) = query.get_mut(sensor) {
                    melee_sensor.targets.push(enemy);
                }
            }
        }
        CollisionEvent::Stopped(d1, d2) => {
            //Enemy left players attack sensor
            if let Some((sensor, enemy)) = enemy_on_sensor(d1, d2) {
                if let Ok(mut melee_sensor) = query.get_mut(sensor) {
                    melee_sensor.targets.retain(|&entity| entity != enemy);
                }
            }
        }
    });
}

fn enemy_on_sensor(d1: &CollisionData, d2: &CollisionData) -> Option<(Entity, Entity)> {
    if is_attack(d1) && is_enemy(d2) {
        Some((d1.collision_shape_entity(), d2.rigid_body_entity()))
    } else if is_attack(d2) && is_enemy(d1) {
        Some((d2.rigid_body_entity(), d1.collision_shape_entity()))
    } else {
        None
    }
}

fn is_attack(data: &CollisionData) -> bool {
    data.collision_layers()
        .contains_group(crate::Layers::Attack)
}

fn _is_player(data: &CollisionData) -> bool {
    data.collision_layers().contains_group(Layers::Player)
}

fn is_enemy(data: &CollisionData) -> bool {
    data.collision_layers().contains_group(Layers::Enemy)
}
