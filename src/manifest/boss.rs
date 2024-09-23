use std::convert::Infallible;

use bevy::{
    asset::{Asset, Handle},
    math::Vec2,
    prelude::{Image, Resource},
    reflect::TypePath,
    sprite::TextureAtlasLayout,
    utils::HashMap,
};
use leafwing_manifest::{
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
};
use serde::{Deserialize, Serialize};

use crate::animation::Animations;

use super::{load_animations, load_texture_data, RawAnimationData, RawTextureData};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct RawBossData {
    name: String,
    health: u32,
    damage: u32,
    speed: u32,
    xp: u32,
    hitbox: Vec2,
    scale: Vec2,
    feet_offset: Option<f32>,
    texture: RawTextureData,
    animations: Vec<RawAnimationData>,
}

#[derive(Debug, PartialEq)]
pub struct BossData {
    pub name: String,
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
    pub xp: u32,
    pub hitbox: Vec2,
    pub scale: Vec2,
    pub feet_offset: Option<f32>,
    pub texture: Handle<Image>,
    pub atlas: Handle<TextureAtlasLayout>,
    pub animations: Animations,
}

#[derive(Debug, Asset, TypePath, Serialize, Deserialize, PartialEq)]
pub struct RawBossManifest {
    bosses: Vec<RawBossData>,
}

#[derive(Debug, Resource)]
pub struct BossManifest {
    pub bosses: HashMap<Id<BossData>, BossData>,
}

impl Manifest for BossManifest {
    type Item = BossData;
    type RawItem = RawBossData;
    type RawManifest = RawBossManifest;

    type ConversionError = Infallible;

    const FORMAT: ManifestFormat = ManifestFormat::Yaml;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        world: &mut bevy::prelude::World,
    ) -> Result<Self, Self::ConversionError> {
        let bosses = raw_manifest
            .bosses
            .iter()
            .map(|raw_boss| {
                let (texture, atlas) = load_texture_data(&raw_boss.texture, world);
                let animations = load_animations(&raw_boss.name, &raw_boss.animations, world);

                let enemy_data = BossData {
                    name: raw_boss.name.clone(),
                    health: raw_boss.health,
                    damage: raw_boss.damage,
                    speed: raw_boss.speed,
                    xp: raw_boss.xp,
                    hitbox: raw_boss.hitbox,
                    scale: raw_boss.scale,
                    feet_offset: raw_boss.feet_offset,
                    texture,
                    atlas,
                    animations,
                };

                (Id::from_name(raw_boss.name.as_str()), enemy_data)
            })
            .collect();

        Ok(BossManifest { bosses })
    }

    fn get(&self, id: Id<Self::Item>) -> Option<&Self::Item> {
        self.bosses.get(&id)
    }
}
