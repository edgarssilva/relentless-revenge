use bevy::{
    input::Input,
    math::Vec2,
    prelude::{
        App, AssetServer, Assets, Commands, Entity, EventReader, EventWriter, KeyCode, Plugin, Res,
        ResMut, Resource,
    },
    sprite::TextureAtlas,
};
use iyes_loopless::prelude::ConditionSet;

use crate::{enemy::EnemyBundle, GameState};

#[derive(Default, Resource)]
pub struct LevelResource {
    pub level: i32,
    pub enemies: Vec<Entity>,
}

pub struct GenerateLevelEvent;
pub struct GenerateMapEvent;

pub struct EnemyKilledEvent(pub Entity);

pub struct SpawnEnemiesEvent {
    pub positions: Vec<Vec2>,
}
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelResource::default())
            .add_event::<GenerateLevelEvent>()
            .add_event::<GenerateMapEvent>()
            .add_event::<SpawnEnemiesEvent>()
            .add_event::<EnemyKilledEvent>()
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(enemy_killed)
                    .with_system(spawn_enemies)
                    .with_system(generate_level)
                    .with_system(keymap_generate)
                    .into(),
            );
    }
}

fn keymap_generate(keys: Res<Input<KeyCode>>, mut map_writer: EventWriter<GenerateLevelEvent>) {
    if keys.just_pressed(KeyCode::LControl) {
        map_writer.send(GenerateLevelEvent);
    }
}

fn generate_level(
    mut event: EventReader<GenerateLevelEvent>,
    mut level: ResMut<LevelResource>,
    mut map_writer: EventWriter<GenerateMapEvent>,
) {
    for _ in event.iter() {
        level.level += 1;
        map_writer.send(GenerateMapEvent);
        return;
    }
}

fn spawn_enemies(
    mut event: EventReader<SpawnEnemiesEvent>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut level: ResMut<LevelResource>,
) {
    for e in event.iter() {
        //Load the textures
        let texture_handle = asset_server.load("monster_flesh_eye_sheet.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::splat(256.), 3, 3, None, None);
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        for pos in e.positions.iter() {
            level.enemies.push(
                commands
                    .spawn(EnemyBundle::new(
                        texture_atlas_handle.clone(),
                        pos.extend(1.0),
                    ))
                    .id(),
            );
        }
    }
}

fn enemy_killed(
    mut event: EventReader<EnemyKilledEvent>,
    mut level: ResMut<LevelResource>,
    mut level_writer: EventWriter<GenerateLevelEvent>,
) {
    for killed in event.iter() {
        level.enemies.retain(|e| *e != killed.0);

        if level.enemies.len() == 0 {
            level_writer.send(GenerateLevelEvent);
        }
    }
}
