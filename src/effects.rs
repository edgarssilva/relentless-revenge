use crate::{game_states::loading::GameAssets, sorting::FeetOffset};
use bevy::prelude::*;

#[derive(Component)]
pub struct Shadow;

pub fn spawn_shadows(
    query: Query<(Entity, Option<&FeetOffset>), Added<Shadow>>,
    game_assets: Res<GameAssets>,
    mut commands: Commands,
) {
    for (entity, offset) in query.iter() {
        let offset = offset.map(|x| x.0).unwrap_or(0.0);
        commands.entity(entity).with_children(|parent| {
            parent.spawn(SpriteBundle {
                texture: game_assets.shadow_texture.clone(),
                transform: Transform::from_xyz(0., -offset, -0.5),
                ..Default::default()
            });
        });
    }
}
