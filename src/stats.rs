use std::time::Duration;

use bevy::prelude::{Assets, Time};
use bevy::{
    math::Vec3Swizzles,
    prelude::{
        Bundle, Commands, Component, DespawnRecursiveExt, Entity, EventReader, EventWriter, Handle,
        Image, Query, Res, Transform, Vec2, Vec3, With,
    },
    sprite::SpriteBundle,
    time::Timer,
};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups};

use crate::metadata::{GameMeta, LevelProgressionMeta};
use crate::{
    collision::BodyLayers, enemy::Enemy, floor::EnemyKilledEvent,
    game_states::loading::TextureAssets, movement::movement::Follow, player::Player,
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
        if let Some(health) = self.current.checked_sub(damage.amount) {
            self.current = health;
        } else {
            self.current = 0;
        }
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
pub struct Level {
    pub level: u32,
}

impl Level {
    pub(crate) fn default() -> Level {
        Self { level: 1 }
    }
}

#[derive(Component)]
pub struct Revenge {
    pub amount: f32,
    pub decay: f32,
    pub active_decay: f32,
    pub active: bool,
    pub total: f32,
}

impl Revenge {
    pub fn decay(&self) -> f32 {
        match self.active {
            true => self.active_decay,
            false => self.decay,
        }
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
        if health.current == 0 {
            if enemy.is_some() {
                enemy_kill_writer.send(EnemyKilledEvent(entity));
            } else {
                commands.entity(entity).despawn_recursive();
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
                follow: Follow::new(player, 2.5, false, 0.1),
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
    query: Query<(&Transform, &XP), With<Enemy>>,
    texture_assets: Res<TextureAssets>,
    player_query: Query<Entity, With<Player>>,
) {
    if let Ok(player) = player_query.get_single() {
        for event in enemy_kill_reader.iter() {
            if let Ok((transform, xp)) = query.get(event.0) {
                XPDropBundle::spawn_enemy_drop(
                    transform.translation.xy(),
                    xp.amount,
                    &mut commands,
                    texture_assets.xp_texture.clone(),
                    player,
                );
            }
        }
    }
}

pub fn level_up(
    mut query: Query<
        (
            &XP,
            &mut Health,
            &mut Damage,
            &mut MovementSpeed,
            &mut Level,
        ),
        With<Player>,
    >,
    game_meta: Res<GameMeta>,
    level_progression: Res<Assets<LevelProgressionMeta>>,
) {
    let progression = level_progression.get(&game_meta.level_progression);
    if let Some(progression) = progression {
        for (xp, mut health, mut damage, mut speed, mut level) in query.iter_mut() {
            if xp.amount >= progression.xp_to_level_up(level.level) {
                //TODO: Add proper stats progression
                health.max += (health.max as f32 * progression.xp_multiplier / 100.) as u32;
                health.current = health.max;

                speed.speed += (speed.speed as f32 * progression.xp_multiplier / 100.) as u32;

                damage.amount += (damage.amount as f32 * progression.xp_multiplier / 50.) as u32;

                level.level += 1;
            }
        }
    }
}

pub fn revenge_mode(
    mut query: Query<(&mut Revenge, &mut Damage, &mut MovementSpeed)>,
    time: Res<Time>,
) {
    for (mut revenge, mut damage, mut speed) in query.iter_mut() {
        if revenge.active {
            if revenge.amount <= 0. {
                damage.amount = (damage.amount as f32 / 1.5) as u32;
                speed.speed = (speed.speed as f32 / 1.5) as u32;
                revenge.active = false;
            }
        } else if revenge.amount >= revenge.total {
            damage.amount = (damage.amount as f32 * 1.5) as u32;
            speed.speed = (speed.speed as f32 * 1.5) as u32;
            revenge.active = true;
        }

        let decay = revenge.decay() * time.delta_seconds();

        if revenge.amount > revenge.total {
            revenge.amount = revenge.total;
        }

        if revenge.amount > decay {
            revenge.amount -= decay;
        } else {
            revenge.amount = 0.;
        }
    }
}
