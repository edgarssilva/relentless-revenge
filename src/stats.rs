use bevy::{
    math::Vec3Swizzles,
    prelude::{
        AssetServer, Bundle, Commands, Component, DespawnRecursiveExt, Entity, EventReader,
        EventWriter, Handle, Image, Query, Res, Time, Transform, Vec2, Vec3, With,
    },
    sprite::SpriteBundle,
};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups};

use crate::{
    collision::BodyLayers, enemy::Enemy, level::EnemyKilledEvent, movement::movement::Follow,
    player::Player,
};

#[derive(Component)]
pub struct Stats {
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
    pub attack_speed: f32,
    attack_timer: f32,
    pub xp: u32,
}

impl Stats {
    pub fn new(health: u32, damage: u32, speed: u32, attack_speed: f32, xp: u32) -> Self {
        Stats {
            health,
            damage,
            speed,
            attack_speed,
            attack_timer: 0.,
            xp,
        }
    }

    pub fn damage(&mut self, damage: u32) {
        self.health = self.health.saturating_sub(damage);
    }

    pub fn can_attack(&self) -> bool {
        self.attack_timer >= self.attack_speed
    }

    pub fn reset_attack_timer(&mut self) {
        self.attack_timer = 0.;
    }
}

pub fn death_system(
    mut commands: Commands,
    query: Query<(Entity, &Stats, Option<&Enemy>)>,
    mut enemy_kill_writer: EventWriter<EnemyKilledEvent>,
) {
    for (entity, stats, enemy) in query.iter() {
        if stats.health <= 0 {
            commands.entity(entity).despawn_recursive();

            if enemy.is_some() {
                enemy_kill_writer.send(EnemyKilledEvent(entity));
            }
        }
    }
}

pub fn attack_cooldown_system(mut query: Query<&mut Stats>, time: Res<Time>) {
    for mut stats in query.iter_mut() {
        stats.attack_timer += time.delta_seconds();
    }
}

#[derive(Component, Clone, Copy)]
pub struct XP(pub u32);

#[derive(Bundle)]
pub struct XPBundle {
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

impl XPBundle {
    pub fn spawn_enemy_drop(
        location: Vec2,
        xp: u32,
        commands: &mut Commands,
        texture: Handle<Image>,
        player: Entity,
    ) -> Entity {
        commands
            .spawn(XPBundle {
                xp: XP(xp),
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
    query: Query<(&Transform, &Stats)>,
    asset_server: Res<AssetServer>,
    player_query: Query<Entity, With<Player>>,
) {
    if let Ok(player) = player_query.get_single() {
        for event in enemy_kill_reader.iter() {
            if let Ok((transform, stats)) = query.get(event.0) {
                XPBundle::spawn_enemy_drop(
                    transform.translation.xy(),
                    stats.xp,
                    &mut commands,
                    asset_server.load("xp.png"),
                    player,
                );
            }
        }
    }
}
