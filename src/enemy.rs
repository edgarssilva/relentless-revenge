use std::time::Duration;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, RigidBody,
};
use big_brain::actions::Steps;
use big_brain::prelude::{ActionBuilder, ScorerBuilder};
use big_brain::{
    prelude::{ActionState, FirstToScore},
    scorers::Score,
    thinker::{Actor, Thinker, ThinkerBuilder},
    BigBrainPlugin, BigBrainStage,
};
use iyes_loopless::prelude::ConditionSet;

use crate::metadata::EnemyMeta;
use crate::{
    animation::Animation,
    attack::{Damageable, ProjectileBundle},
    collision::BodyLayers,
    game_states::loading::TextureAssets,
    movement::movement::{Follow, Velocity},
    player::Player,
    stats::{Cooldown, Damage, Health, MovementSpeed, StatsBundle, XP},
    GameState,
};

pub struct EnemyBehaviourPlugin;

impl Plugin for EnemyBehaviourPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BigBrainPlugin)
            .add_system_set_to_stage(
                BigBrainStage::Actions,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(follow_player_action)
                    .with_system(attack_player_action)
                    .into(),
            )
            .add_system_set_to_stage(
                BigBrainStage::Scorers,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(seeking_scorer)
                    .into(),
            );
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    #[bundle]
    pub spritesheet: SpriteSheetBundle,
    #[bundle]
    pub stats: StatsBundle,
    pub damageable: Damageable,
    pub animation: Animation,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub active_events: ActiveEvents,
    pub active_collision_types: ActiveCollisionTypes,
    // finding_player: FindingPLayer,
    thinker: ThinkerBuilder,
}

impl EnemyBundle {
    pub fn new(meta: &EnemyMeta, translation: Vec3) -> Self {
        Self {
            enemy: Enemy,
            spritesheet: SpriteSheetBundle {
                texture_atlas: meta.texture.atlas_handle.clone(),
                transform: Transform {
                    translation,
                    scale: Vec3::new(0.25, 0.25, 1.0),
                    ..default()
                },
                ..default()
            },
            stats: StatsBundle {
                health: Health::new(meta.health),
                damage: Damage::new(meta.damage),
                speed: MovementSpeed::new(meta.speed),
                xp: XP::new(meta.xp),
                cooldown: Cooldown::new(meta.cooldown),
            },
            damageable: Damageable,
            animation: Animation {
                frames: meta.texture.frames.clone(),
                current_frame: 0,
                timer: Timer::new(
                    Duration::from_millis(meta.texture.duration),
                    bevy::time::TimerMode::Once,
                ),
            },
            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::cuboid(meta.hitbox.x / 2., meta.hitbox.y / 2.),
            collision_groups: CollisionGroups::new(BodyLayers::ENEMY, BodyLayers::PLAYER_ATTACK),
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::all(),
            // finding_player: FindingPLayer,
            thinker: Thinker::build()
                .label("Enemy Behaviour")
                .picker(FirstToScore { threshold: 0.8 })
                .when(
                    NearPlayer,
                    Steps::build()
                        .label("Find then attack player")
                        .step(SeekPlayer)
                        .step(AttackPlayer),
                ),
        }
    }
}

#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct NearPlayer;

fn seeking_scorer(
    seekers: Query<&Transform, With<Enemy>>,
    player: Query<&Transform, With<Player>>,
    mut query: Query<(&Actor, &mut Score), With<NearPlayer>>,
) {
    if let Ok(player_transform) = player.get_single() {
        for (Actor(actor), mut score) in query.iter_mut() {
            if let Ok(seeker_transform) = seekers.get(*actor) {
                let distance = player_transform
                    .translation
                    .xy()
                    .distance(seeker_transform.translation.xy());

                //TODO: Define a distance threshold
                if distance < 120. {
                    score.set(0.8);
                } else {
                    score.set(0.);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct SeekPlayer;

fn follow_player_action(
    mut commands: Commands,
    seekers: Query<&Follow, With<Enemy>>,
    player: Query<Entity, With<Player>>,
    mut query: Query<(&Actor, &mut ActionState), With<SeekPlayer>>,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
        match *state {
            ActionState::Requested => {
                if let Ok(player) = player.get_single() {
                    commands
                        .entity(*actor)
                        .insert(Follow::new(player, 0.10, true, 90.));
                    *state = ActionState::Executing;
                } else {
                    *state = ActionState::Failure;
                }
            }

            ActionState::Executing => {
                if let Ok(follow) = seekers.get(*actor) {
                    if follow.on_target {
                        *state = ActionState::Success;
                    }
                }
            }

            ActionState::Success => {
                commands.entity(*actor).remove::<Follow>();
            }

            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct AttackPlayer;

fn attack_player_action(
    mut commands: Commands,
    seekers: Query<&Transform, With<Enemy>>,
    mut cooldowns: Query<&mut Cooldown, With<Enemy>>,
    player: Query<&Transform, With<Player>>,
    mut query: Query<(&Actor, &mut ActionState), With<AttackPlayer>>,
    texture_assets: Res<TextureAssets>,
    delta_time: Res<Time>,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
        match *state {
            ActionState::Requested => {
                if let Ok(seeker_transform) = seekers.get(*actor) {
                    if let Ok(player_transform) = player.get_single() {
                        let seeker_position = seeker_transform.translation.xy();
                        let player_position = player_transform.translation.xy();

                        let direction = (player_position - seeker_position).normalize();

                        commands.spawn((
                            ProjectileBundle::new(
                                texture_assets.arrow_atlas.clone(),
                                seeker_transform.translation,
                                f32::atan2(direction.y, direction.x),
                                Vec2::new(32., 32.) / 2.,
                                3.,
                                Damage::new(10),
                                false,
                                Velocity(direction * 70.),
                            ),
                            Animation {
                                //TODO: Add animation to projectile
                                frames: (0..(6 * 5)).collect(),
                                current_frame: 0,
                                timer: Timer::new(
                                    Duration::from_millis(300),
                                    bevy::time::TimerMode::Once,
                                ),
                            },
                        ));

                        if let Ok(mut cooldown) = cooldowns.get_mut(*actor) {
                            cooldown.reset();
                        }

                        *state = ActionState::Executing;
                        continue;
                    }
                }
                *state = ActionState::Failure;
            }
            ActionState::Executing => {
                if let Ok(mut cooldown) = cooldowns.get_mut(*actor) {
                    cooldown.update(delta_time.delta());

                    if cooldown.is_ready() {
                        *state = ActionState::Success;
                    }
                }
            }

            ActionState::Success => {}

            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
