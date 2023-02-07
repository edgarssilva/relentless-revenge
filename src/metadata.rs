pub mod asset_loaders;

use bevy::asset::{ AssetPath, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TextureMeta {
    pub path: String,
    /// The image size in pixels
    pub(crate) tile_size: Vec2,
    /// Columns on the sprite sheet
    pub(crate) columns: usize,
    /// Rows on the sprite sheet
    pub(crate) rows: usize,
    /// Padding between columns in pixels
    pub(crate) padding: Option<Vec2>,
    /// Number of pixels offset of the first tile
    pub(crate) offset: Option<Vec2>,
    #[serde(skip)]
    pub atlas_handle: Handle<TextureAtlas>,
    pub frames: Option<Vec<u32>>,
    pub duration: f32,
}

//TODO: Find a way to organize this better and make sure the label don't collide
impl TextureMeta {
    pub(crate) fn load(
        &mut self,
        load_context: &mut LoadContext,
        (texture_path, texture_handle): (AssetPath, Handle<Image>),
    ) {
        self.atlas_handle = load_context.set_labeled_asset(
            &self.path,
            LoadedAsset::new(TextureAtlas::from_grid(
                texture_handle,
                self.tile_size,
                self.columns,
                self.rows,
                self.padding,
                self.offset,
            ))
            .with_dependency(texture_path),
        );

        if self.frames.is_none() {
            self.frames = Some((0..(self.columns * self.rows) as u32).collect());
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct HitboxMeta {
    pub width: u32,
    pub height: u32,
}

#[derive(serde::Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "5e14f87b-d4f1-40e7-a2c8-b10ac660972b"]
pub struct EnemyMeta {
    pub name: String,
    pub health: u32,
    pub damage: u32,
    pub attack_speed: f32,
    pub xp: u32,
    pub texture: TextureMeta,
    pub hitbox: HitboxMeta,
}

#[derive(AssetCollection, Resource)]
pub struct GameMeta {
    #[asset(path = "entities/enemies/enemy.yaml")]
    pub enemy: Handle<EnemyMeta>,
}

pub fn register_assets(app: &mut App) {
    app.add_asset::<EnemyMeta>();
}
