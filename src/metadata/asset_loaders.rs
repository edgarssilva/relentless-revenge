use std::path::{Path, PathBuf};

use bevy::asset::{
    AddAsset, Asset, AssetLoader, AssetPath, BoxedFuture, Error, Handle, LoadContext, LoadedAsset,
};
use bevy::prelude::App;

use crate::metadata::{AttackMeta, EnemyMeta, LevelMeta, PlayerMeta};

/// Calculate an asset's full path relative to another asset
fn relative_asset_path(asset_path: &Path, relative_path: &str) -> PathBuf {
    let is_relative = !relative_path.starts_with('/');

    if is_relative {
        let base = asset_path.parent().unwrap_or_else(|| Path::new(""));
        base.join(relative_path)
    } else {
        Path::new(relative_path)
            .strip_prefix("/")
            .unwrap()
            .to_owned()
    }
}

/// Helper to get relative asset paths and handles
fn get_relative_asset<T: Asset>(
    load_context: &LoadContext,
    self_path: &Path,
    relative_path: &str,
) -> (AssetPath<'static>, Handle<T>) {
    let asset_path = relative_asset_path(self_path, relative_path);
    let asset_path = AssetPath::new(asset_path, None);
    let handle = load_context.get_handle(asset_path.clone());

    (asset_path, handle)
}

pub struct EnemyMetaAssetLoader;

impl AssetLoader for EnemyMetaAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let self_path = load_context.path().to_owned();

            // let mut dependencies = vec![];

            let mut meta: EnemyMeta = serde_yaml::from_slice(bytes)?;

            meta.texture.load(
                load_context,
                get_relative_asset(load_context, &self_path, &meta.texture.path),
            );

            if let AttackMeta::Ranged { ref mut texture, .. } = meta.attack {
                texture.load(
                    load_context,
                    get_relative_asset(load_context, &self_path, &texture.path),
                );
            }

            load_context.set_default_asset(
                LoadedAsset::new(meta), // .with_dependencies(dependencies)
            );
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["enemy"]
    }
}

pub struct PlayerMetaAssetLoader;

impl AssetLoader for PlayerMetaAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let self_path = load_context.path().to_owned();

            // let mut dependencies = vec![];

            let mut meta: PlayerMeta = serde_yaml::from_slice(bytes)?;

            meta.texture.load(
                load_context,
                get_relative_asset(load_context, &self_path, &meta.texture.path),
            );

            load_context.set_default_asset(
                LoadedAsset::new(meta), // .with_dependencies(dependencies)
            );
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["player"]
    }
}

struct LevelMetaAssetLoader;

impl AssetLoader for LevelMetaAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let mut meta: LevelMeta = serde_yaml::from_slice(bytes)?;

            let self_path = load_context.path();

            let mut dependencies = vec![];

            for enemy_spawn in &mut meta.enemies {
                let (enemy_path, enemy_handle) =
                    get_relative_asset(load_context, self_path, &enemy_spawn.path);

                dependencies.push(enemy_path);
                enemy_spawn.enemy = enemy_handle;
            }

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level"]
    }
}

pub fn register_asset_loaders(app: &mut App) {
    app.add_asset_loader(EnemyMetaAssetLoader)
        .add_asset_loader(PlayerMetaAssetLoader)
        .add_asset_loader(LevelMetaAssetLoader);
}
