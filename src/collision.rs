use bevy::{
    math::Vec3Swizzles,
    prelude::{
        App, Camera, Commands, DespawnRecursiveExt, Entity, EventReader, Plugin, Query, Transform,
        With, Without,
    },
};
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};
use iyes_loopless::prelude::ConditionSet;

use crate::{
    attack::{Breakable, Damageable, Knockback},
    helper::Shake,
    movement::easing::{EaseFunction, EaseTo},
    player::Player,
    stats::{Damage, Drop, Health},
    GameState, XP,
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(xp_system)
                    .with_system(damagable_collision)
                    .into(),
            );
    }
}

#[derive(Copy, Clone)]
pub struct BodyLayers;

impl BodyLayers {
    pub const PLAYER: Group = Group::GROUP_1;
    pub const ENEMY: Group = Group::GROUP_2;
    pub const PLAYER_ATTACK: Group = Group::GROUP_3;
    pub const ENEMY_ATTACK: Group = Group::GROUP_4;
    pub const XP_LAYER: Group = Group::GROUP_5;
}

pub fn xp_system(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    drop_query: Query<&XP, (With<Drop>, Without<Player>)>,
    mut player_query: Query<&mut XP, (With<Player>, Without<Drop>)>,
) {
    events.iter().for_each(|e| {
        let (e1, e2, started, flags) = match e {
            CollisionEvent::Started(e1, e2, flags) => (e1, e2, true, flags),
            CollisionEvent::Stopped(e1, e2, flags) => (e1, e2, false, flags),
        };

        //If entity removed from world, don't handle collision
        if !started && *flags == CollisionEventFlags::REMOVED {
            return;
        }

        if let Some((drop_entity, player_entity)) = match (
            drop_query.contains(*e1) && player_query.contains(*e2),
            (player_query.contains(*e2) && drop_query.contains(*e1)),
        ) {
            (true, false) => Some((*e1, *e2)),
            (false, true) => Some((*e2, *e1)),
            _ => None,
        } {
            let drop_xp = drop_query.get(drop_entity).unwrap();
            let mut player_xp = player_query.get_mut(player_entity).unwrap();

            player_xp.add(drop_xp);
            commands.entity(drop_entity).despawn_recursive();
        }
    });
}

pub fn damagable_collision(
    mut events: EventReader<CollisionEvent>,
    mut damage_query: Query<(&Damage, Option<&Knockback>, Option<&mut Breakable>)>,
    mut damageable_query: Query<(&mut Health, &Transform), With<Damageable>>,
    camera_query: Query<Entity, With<Camera>>,
    mut commands: Commands,
) {
    events.iter().for_each(|e| {
        let (e1, e2, started, flags) = match e {
            CollisionEvent::Started(e1, e2, flags) => (e1, e2, true, flags),
            CollisionEvent::Stopped(e1, e2, flags) => (e1, e2, false, flags),
        };

        //If entity removed from world, don't handle collision
        if !started && *flags == CollisionEventFlags::REMOVED {
            return;
        }

        //TODO: Check what to do when both entities have damage and damageable
        if let Some((damage_entity, damaged_entity)) = match (
            damage_query.contains(*e1) && damageable_query.contains(*e2),
            (damage_query.contains(*e2) && damageable_query.contains(*e1)),
        ) {
            (true, false) => Some((*e1, *e2)),
            (false, true) => Some((*e2, *e1)),
            _ => None,
        } {
            let (damage, knockback, breakable) = damage_query.get_mut(damage_entity).unwrap();
            let (mut health, transform) = damageable_query.get_mut(damaged_entity).unwrap();

            health.damage(damage);

            //TODO: Handle on separate system
            if let Some(mut breakable) = breakable {
                if breakable.0 > 0 {
                    breakable.0 -= 1;
                }
            }

            if let Some(knockback) = knockback {
                let new_pos =
                    transform.translation.xy() + knockback.force * knockback.direction.vec();
                commands.entity(damaged_entity).insert(EaseTo::new(
                    new_pos,
                    EaseFunction::EaseOutExpo,
                    1.,
                ));
            }

            //Switch this into a shake event
            if let Ok(camera) = camera_query.get_single() {
                commands.entity(camera).insert(Shake {
                    duration: 0.35,
                    strength: 9.5,
                });
            }
        }
    });
}
