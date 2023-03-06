use std::time::Duration;

use bevy::math::{Vec2, Vec3Swizzles};
use bevy::prelude::{
    App, Commands, Component, Entity, EventWriter, FromReflect, Local, Query, Reflect, Res, Time,
    Timer, TimerMode, Transform, With,
};
use bevy::utils::HashMap;
use iyes_loopless::condition::ConditionSet;
use seldom_state::prelude::*;
use turborand::rng::Rng;
use turborand::TurboRand;

use crate::attack::SpawnEnemyAttack;
use crate::enemy::Enemy;
use crate::movement::movement::{Follow, Velocity};
use crate::player::Player;
use crate::stats::{Cooldown, Damage};
use crate::GameState;

pub(crate) fn register(app: &mut App) {
    app.add_plugin(StateMachinePlugin) //TODO: Move somewhere else
        .add_plugin(TriggerPlugin::<NearPlayer>::default())
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(idle)
                .with_system(wander)
                .with_system(follow_player)
                .with_system(attack_player)
                .into(),
        );
}

pub(crate) fn get_state_machine() -> StateMachine {
    StateMachine::new(Idle)
        .trans::<Idle>(DoneTrigger::Success, Wander)
        .trans::<Wander>(DoneTrigger::Success, Idle)
        .trans::<Wander>(NearPlayer { range: 120.0 }, FollowPlayer)
        .trans::<FollowPlayer>(DoneTrigger::Success, Attack)
        .trans::<FollowPlayer>(NotTrigger(NearPlayer { range: 120.0 }), Wander)
        .trans::<Attack>(DoneTrigger::Success, Wander)
        .remove_on_exit::<Wander, Velocity>()
}

//States
#[derive(Component, Clone, Reflect)]
#[component(storage = "SparseSet")]
struct Idle;

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
#[derive(Clone, Copy, FromReflect, Reflect)]
struct NearPlayer {
    range: f32,
}

impl BoolTrigger for NearPlayer {
    type Param<'w, 's> = (
        Query<'w, 's, &'static Transform, With<Enemy>>,
        Query<'w, 's, &'static Transform, With<Player>>,
    );

    fn trigger(&self, entity: Entity, (enemies, player): &Self::Param<'_, '_>) -> bool {
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
                Duration::from_millis(rand.u64(0..=1000)),
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
                    commands
                        .entity(entity)
                        .remove::<Velocity>()
                        .insert(Done::Success);
                }
            }
            continue;
        }

        let x = rand.i32(-1..1) as f32;
        let y = rand.i32(-5..5) as f32 / 10.;
        let speed = 15.;
        let direction = Vec2::new(x, y); //.normalize_or_zero();

        commands
            .entity(entity)
            .insert(Velocity(direction * speed, true));
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
                    commands
                        .entity(enemy)
                        .remove::<Follow>()
                        .insert(Done::Success);
                }
            } else {
                commands
                    .entity(enemy)
                    .insert(Follow::new(player, 0.10, true, 80.));
            }
        }
    }
}

fn attack_player(
    player_query: Query<&Transform, With<Player>>,
    mut event: EventWriter<SpawnEnemyAttack>,
    mut enemies: Query<(Entity, &Enemy, &Transform, &Damage, &mut Cooldown), With<Attack>>,
    mut commands: Commands,
) {
    for (entity, enemy, transform, damage, mut cooldown) in enemies.iter_mut() {
        if cooldown.is_ready() {
            if let Ok(player) = player_query.get_single() {
                let direction = (player.translation - transform.translation)
                    .xy()
                    .normalize();

                event.send(SpawnEnemyAttack {
                    meta: enemy.0.attack.clone(),
                    damage: *damage,
                    direction,
                    position: transform.translation,
                    enemy_size: enemy.0.hitbox,
                });
                cooldown.reset();
                commands.entity(entity).insert(Done::Success);
            }
            // commands.entity(enemy).insert(Done::Failure);
        }
    }
}
