use bevy::math::{Vec2, Vec3Swizzles};
use bevy::prelude::{App, Commands, Component, Entity, Plugin, Query, Res, Time, Transform};

use super::easing::ease_to_position;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(follow_entity_system)
            .add_system(movement_system)
            .add_system(ease_to_position);
    }
}

#[derive(Component)]
pub struct Follow {
    pub target: Entity,
    pub speed: f32,
    pub continous: bool,
    pub treshhold: f32,
    pub(crate) on_target: bool,
}

impl Follow {
    pub fn new(target: Entity, speed: f32, continous: bool, treshhold: f32) -> Self {
        Self {
            target,
            speed,
            continous,
            treshhold,
            on_target: false,
        }
    }

    /* pub fn on_target(self: &Self) -> bool {
        self.on_target
    } */
}

//System for an entity to follow another
pub fn follow_entity_system(
    mut commands: Commands,
    mut query_followers: Query<(&mut Follow, Entity)>,
    mut transform_query: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (mut follow, entity) in query_followers.iter_mut() {
        let pos: Vec2 = if let Ok(transform) = transform_query.get_mut(follow.target) {
            transform.translation.xy()
        } else {
            //Entity was removed or does not exist
            commands.entity(entity).remove::<Follow>();
            continue;
        };

        if let Ok(mut transform) = transform_query.get_mut(entity) {
            //TODO: Check distance threshold (This was added because of Changed<>)
            if transform.translation.xy().distance(pos) > follow.treshhold {
                transform.translation = transform
                    .translation
                    .xy()
                    .lerp(pos, follow.speed * time.delta_seconds())
                    .extend(transform.translation.z);

                follow.on_target = false;
            } else {
                follow.on_target = true;

                if !follow.continous {
                    commands.entity(entity).remove::<Follow>();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

pub fn movement_system(mut query_velocity: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query_velocity.iter_mut() {
        transform.translation += velocity.0.extend(0.) * time.delta_seconds();
    }
}
