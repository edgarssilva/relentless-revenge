use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::GameState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::InGame)
                .with_collection::<TextureAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 8, rows = 8))]
    #[asset(path = "tiny_hero.png")]
    pub player_atlas: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 100., tile_size_y = 100., columns = 6, rows = 5))]
    #[asset(path = "arrow.png")]
    pub arrow_atlas: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 256., tile_size_y = 256., columns = 3, rows = 3))]
    #[asset(path = "monster_flesh_eye_sheet.png")]
    pub enemy_atlas: Handle<TextureAtlas>,

    #[asset(path = "xp.png")]
    pub xp_texture: Handle<Image>,

    #[asset(path = "tileset.png")]
    pub map_texture: Handle<Image>,
}
