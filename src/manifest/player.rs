use std::convert::Infallible;

use bevy::{
    asset::{Asset, Handle},
    ecs::system::Resource,
    math::Vec2,
    reflect::TypePath,
    render::texture::Image,
    sprite::TextureAtlasLayout,
};

use leafwing_manifest::manifest::{Manifest, ManifestFormat};
use serde::{Deserialize, Serialize};

use crate::animation::DirectionalAnimations;

use super::{
    load_directional_animations, load_texture_data, RawDirectionalAnimationData, RawTextureData,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RawPlayerData {
    size: Vec2,
    health: u32,
    damage: u32,
    speed: u32,
    cooldown: u32,
    xp: u32,
    base_xp: u32,
    xp_multiplier: f32,
    hitbox: Vec2,
    feet_offset: Option<f32>,
    texture: RawTextureData,
    animations: Vec<RawDirectionalAnimationData>,
}

#[derive(Debug, PartialEq)]
pub struct PlayerData {
    pub size: Vec2,
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
    pub cooldown: u32,
    pub xp: u32,
    pub base_xp: u32,
    pub xp_multiplier: f32,
    pub hitbox: Vec2,
    pub feet_offset: Option<f32>,
    pub texture: Handle<Image>,
    pub atlas: Handle<TextureAtlasLayout>,
    pub animations: DirectionalAnimations,
}

#[derive(Debug, Asset, TypePath, Serialize, Deserialize, PartialEq)]
pub struct RawPlayerManifest(RawPlayerData);

#[derive(Debug, Resource, PartialEq)]
pub struct PlayerManifest {
    pub player_data: PlayerData,
}

impl Manifest for PlayerManifest {
    type RawManifest = RawPlayerManifest;

    type RawItem = RawPlayerData;

    type Item = PlayerData;

    type ConversionError = Infallible;

    const FORMAT: ManifestFormat = ManifestFormat::Yaml;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        world: &mut bevy::prelude::World,
    ) -> Result<Self, Self::ConversionError> {
        let raw_data = raw_manifest.0;

        let (texture, atlas) = load_texture_data(&raw_data.texture, world);

        let player_data = PlayerData {
            size: raw_data.size,
            health: raw_data.health,
            damage: raw_data.damage,
            speed: raw_data.speed,
            cooldown: raw_data.cooldown,
            xp: raw_data.xp,
            base_xp: raw_data.base_xp,
            xp_multiplier: raw_data.xp_multiplier,
            hitbox: raw_data.hitbox,
            feet_offset: raw_data.feet_offset,
            texture,
            atlas,
            animations: load_directional_animations(&raw_data.animations, world),
        };

        Ok(PlayerManifest { player_data })
    }

    fn get(&self, _id: leafwing_manifest::identifier::Id<Self::Item>) -> Option<&Self::Item> {
        Some(&self.player_data)
    }
}
