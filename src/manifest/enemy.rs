use std::convert::Infallible;

use bevy::{
    asset::{Asset, Handle},
    ecs::system::Resource,
    math::Vec2,
    reflect::TypePath,
    render::texture::Image,
    sprite::TextureAtlasLayout,
    utils::HashMap,
};
use leafwing_manifest::{
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
};
use serde::{Deserialize, Serialize};

use super::{load_attack_data, load_texture_data, AttackData, RawAttackData, TextureData};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct RawEnemyData {
    name: String,
    health: u32,
    damage: u32,
    speed: u32,
    cooldown: u32,
    xp: u32,
    texture: TextureData,
    hitbox: Vec2,
    scale: Vec2,
    attack: RawAttackData,
    feet_offset: Option<f32>,
}

#[derive(Debug, PartialEq)]
pub struct EnemyData {
    pub name: String,
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
    pub cooldown: u32,
    pub xp: u32,
    pub texture: Handle<Image>,
    pub atlas: Handle<TextureAtlasLayout>,
    pub hitbox: Vec2,
    pub scale: Vec2,
    pub frames: Vec<usize>,
    pub frame_duration: u64,
    pub attack: AttackData,
    pub feet_offset: Option<f32>,
}

#[derive(Debug, Asset, TypePath, Serialize, Deserialize, PartialEq)]
pub struct RawEnemyManifest {
    enemies: Vec<RawEnemyData>,
}

#[derive(Debug, Resource, PartialEq)]
pub struct EnemyManifest {
    pub enemies: HashMap<Id<EnemyData>, EnemyData>,
}

impl Manifest for EnemyManifest {
    type Item = EnemyData;
    type RawItem = RawEnemyData;
    type RawManifest = RawEnemyManifest;

    type ConversionError = Infallible;

    const FORMAT: ManifestFormat = ManifestFormat::Yaml;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        world: &mut bevy::prelude::World,
    ) -> Result<Self, Self::ConversionError> {
        let enemies = raw_manifest
            .enemies
            .iter()
            .map(|raw_enemy| {
                let (texture, atlas) = load_texture_data(&raw_enemy.texture, world);

                let frames = if raw_enemy.texture.frames.is_empty() {
                    (0..raw_enemy.texture.columns * raw_enemy.texture.rows).collect()
                } else {
                    raw_enemy.texture.frames.clone()
                };

                let enemy_data = EnemyData {
                    name: raw_enemy.name.clone(),
                    health: raw_enemy.health,
                    damage: raw_enemy.damage,
                    speed: raw_enemy.speed,
                    cooldown: raw_enemy.cooldown,
                    xp: raw_enemy.xp,
                    hitbox: raw_enemy.hitbox,
                    scale: raw_enemy.scale,
                    frames,
                    feet_offset: raw_enemy.feet_offset,
                    texture,
                    atlas,
                    attack: load_attack_data(&raw_enemy.attack, world),
                    frame_duration: raw_enemy.texture.animation_duration,
                };

                (Id::from_name(raw_enemy.name.as_str()), enemy_data)
            })
            .collect();

        Ok(EnemyManifest { enemies })
    }

    fn get(&self, id: Id<Self::Item>) -> Option<&Self::Item> {
        self.enemies.get(&id)
    }
}
