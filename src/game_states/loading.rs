use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use leafwing_manifest::asset_state::SimpleAssetState;

use crate::{manifest::DataManifestPlugin, GameState};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DataManifestPlugin {})
            //.add_plugins(WorldInspectorPlugin::new())
            .add_systems(OnEnter(GameState::Loading), setup_assets)
            .add_systems(
                Update,
                check_loading_progress.run_if(in_state(GameState::Loading)),
            );
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

fn check_loading_progress(
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAssets>,
    manifest_state: Res<State<SimpleAssetState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let manifests_ready = *manifest_state.get() == SimpleAssetState::Ready;

    let assets_ready = asset_server.is_loaded_with_dependencies(&game_assets.font)
        && asset_server.is_loaded_with_dependencies(&game_assets.xp_texture)
        && asset_server.is_loaded_with_dependencies(&game_assets.map_texture)
        && asset_server.is_loaded_with_dependencies(&game_assets.shadow_texture);

    if manifests_ready && assets_ready {
        next_state.set(GameState::InGame);
    }
}
