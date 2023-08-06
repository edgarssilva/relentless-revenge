use std::collections::BTreeMap;

use bevy::asset::Assets;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{
    in_state, Camera, Event, IntoSystemConfigs, Query, Transform, Update, With, Without,
};
use bevy::{
    input::Input,
    math::Vec2,
    prelude::{
        App, Commands, Entity, EventReader, EventWriter, KeyCode, Plugin, Res, ResMut, Resource,
    },
};
use noisy_bevy::simplex_noise_2d;
use turborand::rng::Rng;
use turborand::TurboRand;

use crate::enemy::state_machine::Idle;
use crate::map::generation::open_level_portal;
use crate::map::walkable::travel_through_portal;
use crate::metadata::{EnemyMeta, FloorMeta, GameMeta};
use crate::player::Player;
use crate::{enemy::EnemyBundle, GameState};

#[derive(Default, Resource)]
pub struct FloorResource {
    pub floor: u32,
    pub meta: Option<FloorMeta>,
    pub enemies: Vec<Entity>,
}

//Floor Generation Events
#[derive(Event)]
pub struct GenerateFloorEvent;

#[derive(Event)]
pub struct SpawnFloorEntitiesEvent {
    pub spawnable_pos: Vec<Vec2>,
    pub player_pos: Vec2,
}

//Floor Clearing Events
#[derive(Event)]
pub struct EnemyKilledEvent(pub Entity); // Entity killed

#[derive(Event)]
pub struct FloorClearedEvent; // All enemies killed

#[derive(Event)]
pub struct TriggerNextFloorEvent; // Player triggered next level

pub struct FloorPlugin;

impl Plugin for FloorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FloorResource::default())
            .add_event::<GenerateFloorEvent>()
            .add_event::<SpawnFloorEntitiesEvent>()
            .add_event::<EnemyKilledEvent>()
            .add_event::<FloorClearedEvent>()
            .add_event::<TriggerNextFloorEvent>()
            .add_systems(
                Update,
                (
                    move_player,
                    enemy_killed,
                    spawn_enemies,
                    generate_floor,
                    keymap_generate,
                    open_level_portal,
                    travel_through_portal,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn keymap_generate(keys: Res<Input<KeyCode>>, mut writer: EventWriter<TriggerNextFloorEvent>) {
    if keys.just_pressed(KeyCode::ControlLeft) {
        writer.send(TriggerNextFloorEvent);
    }
}

fn generate_floor(
    mut event: EventReader<TriggerNextFloorEvent>,
    mut writer: EventWriter<GenerateFloorEvent>,
    mut floor_resource: ResMut<FloorResource>,
    game_meta: Res<GameMeta>,
    floors: Res<Assets<FloorMeta>>,
) {
    for _ in event.iter() {
        floor_resource.floor += 1;

        //TODO: Optimize this
        let floor_meta = game_meta
            .floors
            .iter()
            .find_map(|meta| {
                let meta = floors.get(meta).unwrap();
                if floor_resource.floor >= meta.floors.0 && floor_resource.floor <= meta.floors.1 {
                    Some(meta)
                } else {
                    None
                }
            })
            .expect("No floor found");

        floor_resource.meta = Some(floor_meta.clone());
        writer.send(GenerateFloorEvent);
    }
}

fn move_player(
    mut player_query: Query<&mut Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut event: EventReader<SpawnFloorEntitiesEvent>,
) {
    for e in event.iter() {
        let pos = e.player_pos;

        if let Ok(mut transform) = player_query.get_single_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }

        if let Ok(mut transform) = camera_query.get_single_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

fn spawn_enemies(
    mut event: EventReader<SpawnFloorEntitiesEvent>,
    mut commands: Commands,
    enemies: Res<Assets<EnemyMeta>>,
    mut level: ResMut<FloorResource>,
) {
    for e in event.iter() {
        if let Some(meta) = &level.meta {
            let rand = Rng::new();
            let spawnable_pos = &e.spawnable_pos;

            let spawnable_enemies = meta.enemies.clone();
            let enemy_count = rand.u32(meta.enemies_count.0..=meta.enemies_count.1);
            let weight_count = spawnable_enemies.iter().map(|e| e.weight).sum::<u32>();

            let mut pos_noise = spawnable_pos
                .iter()
                .map(|p| ((simplex_noise_2d(*p) * 100.) as i32, p))
                .collect::<BTreeMap<i32, &Vec2>>();

            for _ in 0..enemy_count {
                if let Some(pos) = pos_noise.pop_last() {
                    let mut weight = rand.u32(0..=weight_count) as i32;

                    for enemy in spawnable_enemies.iter() {
                        weight -= enemy.weight as i32;
                        if weight > 0 {
                            continue;
                        }

                        if let Some(enemy_meta) = enemies.get(&enemy.enemy) {
                            level.enemies.push(
                                commands
                                    .spawn(EnemyBundle::new(enemy_meta, pos.1.extend(38.0)))
                                    .insert(Idle)
                                    .id(),
                            );
                        }

                        break;
                    }
                }
            }
        }
    }
}

fn enemy_killed(
    mut event: EventReader<EnemyKilledEvent>,
    mut level: ResMut<FloorResource>,
    mut portal_writer: EventWriter<FloorClearedEvent>,
    mut commands: Commands,
) {
    for killed in event.iter() {
        level.enemies.retain(|e| *e != killed.0);

        //        commands.entity(killed.0).despawn_recursive();
        if let Some(ec) = commands.get_entity(killed.0) {
            ec.despawn_recursive();
        }

        if level.enemies.is_empty() {
            portal_writer.send(FloorClearedEvent);
        }
    }
}
