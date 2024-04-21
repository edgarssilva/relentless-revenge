use std::path::PathBuf;

use bevy::{
    app::Plugin,
    asset::{AssetServer, Assets, Handle},
    math::Vec2,
    render::texture::Image,
    sprite::TextureAtlasLayout,
};
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    plugin::{ManifestPlugin, RegisterManifest},
};
use serde::{Deserialize, Serialize};

use self::{enemy::EnemyManifest, floor::DomainManifest, player::PlayerManifest};

pub mod enemy;
pub mod floor;
pub mod player;

pub struct DataManifestPlugin {}

impl Plugin for DataManifestPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<SimpleAssetState>() //TODO: Check how to handle the loading state
            .add_plugins(ManifestPlugin::<SimpleAssetState>::default())
            .register_manifest::<EnemyManifest>("entities/enemies/data.yaml")
            .register_manifest::<PlayerManifest>("entities/player/player.yaml")
            .register_manifest::<DomainManifest>("domains.yaml");
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TextureData {
    pub path: PathBuf,
    pub tile_size: Vec2,
    pub columns: usize,
    pub rows: usize,
    pub padding: Option<Vec2>,
    pub offset: Option<Vec2>,
    #[serde(default)]
    pub frames: Vec<usize>,
    pub animation_duration: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum RawAttackData {
    Melee {
        size: Vec2,
        duration: f32,
        knockback: f32,
    },
    Ranged {
        size: Vec2,
        duration: f32,
        velocity: f32,
        texture: TextureData,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum AttackData {
    Melee {
        size: Vec2,
        duration: f32,
        knockback: f32,
    },
    Ranged {
        size: Vec2,
        duration: f32,
        velocity: f32,
        texture: Handle<Image>,
        atlas: Handle<TextureAtlasLayout>,
    },
}

pub fn load_texture_data(
    data: &TextureData,
    world: &mut bevy::prelude::World,
) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
    let asset_server = world.resource::<AssetServer>();
    let texture: Handle<Image> = asset_server.load(data.path.clone());

    let mut text_atlas_layout = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let layout = TextureAtlasLayout::from_grid(
        data.tile_size,
        data.columns,
        data.rows,
        data.padding,
        data.offset,
    );

    (texture, text_atlas_layout.add(layout))
}

pub fn load_attack_data(data: &RawAttackData, world: &mut bevy::prelude::World) -> AttackData {
    match data {
        RawAttackData::Ranged {
            size: raw_size,
            duration: raw_duration,
            velocity: raw_velocity,
            texture: raw_texture,
        } => {
            let (texture, atlas) = load_texture_data(&raw_texture, world);
            AttackData::Ranged {
                size: *raw_size,
                duration: *raw_duration,
                velocity: *raw_velocity,
                texture,
                atlas,
            }
        }
        RawAttackData::Melee {
            size: raw_size,
            duration: raw_duration,
            knockback: raw_knockback,
        } => AttackData::Melee {
            size: *raw_size,
            duration: *raw_duration,
            knockback: *raw_knockback,
        },
    }
}
