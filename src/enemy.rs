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

use crate::{
    animation::Animation,
    collision::BodyLayers,
    controller::PlayerControlled,
    follow::{Follow, FollowTarget},
    stats::Stats,
};

pub struct EnemyBehaviourPlugin;

impl Plugin for EnemyBehaviourPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BigBrainPlugin)
            .add_system_to_stage(BigBrainStage::Actions, follow_player_action)
            .add_system_to_stage(BigBrainStage::Scorers, seeking_scorer)
            .add_system(seeking_scorer);
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    #[bundle]
    pub sprisheet: SpriteSheetBundle,
    pub stats: Stats,
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
            stats: Stats::new(100, 20, 20, 2., 5),
            animation: Animation {
                frames: (0..7).collect(),
                current_frame: 0,
                timer: Timer::new(Duration::from_millis(250), true),
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
    player: Query<&Transform, With<PlayerControlled>>,
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
    seekers: Query<Option<&Follow>, With<Enemy>>,
    player: Query<Entity, With<PlayerControlled>>,
    mut query: Query<(&Actor, &mut ActionState), With<SeekPlayer>>,
) {
    if let Ok(player) = player.get_single() {
        for (Actor(actor), mut state) in query.iter_mut() {
            if let Ok(follow) = seekers.get(*actor) {
                match *state {
                    ActionState::Requested => {
                        commands.entity(*actor).insert(Follow::new(
                            FollowTarget::Transform(player),
                            0.10,
                            true,
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
