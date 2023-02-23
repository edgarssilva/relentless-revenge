use bevy::asset::Assets;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::{
    input::Input,
    math::Vec2,
    prelude::{
        App, Commands, Entity, EventReader, EventWriter, KeyCode, Plugin, Res, ResMut, Resource,
    },
};
use iyes_loopless::prelude::ConditionSet;

use crate::map::generation::open_level_portal;
use crate::map::walkable::travel_through_portal;
use crate::metadata::{EnemyMeta, GameMeta};
use crate::{enemy::EnemyBundle, GameState};

#[derive(Default, Resource)]
pub struct LevelResource {
    pub level: i32,
    pub enemies: Vec<Entity>,
}

pub struct GenerateLevelEvent;

pub struct EnemyKilledEvent(pub Entity);

pub struct SpawnEnemiesEvent {
    pub positions: Vec<Vec2>,
}

pub struct OpenLevelPortalEvent;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelResource::default())
            .add_event::<GenerateLevelEvent>()
            .add_event::<SpawnEnemiesEvent>()
            .add_event::<EnemyKilledEvent>()
            .add_event::<OpenLevelPortalEvent>()
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(enemy_killed)
                    .with_system(spawn_enemies)
                    .with_system(generate_level)
                    .with_system(keymap_generate)
                    .with_system(open_level_portal)
                    .with_system(travel_through_portal)
                    .into(),
            );
    }
}

fn keymap_generate(keys: Res<Input<KeyCode>>, mut map_writer: EventWriter<GenerateLevelEvent>) {
    if keys.just_pressed(KeyCode::LControl) {
        map_writer.send(GenerateLevelEvent);
    }
}

fn generate_level(mut event: EventReader<GenerateLevelEvent>, mut level: ResMut<LevelResource>) {
    for _ in event.iter() {
        level.level += 1;
    }
}

fn spawn_enemies(
    mut event: EventReader<SpawnEnemiesEvent>,
    mut commands: Commands,
    game_meta: Res<GameMeta>,
    enemies: Res<Assets<EnemyMeta>>,
    mut level: ResMut<LevelResource>,
) {
    for e in event.iter() {
        for pos in e.positions.iter() {
            level.enemies.push(
                commands
                    .spawn(EnemyBundle::new(
                        enemies.get(&game_meta.enemy).unwrap(),
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
    mut portal_writer: EventWriter<OpenLevelPortalEvent>,
    mut commands: Commands,
) {
    for killed in event.iter() {
        level.enemies.retain(|e| *e != killed.0);

        commands.entity(killed.0).despawn_recursive();

        if level.enemies.is_empty() {
            portal_writer.send(OpenLevelPortalEvent);
        }
    }
}
