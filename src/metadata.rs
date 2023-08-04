use bevy::asset::{AssetPath, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy_asset_loader::asset_collection::AssetCollection;

pub mod asset_loaders;

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
    #[serde(default)]
    pub frames: Vec<usize>,
    pub duration: u64,
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

        if self.frames.is_empty() {
            self.frames = (0..self.columns * self.rows).collect();
        }
    }
}

#[derive(serde::Deserialize, TypePath, TypeUuid, Debug, Clone)]
#[uuid = "5e14f87b-d4f1-40e7-a2c8-b10ac660972b"]
pub struct EnemyMeta {
    // pub name: String,
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
    pub cooldown: u32,
    pub xp: u32,
    pub texture: TextureMeta,
    pub hitbox: Vec2,
    pub scale: Vec2,
    pub attack: AttackMeta,
}

#[derive(serde::Deserialize, TypePath, TypeUuid, Debug, Clone)]
#[uuid = "1b7e6673-6604-445a-b877-3735243a0b42"]
pub struct PlayerMeta {
    pub size: Vec2,
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
    pub cooldown: u32,
    pub xp: u32,
    pub texture: TextureMeta,
    pub hitbox: Vec2,
    //TODO: Add animation data
}

#[derive(serde::Deserialize, TypePath, TypeUuid, Debug, Clone)]
#[uuid = "18c28813-20dc-4494-a63a-3071a5be69f3"]
pub struct FloorMeta {
    pub floors: (u32, u32),
    pub rooms: (u32, u32),
    pub room_size: (u32, u32),
    pub enemies_per_room: (u32, u32),

    pub enemies: Vec<SpawnMeta>,
    // pub boss: BossMeta
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SpawnMeta {
    pub path: String,
    #[serde(skip)]
    pub enemy: Handle<EnemyMeta>,
    pub weight: u32,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AttackMeta {
    Melee {
        size: Vec2,
        duration: f32,
        knockback: f32,
    },
    Ranged {
        size: Vec2,
        duration: f32,
        velocity: f32,
        texture: TextureMeta,
    },
}

#[derive(serde::Deserialize, TypePath, TypeUuid, Debug, Clone)]
#[uuid = "86e2573e-1b08-4c7c-9959-3a8d9cec1b0d"]
pub struct LevelProgressionMeta {
    pub base_xp: u32,
    pub xp_multiplier: f32,
}

impl LevelProgressionMeta {
    pub(crate) fn xp_to_level_up(&self, level: u32) -> u32 {
        (self.base_xp as f32 * self.xp_multiplier.powi(level as i32)) as u32
    }
}

#[derive(AssetCollection, Resource)]
pub struct GameMeta {
    #[asset(path = "entities/enemies", collection(typed))]
    pub enemies: Vec<Handle<EnemyMeta>>,
    #[asset(path = "entities/player/yaml.player")]
    pub player: Handle<PlayerMeta>,
    #[asset(path = "BitPotionExt.ttf")]
    pub text_font: Handle<Font>,
    #[asset(path = "floors", collection(typed))]
    pub floors: Vec<Handle<FloorMeta>>,
    #[asset(path = "level.progression.yaml")]
    pub level_progression: Handle<LevelProgressionMeta>,
    #[asset(path = "shadow.png")]
    pub shadow_texture: Handle<Image>,
}

pub fn register_assets(app: &mut App) {
    app.add_asset::<EnemyMeta>()
        .add_asset::<PlayerMeta>()
        .add_asset::<FloorMeta>()
        .add_asset::<LevelProgressionMeta>();
}
