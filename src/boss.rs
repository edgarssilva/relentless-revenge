use crate::{
    animation::Animations,
    attack::Damageable,
    collision::BodyLayers,
    effects::Shadow,
    enemy::state_machine,
    manifest::boss::BossData,
    sorting::{self, FeetOffset, YSort},
    stats::StatsBundle,
    Cooldown, Damage, Health, MovementSpeed, XP,
};
use bevy::{
    math::Vec3,
    prelude::{default, Bundle, Component, Transform},
    sprite::{SpriteBundle, TextureAtlas},
};
use bevy_rapier2d::prelude::{ActiveEvents, Collider, CollisionGroups, RigidBody};
use bevy_spritesheet_animation::prelude::SpritesheetAnimation;
use seldom_state::prelude::StateMachine;

#[derive(Component)]
pub struct Boss(pub String);

#[derive(Bundle)]
pub struct BossBundle {
    boss: Boss,
    stats: StatsBundle,
    damageable: Damageable,

    state_matchine: StateMachine,

    //Sprite
    ysort: YSort,
    shadow: Shadow,
    sprite: SpriteBundle,
    atlas: TextureAtlas,
    animation: SpritesheetAnimation,
    animations: Animations,
    feet_offset: FeetOffset,

    //Physics
    collider: Collider,
    rigid_body: RigidBody,
    active_events: ActiveEvents,
    collision_groups: CollisionGroups,
}

impl BossBundle {
    pub fn new(data: &BossData, translation: Vec3) -> Self {
        Self {
            boss: Boss(data.name.clone()),

            //TODO: Extract this
            stats: StatsBundle {
                health: Health::new(data.health),
                damage: Damage::new(data.damage),
                speed: MovementSpeed::new(data.speed),
                xp: XP::new(data.xp),
                cooldown: Cooldown::new(0), //Check this
            },
            damageable: Damageable,

            ysort: YSort(sorting::ENTITIES_LAYER),
            shadow: Shadow,

            //TODO: Extract this
            sprite: SpriteBundle {
                texture: data.texture.clone(),
                transform: Transform {
                    translation,
                    scale: data.scale.extend(1.),
                    ..default()
                },
                ..default()
            },
            atlas: TextureAtlas {
                layout: data.atlas.clone(),
                index: 0,
            },
            animation: SpritesheetAnimation::from_id(
                *data
                    .animations
                    .0
                    .get("idle")
                    .expect(format!("No idle animation for {}", data.name).as_str()),
            ),
            animations: data.animations.clone(),
            feet_offset: FeetOffset(data.feet_offset.unwrap_or_default()),

            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::cuboid(data.hitbox.x / 2., data.hitbox.y / 2.),
            collision_groups: CollisionGroups::new(BodyLayers::ENEMY, BodyLayers::PLAYER_ATTACK),
            active_events: ActiveEvents::COLLISION_EVENTS,

            state_matchine: state_machine::get_state_machine(),
        }
    }
}
