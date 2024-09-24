use std::collections::BTreeMap;
use std::time::Duration;

use bevy::hierarchy::DespawnRecursiveExt;
use bevy::input::ButtonInput;
use bevy::prelude::{
    in_state, Camera, Event, IntoSystemConfigs, Query, Transform, Update, With, Without, World,
};
use bevy::time::Timer;
use bevy::{
    math::Vec2,
    prelude::{
        App, Commands, Entity, EventReader, EventWriter, KeyCode, Plugin, Res, ResMut, Resource,
    },
};
use leafwing_manifest::manifest::Manifest;
use noisy_bevy::simplex_noise_2d;
use turborand::rng::Rng;
use turborand::TurboRand;

use crate::boss::BossBundle;
use crate::enemy::state_machine::Idle;
use crate::manifest::boss::BossManifest;
use crate::manifest::enemy::EnemyManifest;
use crate::manifest::floor::{DomainData, DomainManifest};
use crate::map::generation::open_level_portal;
use crate::map::walkable::travel_through_portal;
use crate::player::Player;
use crate::ui::boss::DomainName;
use crate::{enemy::EnemyBundle, GameState};

#[derive(Default, Resource)]
pub struct FloorResource {
    pub floor: u32,
    pub domain: Option<DomainData>,
    pub enemies: Vec<Entity>,
    pub boss: Option<Entity>,
}

//Floor Generation Events
#[derive(Event)]
pub struct GenerateFloorEvent;

#[derive(Event)]
pub struct SpawnFloorEntitiesEvent {
    pub spawnable_pos: Vec<Vec2>,
    pub player_pos: Vec2,
    pub portal_pos: Vec2,
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
                    new_domain_trigger,
                    move_player,
                    enemy_killed,
                    spawn_enemies,
                    spawn_boss,
                    generate_floor,
                    keymap_generate,
                    open_level_portal,
                    travel_through_portal,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn keymap_generate(
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: EventWriter<TriggerNextFloorEvent>,
) {
    if keys.just_pressed(KeyCode::ControlLeft) {
        writer.send(TriggerNextFloorEvent);
    }
}

fn new_domain_trigger(
    mut commands: Commands,
    mut event: EventReader<GenerateFloorEvent>,
    floor: Res<FloorResource>,
) {
    if event.is_empty() {
        return;
    }

    if let Some(domain) = &floor.domain {
        if floor.floor == domain.floors.0 {
            commands.spawn(DomainName(
                domain.name.clone(),
                Timer::new(Duration::from_secs(3), bevy::time::TimerMode::Once),
            ));
        }
    }

    event.clear();
}

fn generate_floor(
    mut event: EventReader<TriggerNextFloorEvent>,
    mut writer: EventWriter<GenerateFloorEvent>,
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
        writer.send(GenerateFloorEvent);
    }
}

fn move_player(
    mut player_query: Query<&mut Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut event: EventReader<SpawnFloorEntitiesEvent>,
) {
    for e in event.read() {
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

fn spawn_boss(
    mut commands: Commands,
    boss_manifest: Res<BossManifest>,
    mut floor: ResMut<FloorResource>,
    mut event: EventReader<SpawnFloorEntitiesEvent>,
) {
    for e in event.read() {
        if let Some(domain) = &floor.domain {
            if domain.floors.1 != floor.floor {
                return;
            }

            if let Some(boss) = boss_manifest.get_by_name(domain.boss.clone()) {
                floor.boss = Some(
                    commands
                        .spawn(BossBundle::new(boss, e.portal_pos.extend(38.0)))
                        .insert(Idle)
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
    mut event: EventReader<SpawnFloorEntitiesEvent>,
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
                                    .spawn(EnemyBundle::new(enemy_data, pos.1.extend(38.0)))
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
    for killed in event.read() {
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
