use bevy::asset::Assets;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::reflect::Array;
use bevy::{
    input::Input,
    math::Vec2,
    prelude::{
        App, Commands, Entity, EventReader, EventWriter, KeyCode, Plugin, Res, ResMut, Resource,
    },
};
use iyes_loopless::prelude::ConditionSet;
use turborand::rng::Rng;
use turborand::TurboRand;

use crate::map::generation::open_level_portal;
use crate::map::walkable::travel_through_portal;
use crate::metadata::{EnemyMeta, GameMeta, LevelMeta};
use crate::{enemy::EnemyBundle, GameState};

#[derive(Default, Resource)]
pub struct LevelResource {
    pub level: u32,
    pub meta: Option<LevelMeta>,
    pub enemies: Vec<Entity>,
}

//Level Generation Events
pub struct GenerateLevelEvent;
pub struct SpawnLevelEntitiesEvent(pub Vec<Vec<Vec2>>); // Available positions

//Level Clearing Events
pub struct EnemyKilledEvent(pub Entity); // Entity killed
pub struct LevelFinishedEvent; // All enemies killed
pub struct TriggerNextLevelEvent; // Player triggered next level

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelResource::default())
            .add_event::<GenerateLevelEvent>()
            .add_event::<SpawnLevelEntitiesEvent>()
            .add_event::<EnemyKilledEvent>()
            .add_event::<LevelFinishedEvent>()
            .add_event::<TriggerNextLevelEvent>()
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

fn keymap_generate(keys: Res<Input<KeyCode>>, mut writer: EventWriter<TriggerNextLevelEvent>) {
    if keys.just_pressed(KeyCode::LControl) {
        writer.send(TriggerNextLevelEvent);
    }
}

fn generate_level(
    mut event: EventReader<TriggerNextLevelEvent>,
    mut writer: EventWriter<GenerateLevelEvent>,
    mut level_resource: ResMut<LevelResource>,
    game_meta: Res<GameMeta>,
    levels: Res<Assets<LevelMeta>>,
) {
    for _ in event.iter() {
        level_resource.level += 1;

        //TODO: Optimize this
        let level_meta = game_meta
            .levels
            .iter()
            .find_map(|meta| {
                let meta = levels.get(meta.downcast_ref().unwrap()).unwrap();
                if level_resource.level >= meta.levels.0 && level_resource.level <= meta.levels.1 {
                    Some(meta)
                } else {
                    None
                }
            })
            .expect("No level found");

        level_resource.meta = Some(level_meta.clone());
        writer.send(GenerateLevelEvent);
    }
}

fn spawn_enemies(
    mut event: EventReader<SpawnLevelEntitiesEvent>,
    mut commands: Commands,
    enemies: Res<Assets<EnemyMeta>>,
    mut level: ResMut<LevelResource>,
) {
    for e in event.iter() {
        //TODO: Find a way to not clone this
        if let Some(meta) = &level.meta.clone() {
            let rand = Rng::new();
            let mut spawnable_room_positions = e.0.clone();

            let spawnable_enemies = meta.enemies.clone();
            let weight_count = spawnable_enemies.iter().map(|e| e.weight).sum::<u32>();

            for spawnable_positions in spawnable_room_positions.iter_mut() {
                rand.shuffle(spawnable_positions); //Shuffle positions
                let enemies_per_room = rand.u32(meta.enemies_per_room.0..=meta.enemies_per_room.1);

                for _ in 0..enemies_per_room {
                    let mut weight = rand.u32(0..=weight_count) as i32; //Get random weight
                    let pos = spawnable_positions.pop().unwrap(); //Get first position

                    for enemy in spawnable_enemies.iter() {
                        weight -= enemy.weight as i32;
                        if weight > 0 {
                            continue;
                        }

                        if let Some(enemy_meta) = enemies.get(&enemy.enemy) {
                            level.enemies.push(
                                commands
                                    .spawn(EnemyBundle::new(enemy_meta, pos.extend(1.0)))
                                    .id(),
                            );
                        }
                    }
                }
            }
        }
    }
}

fn enemy_killed(
    mut event: EventReader<EnemyKilledEvent>,
    mut level: ResMut<LevelResource>,
    mut portal_writer: EventWriter<LevelFinishedEvent>,
    mut commands: Commands,
) {
    for killed in event.iter() {
        level.enemies.retain(|e| *e != killed.0);

        commands.entity(killed.0).despawn_recursive();

        if level.enemies.is_empty() {
            portal_writer.send(LevelFinishedEvent);
        }
    }
}
