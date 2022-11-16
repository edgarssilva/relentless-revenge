use std::time::Duration;

use bevy::{
    math::Vec3Swizzles,
    prelude::{
        AssetServer, Bundle, Commands, Component, DespawnRecursiveExt, Entity, EventReader,
        EventWriter, Handle, Image, Query, Res, Transform, Vec2, Vec3, With,
    },
    sprite::SpriteBundle,
    time::Timer,
};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups};

use crate::{
    collision::BodyLayers, enemy::Enemy, level::EnemyKilledEvent, movement::movement::Follow,
    player::Player,
};

#[derive(Component)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

impl Health {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    pub fn damage(&mut self, damage: &Damage) {
        self.current -= damage.amount;
    }
}

#[derive(Component, Clone, Copy)]
pub struct Damage {
    pub amount: u32,
}

impl Damage {
    pub fn new(amount: u32) -> Self {
        Self { amount }
    }
}

#[derive(Component)]
pub struct MovementSpeed {
    pub speed: u32,
}

impl MovementSpeed {
    pub fn new(speed: u32) -> Self {
        Self { speed }
    }
}

#[derive(Component)]
pub struct XP {
    pub amount: u32,
}

impl XP {
    pub fn new(xp: u32) -> Self {
        Self { amount: xp }
    }

    pub fn add(&mut self, other: &Self) {
        self.amount += other.amount;
    }
}

#[derive(Component)]
pub struct Cooldown {
    pub timer: Timer,
}
impl Cooldown {
    pub fn new(millis: u32) -> Self {
        Self {
            timer: Timer::from_seconds(millis as f32 / 1000., bevy::time::TimerMode::Once),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.timer.finished()
    }

    pub fn reset(&mut self) {
        self.timer.reset();
    }

    pub fn update(&mut self, time: Duration) {
        self.timer.tick(time);
    }
}

//TODO: Check if a new method is needed
#[derive(Bundle)]
pub struct StatsBundle {
    pub health: Health,
    pub damage: Damage,
    pub speed: MovementSpeed,
    pub xp: XP,
    pub cooldown: Cooldown,
}

pub fn death_system(
    mut commands: Commands,
    query: Query<(Entity, &Health, Option<&Enemy>)>,
    mut enemy_kill_writer: EventWriter<EnemyKilledEvent>,
) {
    for (entity, health, enemy) in query.iter() {
        if health.current <= 0 {
            commands.entity(entity).despawn_recursive();

            if enemy.is_some() {
                enemy_kill_writer.send(EnemyKilledEvent(entity));
            }
        }
    }
}

//TODO: Move this to a separate file
#[derive(Component)]
pub struct Drop;

#[derive(Bundle)]
pub struct XPDropBundle {
    pub drop: Drop,
    pub xp: XP,
    //TODO: Check if this a simple sprite or a sprite sheet animation
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    pub follow: Follow,
    pub collider: Collider,
    pub collision_events: ActiveEvents,
    pub collision_types: ActiveCollisionTypes,
    pub collision_groups: CollisionGroups,
}

impl XPDropBundle {
    pub fn spawn_enemy_drop(
        location: Vec2,
        xp: u32,
        commands: &mut Commands,
        texture: Handle<Image>,
        player: Entity,
    ) -> Entity {
        commands
            .spawn(XPDropBundle {
                drop: Drop,
                xp: XP::new(xp),
                sprite_bundle: SpriteBundle {
                    texture,
                    transform: Transform::from_translation(Vec3::new(location.x, location.y, 3.)),
                    ..Default::default()
                },
                follow: Follow::new(player, 2., false, 0.5),
                collider: Collider::ball(4.),
                collision_events: ActiveEvents::COLLISION_EVENTS,
                collision_types: ActiveCollisionTypes::all(),
                collision_groups: CollisionGroups::new(BodyLayers::XP_LAYER, BodyLayers::PLAYER),
            })
            .id()
    }
}

pub fn drop_xp_system(
    mut commands: Commands,
    mut enemy_kill_reader: EventReader<EnemyKilledEvent>,
    query: Query<&Transform, With<Drop>>,
    asset_server: Res<AssetServer>,
    player_query: Query<Entity, With<Player>>,
) {
    if let Ok(player) = player_query.get_single() {
        for event in enemy_kill_reader.iter() {
            if let Ok(transform) = query.get(event.0) {
                XPDropBundle::spawn_enemy_drop(
                    transform.translation.xy(),
                    5, // stats.xp,
                    &mut commands,
                    asset_server.load("xp.png"),
                    player,
                );
            }
        }
    }
}
