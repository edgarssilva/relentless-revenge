use bevy::prelude::{App, Plugin};

#[derive(Default)]
pub struct DifficultyResource {
    pub level: i32,
}

pub struct GenerateLevelEvent;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DifficultyResource::default())
            .add_event::<GenerateLevelEvent>();
    }
}
