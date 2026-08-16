use std::collections::BTreeMap;
use std::time::Duration;

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::input::ButtonInput;
use bevy::prelude::{in_state, Camera, Query, Transform, Update, With, Without};
use bevy::time::Timer;
use bevy::{
    math::Vec2,
    prelude::{App, Commands, Entity, KeyCode, Plugin, Res, ResMut, Resource},
};
use leafwing_manifest::manifest::Manifest;
use noisy_bevy::simplex_noise_2d;
use turborand::rng::Rng;
use turborand::TurboRand;

use crate::boss::BossBundle;
//use crate::enemy::state_machine::Idle;
use crate::layers::world_z;
use crate::manifest::boss::BossManifest;
use crate::manifest::enemy::EnemyManifest;
use crate::manifest::floor::{DomainData, DomainManifest};
use crate::map::generation::open_level_portal;
use crate::map::walkable::{restrict_movement, travel_through_portal};
use crate::player::Player;
use crate::{enemy::EnemyBundle, GameState};

#[derive(Default, Resource)]
pub struct FloorResource {
    pub floor: u32,
    pub domain: Option<DomainData>,
    pub enemies: Vec<Entity>,
    pub boss: Option<Entity>,
}

//Floor Generation Events
#[derive(Message)]
pub struct GenerateFloorMessage;

#[derive(Message)]
pub struct SpawnFloorEntitiesMessage {
    pub spawnable_pos: Vec<Vec2>,
    pub player_pos: Vec2,
    pub portal_pos: Vec2,
}

//Floor Clearing Events
#[derive(Message)]
pub struct EnemyKilledMessage(pub Entity); // Entity killed

//TODO: This two can be made events, and observed
#[derive(Message)]
pub struct FloorClearedMessage; // All enemies killed

#[derive(Message)]
pub struct TriggerNextFloorMessage; // Player triggered next level

pub struct FloorPlugin;

impl Plugin for FloorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FloorResource::default())
            .add_message::<GenerateFloorMessage>()
            .add_message::<SpawnFloorEntitiesMessage>()
            .add_message::<EnemyKilledMessage>()
            .add_message::<FloorClearedMessage>()
            .add_message::<TriggerNextFloorMessage>()
            .add_systems(
                Update,
                (
                    new_domain_trigger,
                    move_player,
                    enemy_killed,
                    //spawn_enemies,
                    spawn_boss,
                    generate_floor,
                    keymap_generate,
                    open_level_portal,
                    travel_through_portal,
                    restrict_movement,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn keymap_generate(
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<TriggerNextFloorMessage>,
) {
    if keys.just_pressed(KeyCode::ControlLeft) {
        writer.write(TriggerNextFloorMessage);
    }
}

fn new_domain_trigger(
    mut commands: Commands,
    mut event: MessageReader<GenerateFloorMessage>,
    floor: Res<FloorResource>,
) {
    if event.is_empty() {
        return;
    }

    if let Some(domain) = &floor.domain {
        if floor.floor == domain.floors.0 {
            //TODO: Spawn domain title
        }
    }

    event.clear();
}

fn generate_floor(
    mut event: MessageReader<TriggerNextFloorMessage>,
    mut writer: MessageWriter<GenerateFloorMessage>,
    mut floor_resource: ResMut<FloorResource>,
    domain_manifest: Res<DomainManifest>,
) {
    for _ in event.read() {
        floor_resource.floor += 1;

        //TODO: Optimize this
        let domain = domain_manifest
            .domains
            .values()
            .find_map(|domain| {
                if floor_resource.floor >= domain.floors.0
                    && floor_resource.floor <= domain.floors.1
                {
                    Some(domain)
                } else {
                    None
                }
            })
            .expect("No floor found");

        floor_resource.domain = Some(domain.clone());
        writer.write(GenerateFloorMessage);
    }
}

fn move_player(
    mut player_query: Query<&mut Transform, (With<Player>, Without<Camera>)>,
    //mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut event: MessageReader<SpawnFloorEntitiesMessage>,
) {
    for e in event.read() {
        let pos = e.player_pos;

        if let Ok(mut transform) = player_query.single_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
            transform.translation.z = world_z::PLAYER;
        }

        /*if let Ok(mut transform) = camera_query.single_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }*/
    }
}

fn spawn_boss(
    mut commands: Commands,
    boss_manifest: Res<BossManifest>,
    mut floor: ResMut<FloorResource>,
    mut event: MessageReader<SpawnFloorEntitiesMessage>,
) {
    for e in event.read() {
        if let Some(domain) = &floor.domain {
            if domain.floors.1 != floor.floor {
                return;
            }

            if let Some(boss) = boss_manifest.get_by_name(domain.boss.clone()) {
                floor.boss = Some(
                    commands
                        .spawn(BossBundle::new(boss, e.portal_pos.extend(world_z::ENEMY)))
                        //.insert(Idle)
                        .id(),
                );
            }
        }
    }
}

fn spawn_enemies(
    mut commands: Commands,
    enemy_manifest: Res<EnemyManifest>,
    mut floor: ResMut<FloorResource>,
    mut event: MessageReader<SpawnFloorEntitiesMessage>,
) {
    for e in event.read() {
        if let Some(domain) = &floor.domain {
            if domain.floors.1 == floor.floor {
                return;
            }

            let rand = Rng::new();
            let spawnable_pos = &e.spawnable_pos;

            let spawnable_enemies = domain.enemies.clone();
            let enemy_count = rand.u32(domain.enemies_count.0..=domain.enemies_count.1);
            let weight_count = spawnable_enemies.iter().map(|e| e.0).sum::<u32>();

            let mut pos_noise = spawnable_pos
                .iter()
                .map(|p| ((simplex_noise_2d(*p) * 100.) as i32, p))
                .collect::<BTreeMap<i32, &Vec2>>();

            for _ in 0..enemy_count {
                if let Some(pos) = pos_noise.pop_last() {
                    let mut weight = rand.u32(0..=weight_count) as i32;

                    for enemy in spawnable_enemies.iter() {
                        weight -= enemy.0 as i32;
                        if weight > 0 {
                            continue;
                        }

                        if let Some(enemy_data) = enemy_manifest.get_by_name(enemy.1.clone()) {
                            floor.enemies.push(
                                commands
                                    .spawn(EnemyBundle::new(
                                        enemy_data,
                                        pos.1.extend(world_z::ENEMY),
                                    ))
                                    //.insert(Idle)
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
    mut event: MessageReader<EnemyKilledMessage>,
    mut level: ResMut<FloorResource>,
    mut portal_writer: MessageWriter<FloorClearedMessage>,
    mut commands: Commands,
) {
    for killed in event.read() {
        level.enemies.retain(|e| *e != killed.0);

        //        commands.entity(killed.0).despawn_recursive();
        if let Ok(mut ec) = commands.get_entity(killed.0) {
            ec.despawn();
        }

        if level.enemies.is_empty() {
            portal_writer.write(FloorClearedMessage);
        }
    }
}
