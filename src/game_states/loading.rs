use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_egui::EguiPlugin;

use crate::GameState;
use crate::metadata::{GameMeta, register_assets};
use crate::metadata::asset_loaders::register_asset_loaders;


pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        register_assets(app);
        register_asset_loaders(app);

        app.add_plugin(EguiPlugin)
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::InGame)
                    .with_collection::<TextureAssets>()
                    .with_collection::<GameMeta>(),
            );
    }
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
  /*  #[asset(texture_atlas(tile_size_x = 100., tile_size_y = 100., columns = 6, rows = 5))]
    #[asset(path = "arrow.png")]
    pub arrow_atlas: Handle<TextureAtlas>,*/

    #[asset(path = "xp.png")]
    pub xp_texture: Handle<Image>,

    #[asset(path = "tileset.png")]
    pub map_texture: Handle<Image>,

}
