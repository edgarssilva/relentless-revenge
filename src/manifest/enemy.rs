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
use bevy_spritesheet_animation::prelude::AnimationId;
use leafwing_manifest::{
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
};
use serde::{Deserialize, Serialize};

use crate::animation::Animations;

use super::{
    load_animations, load_attack_data, load_texture_data, AttackData, RawAnimationData,
    RawAttackData, RawTextureData,
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct RawEnemyData {
    name: String,
    health: u32,
    damage: u32,
    speed: u32,
    cooldown: u32,
    xp: u32,
    hitbox: Vec2,
    scale: Vec2,
    feet_offset: Option<f32>,
    attack: RawAttackData,
    texture: RawTextureData,
    animations: Vec<RawAnimationData>,
}

#[derive(Debug, PartialEq)]
pub struct EnemyData {
    pub name: String,
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
    pub cooldown: u32,
    pub xp: u32,
    pub hitbox: Vec2,
    pub scale: Vec2,
    pub attack: AttackData,
    pub feet_offset: Option<f32>,
    pub texture: Handle<Image>,
    pub atlas: Handle<TextureAtlasLayout>,
    pub animations: Animations,
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

                let enemy_data = EnemyData {
                    name: raw_enemy.name.clone(),
                    health: raw_enemy.health,
                    damage: raw_enemy.damage,
                    speed: raw_enemy.speed,
                    cooldown: raw_enemy.cooldown,
                    xp: raw_enemy.xp,
                    hitbox: raw_enemy.hitbox,
                    scale: raw_enemy.scale,
                    feet_offset: raw_enemy.feet_offset,
                    texture,
                    atlas,
                    attack: load_attack_data(&raw_enemy.attack, world),
                    animations: load_animations(&raw_enemy.name, &raw_enemy.animations, world),
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
