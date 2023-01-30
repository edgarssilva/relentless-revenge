use std::time::Duration;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, RigidBody,
};
use big_brain::{
    prelude::{ActionState, FirstToScore},
    scorers::Score,
    thinker::{Actor, Thinker, ThinkerBuilder},
    BigBrainPlugin, BigBrainStage,
};
use iyes_loopless::prelude::ConditionSet;

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
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(follow_player_action)
                    .with_system(seeking_scorer)
                    .into(),
            )
            .add_system_set_to_stage(
                BigBrainStage::Actions,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(follow_player_action)
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
    pub sprisheet: SpriteSheetBundle,
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
    pub fn new(texture_handle: Handle<TextureAtlas>, translation: Vec3) -> Self {
        Self {
            enemy: Enemy,
            sprisheet: SpriteSheetBundle {
                texture_atlas: texture_handle,
                transform: Transform {
                    translation,
                    scale: Vec3::new(0.25, 0.25, 1.0),
                    ..default()
                },
                ..default()
            },
            stats: StatsBundle {
                health: Health::new(100),
                damage: Damage::new(10),
                speed: MovementSpeed::new(20),
                xp: XP::new(10),
                cooldown: Cooldown::new(500),
            },
            damageable: Damageable,
            animation: Animation {
                frames: (0..7).collect(),
                current_frame: 0,
                timer: Timer::new(Duration::from_millis(250), bevy::time::TimerMode::Once),
            },
            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::cuboid(256. * 0.2, 256. * 0.2),
            collision_groups: CollisionGroups::new(BodyLayers::ENEMY, BodyLayers::PLAYER_ATTACK),
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::all(),
            // finding_player: FindingPLayer,
            thinker: Thinker::build()
                .picker(FirstToScore { threshold: 0.8 })
                .when(FindingPLayer, SeekPlayer),
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct FindingPLayer;

fn seeking_scorer(
    seekers: Query<&Transform, With<Enemy>>,
    player: Query<&Transform, With<Player>>,
    mut query: Query<(&Actor, &mut Score), With<FindingPLayer>>,
) {
    if let Ok(player_transform) = player.get_single() {
        for (Actor(actor), mut score) in query.iter_mut() {
            if let Ok(seeker_transform) = seekers.get(*actor) {
                let distance = player_transform
                    .translation
                    .xy()
                    .distance(seeker_transform.translation.xy());

                //TODO: Define a distance threshold
                if distance < 128. {
                    score.set(0.8);
                } else {
                    score.set(0.);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct SeekPlayer;

fn follow_player_action(
    mut commands: Commands,
    seekers: Query<(&Transform, Option<&Follow>), With<Enemy>>,
    player: Query<(Entity, &Transform), With<Player>>,
    mut query: Query<(&Actor, &mut ActionState), With<SeekPlayer>>,
    //Temporary for projectiles
    texture_assets: Res<TextureAssets>,
) {
    if let Ok((player, player_transform)) = player.get_single() {
        for (Actor(actor), mut state) in query.iter_mut() {
            if let Ok((seeker_transform, follow)) = seekers.get(*actor) {
                match *state {
                    ActionState::Requested => {
                        commands
                            .entity(*actor)
                            .insert(Follow::new(player, 0.10, true, 0.5));
                        //Temporary projectile spawning

                        let seeker_position = seeker_transform.translation.xy();
                        let player_position = player_transform.translation.xy();

                        let direction = (player_position - seeker_position).normalize();

                        commands.spawn((
                            ProjectileBundle::new(
                                texture_assets.arrow_atlas.clone(),
                                seeker_transform.translation.clone(),
                                f32::atan2(direction.y, direction.x),
                                Vec2::new(32., 32.) / 2.,
                                3.,
                                Damage::new(10),
                                false,
                                Velocity(direction * 75.),
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
                        *state = ActionState::Executing;
                    }

                    ActionState::Executing => {
                        if let Some(follow) = follow {
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
    }
}
