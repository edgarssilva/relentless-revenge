use bevy::prelude::{Res, Time, Timer, Assets, Handle, TextureAtlas, TextureAtlasSprite, Query};

/*struct Animation {
    frames: [i32],
    duration: f32,
    repeat: bool,
}

pub struct SpriteAnimation {

}*/


//Cycles through all the animations TODO: Needs different animations
pub fn sprite_animation(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        }
    }
}
