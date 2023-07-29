use bevy::prelude::{
    in_state, Color, IntoSystemConfigs, Parent, Res, Text, Text2dBundle, TextAlignment, TextStyle,
    Timer, Update, Vec2,
};
use bevy::time::TimerMode;
use bevy::{
    math::Vec3Swizzles,
    prelude::{
        App, Camera, Commands, DespawnRecursiveExt, Entity, EventReader, Plugin, Query, Transform,
        With, Without,
    },
};
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};
use std::time::Duration;

use crate::attack::{EntitiesHit, Lifetime};
use crate::metadata::GameMeta;
use crate::stats::Revenge;
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
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(
                Update,
                (damageable_collision, xp_system).distributive_run_if(in_state(GameState::InGame)),
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
            (player_query.contains(*e1) && drop_query.contains(*e2)),
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

pub fn damageable_collision(
    mut events: EventReader<CollisionEvent>,
    mut damage_query: Query<(
        &Damage,
        Option<&mut EntitiesHit>,
        Option<&Knockback>,
        Option<&mut Breakable>,
        Option<&Parent>,
    )>,
    mut damageable_query: Query<(&mut Health, &Transform), With<Damageable>>,
    mut player_query: Query<&mut Revenge, With<Player>>,
    camera_query: Query<Entity, With<Camera>>,
    mut commands: Commands,
    game_meta: Res<GameMeta>,
) {
    events.iter().for_each(|e| {
        let (e1, e2, started, flags) = match e {
            CollisionEvent::Started(e1, e2, flags) => (e1, e2, true, flags),
            CollisionEvent::Stopped(e1, e2, flags) => (e1, e2, false, flags),
        };

        //If entity removed from world, don't handle collision
        if !started || *flags == CollisionEventFlags::REMOVED {
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
            let (damage, entities_hit, knockback, breakable, parent) =
                damage_query.get_mut(damage_entity).unwrap();

            if let Some(mut entities_hit) = entities_hit {
                if entities_hit.0.contains(&damaged_entity) {
                    return;
                } else {
                    entities_hit.0.push(damaged_entity);
                }
            }

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
                if let Some(mut ec) = commands.get_entity(damaged_entity) {
                    if health.current > 0 {
                        ec.insert(EaseTo::new(new_pos, EaseFunction::EaseOutExpo, 0.5));
                    }
                }
            }

            if let Some(parent) = parent {
                if let Ok(mut revenge) = player_query.get_mut(parent.get()) {
                    revenge.amount += damage.amount as f32 / 10.;
                }
            }

            //Switch this into a shake event
            if let Ok(camera) = camera_query.get_single() {
                commands.entity(camera).insert(Shake {
                    duration: 0.25,
                    strength: (damage.amount as f32 / 25.).powi(2),
                });
            }

            //TODO: Move this into a separate system using events
            let text_style = TextStyle {
                font: game_meta.text_font.clone(),
                font_size: 12.0,
                color: Color::WHITE,
            };
            let text_alignment = TextAlignment::Center;

            //Spawn damage indicator text
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(format!("-{}", damage.amount), text_style)
                        .with_alignment(text_alignment),
                    transform: Transform::from_translation(transform.translation.xy().extend(500.)),
                    ..Default::default()
                },
                EaseTo::new(
                    transform.translation.xy() + Vec2::new(0., 20.),
                    EaseFunction::EaseOutExpo,
                    1.,
                ),
                Lifetime(Timer::new(Duration::from_secs_f32(1.), TimerMode::Once)),
            ));
        }
    });
}
