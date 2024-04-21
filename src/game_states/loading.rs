use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use leafwing_manifest::asset_state::SimpleAssetState;

use crate::{manifest::DataManifestPlugin, GameState};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin, DataManifestPlugin {}))
            .add_systems(OnEnter(GameState::Loading), setup_assets)
            .add_systems(OnEnter(SimpleAssetState::Ready), finish_loading);
    }
}

#[derive(Resource)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub xp_texture: Handle<Image>,
    pub map_texture: Handle<Image>,
    pub shadow_texture: Handle<Image>,
}

fn setup_assets(asset_server: Res<AssetServer>, mut commands: Commands) {
    let font = asset_server.load("BitPotionExt.ttf");
    let xp_texture = asset_server.load("xp.png");
    let map_texture = asset_server.load("tileset.png");
    let shadow_texture = asset_server.load("shadow.png");

    commands.insert_resource(GameAssets {
        font,
        xp_texture,
        map_texture,
        shadow_texture,
    });
}

fn finish_loading(mut next_state: ResMut<NextState<GameState>>) {
    //TODO: Check if our own assets have loaded aswell
    next_state.set(GameState::InGame);
}
