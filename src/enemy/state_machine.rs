use std::time::Duration;

use bevy::math::{Vec2, Vec3Swizzles};
use bevy::prelude::{
    in_state, App, Commands, Component, Entity, EventWriter, IntoSystemConfigs, Local, Query,
    Reflect, Res, Time, Timer, TimerMode, Transform, Update, With,
};
use bevy::utils::HashMap;
use seldom_state::prelude::*;
use seldom_state::trigger::NotTrigger;
use turborand::rng::Rng;
use turborand::TurboRand;

use crate::attack::SpawnEnemyAttack;
use crate::enemy::Enemy;
use crate::metadata::AttackMeta;
use crate::movement::movement::{Follow, Velocity};
use crate::player::Player;
use crate::stats::{Cooldown, Damage};
use crate::GameState;

pub(crate) fn register(app: &mut App) {
    app.add_plugins(StateMachinePlugin) //TODO: Move somewhere else
        .add_systems(
            Update,
            (idle, wander, follow_player, attack_player).run_if(in_state(GameState::InGame)),
        );
}

pub(crate) fn get_state_machine() -> StateMachine {
    let near_player = NearPlayer { range: 120.0 };

    StateMachine::default()
        .trans::<Idle>(DoneTrigger::Success, Wander)
        .trans::<Wander>(near_player, FollowPlayer)
        .trans::<Wander>(DoneTrigger::Success, Idle)
        .trans::<FollowPlayer>(DoneTrigger::Success, Attack)
        .trans::<FollowPlayer>(NotTrigger(near_player), Wander)
        .trans::<Attack>(DoneTrigger::Success, Idle)
}

//States
#[derive(Component, Clone, Reflect)]
#[component(storage = "SparseSet")]
pub struct Idle;

#[derive(Component, Clone, Reflect)]
#[component(storage = "SparseSet")]
struct Wander;

#[derive(Component, Clone, Reflect)]
#[component(storage = "SparseSet")]
struct FollowPlayer;

#[derive(Component, Clone, Reflect)]
#[component(storage = "SparseSet")]
struct Attack;

//Triggers
#[derive(Clone, Copy, Reflect)]
struct NearPlayer {
    range: f32,
}

impl BoolTrigger for NearPlayer {
    type Param<'w, 's> = (
        Query<'w, 's, &'static Transform, With<Enemy>>,
        Query<'w, 's, &'static Transform, With<Player>>,
    );

    fn trigger(&self, entity: Entity, (enemies, player): Self::Param<'_, '_>) -> bool {
        if let Ok(enemy_transform) = enemies.get(entity) {
            if let Ok(player_transform) = player.get_single() {
                let distance = player_transform
                    .translation
                    .xy()
                    .distance(enemy_transform.translation.xy());

                return distance <= self.range;
            }
        }

        false
    }
}

#[derive(Component)]
struct IdleDuration(Timer);

fn idle(
    mut commands: Commands,
    mut enemies: Query<(Entity, Option<&mut IdleDuration>), With<Idle>>,
    time: Res<Time>,
) {
    let rand = Rng::new();
    for (entity, timer) in enemies.iter_mut() {
        if let Some(mut timer) = timer {
            timer.0.tick(time.delta());

            if timer.0.finished() {
                commands
                    .entity(entity)
                    .remove::<IdleDuration>()
                    .insert(Done::Success);
            }
        } else {
            commands.entity(entity).insert(IdleDuration(Timer::new(
                Duration::from_millis(rand.u64(0..=1500)),
                TimerMode::Once,
            )));
        }
    }
}

fn wander(
    enemies: Query<(Entity, Option<&Velocity>), With<Wander>>,
    mut commands: Commands,
    mut timers: Local<HashMap<Entity, f32>>,
    time: Res<Time>,
) {
    let rand = Rng::new();
    for (entity, velocity) in enemies.iter() {
        if velocity.is_some() {
            if timers.contains_key(&entity) {
                let timer = timers.get_mut(&entity).unwrap();
                *timer += time.delta_seconds();

                if *timer >= rand.i32(1..=3) as f32 {
                    timers.remove(&entity);
                    if let Some(mut ec) = commands.get_entity(entity) {
                        ec.insert(Done::Success).remove::<Velocity>();
                    }
                }
            }
            continue;
        }

        let x = rand.i32(-1..1) as f32;
        let y = rand.i32(-5..5) as f32 / 10.;
        let speed = 15.;
        let direction = Vec2::new(x, y); //.normalize_or_zero();

        if let Some(mut ec) = commands.get_entity(entity) {
            ec.insert(Velocity(direction * speed, true));
        }
        timers.insert(entity, 0.);
    }
}

fn follow_player(
    enemies: Query<Entity, (With<Enemy>, With<FollowPlayer>)>,
    player: Query<Entity, With<Player>>,
    follows: Query<&Follow, With<Enemy>>,
    mut commands: Commands,
) {
    for enemy in enemies.iter() {
        if let Ok(player) = player.get_single() {
            if let Ok(follow) = follows.get(enemy) {
                if follow.on_target {
                    if let Some(mut ec) = commands.get_entity(enemy) {
                        ec.insert(Done::Success).remove::<Follow>();
                    }
                }
            } else if let Some(mut ec) = commands.get_entity(enemy) {
                ec.insert(Follow::new(player, 0.10, true, 80.));
            }
        }
    }
}

fn attack_player(
    player_query: Query<&Transform, With<Player>>,
    mut event: EventWriter<SpawnEnemyAttack>,
    mut enemies: Query<(Entity, &Enemy, &Transform, &Damage, &mut Cooldown), With<Attack>>,
    mut commands: Commands,
    mut durations: Local<HashMap<Entity, f32>>,
    time: Res<Time>,
) {
    for (entity, enemy, transform, damage, mut cooldown) in enemies.iter_mut() {
        if !durations.contains_key(&entity) && cooldown.is_ready() {
            if let Ok(player) = player_query.get_single() {
                let direction = (player.translation - transform.translation)
                    .xy()
                    .normalize();

                let duration = match enemy.0.attack {
                    AttackMeta::Melee { duration, .. } => duration,
                    AttackMeta::Ranged { duration, .. } => duration,
                };

                durations.insert(entity, duration);

                event.send(SpawnEnemyAttack {
                    meta: enemy.0.attack.clone(),
                    damage: *damage,
                    direction,
                    position: transform.translation,
                    enemy_size: enemy.0.hitbox,
                });
                cooldown.reset();
            }
            // commands.entity(enemy).insert(Done::Failure);
        }

        if let Some(duration) = durations.get_mut(&entity) {
            *duration -= time.delta_seconds();

            if *duration <= 0. {
                durations.remove(&entity);
                if let Some(mut ec) = commands.get_entity(entity) {
                    ec.insert(Done::Success);
                }
            }
        }
    }
}
