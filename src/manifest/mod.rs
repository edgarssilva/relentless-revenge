use std::path::PathBuf;

use bevy::{
    app::Plugin,
    asset::{AssetServer, Assets, Handle},
    image::{Image, TextureAtlasLayout},
    math::{UVec2, Vec2},
    platform::collections::HashMap,
    prelude::AppExtStates,
};
use bevy_spritesheet_animation::{
    clip::Clip,
    prelude::{Animation, AnimationDuration},
    spritesheet::Spritesheet,
};
use boss::BossManifest;
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    plugin::{ManifestPlugin, RegisterManifest},
};
use serde::{Deserialize, Serialize};

use crate::{
    animation::{Animations, DirectionalAnimations},
    movement::direction::Direction,
    state::State,
};

use self::{enemy::EnemyManifest, floor::DomainManifest, player::PlayerManifest};

pub mod boss;
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
            .register_manifest::<DomainManifest>("domains.yaml")
            .register_manifest::<BossManifest>("entities/enemies/bosses.yaml");
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RawTextureData {
    pub path: PathBuf,
    pub tile_size: UVec2,
    pub columns: u32,
    pub rows: u32,
    pub padding: Option<UVec2>,
    pub offset: Option<UVec2>,
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
        texture: RawTextureData,
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
    data: &RawTextureData,
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RawAnimationData {
    pub name: String,
    pub frames: Vec<usize>,
    pub duration: u32,
}

pub fn load_animations(
    spritesheet: &Spritesheet,
    data: &Vec<RawAnimationData>,
    world: &mut bevy::prelude::World,
) -> Animations {
    let mut animations = world.resource_mut::<Assets<Animation>>();

    Animations(
        data.iter()
            .map(|raw| {
                let animation = spritesheet
                    .create_animation()
                    .add_indices(raw.frames.clone())
                    .set_duration(AnimationDuration::PerFrame(raw.duration))
                    .build();

                (raw.name.clone(), animations.add(animation))
            })
            .collect(),
    )
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RawDirectionalAnimationData {
    pub duration: u32,
    pub state: State,
    pub directions: HashMap<Direction, Vec<usize>>,
}

pub fn load_directional_animations(
    spritesheet: &Spritesheet,
    data: &Vec<RawDirectionalAnimationData>,
    world: &mut bevy::prelude::World,
) -> DirectionalAnimations {
    let mut animations = world.resource_mut::<Assets<Animation>>();

    let mut states: HashMap<State, HashMap<Direction, Handle<Animation>>> = HashMap::new();

    for raw in data {
        states.insert(
            raw.state,
            raw.directions
                .iter()
                .map(|(dir, frames)| {
                    let animation = spritesheet
                        .create_animation()
                        .add_indices(frames.clone())
                        .set_duration(AnimationDuration::PerFrame(raw.duration))
                        .build();

                    (*dir, animations.add(animation))
                })
                .collect(),
        );
    }

    DirectionalAnimations(states)
}
