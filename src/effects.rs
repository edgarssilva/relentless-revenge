use crate::metadata::GameMeta;
use bevy::prelude::*;

#[derive(Component)]
pub struct Shadow;

pub fn spawn_shadows(
    query: Query<Entity, Added<Shadow>>,
    meta: Res<GameMeta>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn(SpriteBundle {
                texture: meta.shadow_texture.clone(),
                transform: Transform::from_xyz(0., -14., -0.5),
                ..Default::default()
            });
        });
    }
}
