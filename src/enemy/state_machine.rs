use std::time::Duration;

use bevy::math::{Vec2, Vec3Swizzles};
use bevy::prelude::{
    App, Commands, Component, Entity, FromReflect, Query, Reflect, Res, Time, Timer, TimerMode,
    Transform, With,
};
use iyes_loopless::condition::ConditionSet;
use seldom_state::prelude::*;

use crate::animation::Animation;
use crate::attack::ProjectileBundle;
use crate::enemy::Enemy;
use crate::game_states::loading::TextureAssets;
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
                .with_system(follow_player)
                .with_system(attack_player)
                .into(),
        );
}

pub(crate) fn get_state_machine() -> StateMachine {
    StateMachine::new(Wounder)
        .trans::<Wounder>(NearPlayer { range: 120.0 }, FollowPlayer)
        .trans::<FollowPlayer>(DoneTrigger::Success, Attack)
        .trans::<FollowPlayer>(NotTrigger(NearPlayer { range: 120.0 }), Wounder)
        .trans::<Attack>(DoneTrigger::Success, Wounder)
}

//States
#[derive(Component, Clone, Reflect)]
#[component(storage = "SparseSet")]
struct Wounder;

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

impl Trigger for NearPlayer {
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

fn _wounder() {
    todo!();
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
                    .insert(Follow::new(player, 0.10, true, 90.));
            }
        }
    }
}

fn attack_player(
    mut commands: Commands,
    mut enemies: Query<(Entity, &Transform, &mut Cooldown), (With<Enemy>, With<Attack>)>,
    player_query: Query<&Transform, With<Player>>,
    texture_assets: Res<TextureAssets>,
    delta_time: Res<Time>,
) {
    for (enemy, transform, mut cooldown) in enemies.iter_mut() {
        cooldown.update(delta_time.delta());

        if cooldown.is_ready() {
            if let Ok(player) = player_query.get_single() {
                let direction = (player.translation - transform.translation)
                    .xy()
                    .normalize();

                commands.spawn((
                    ProjectileBundle::new(
                        texture_assets.arrow_atlas.clone(),
                        transform.translation,
                        f32::atan2(direction.y, direction.x),
                        Vec2::new(100., 20.) / 2.,
                        3.,
                        Damage::new(10),
                        false,
                        Velocity(direction * 70.),
                    ),
                    Animation {
                        //TODO: Add animation to projectile
                        frames: (0..(6 * 5)).collect(),
                        current_frame: 0,
                        timer: Timer::new(Duration::from_millis(300), TimerMode::Once),
                    },
                ));

                cooldown.reset();
                commands.entity(enemy).insert(Done::Success);
            }
            // commands.entity(enemy).insert(Done::Failure);
        }
    }
}
