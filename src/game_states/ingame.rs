use bevy::camera::ScalingMode;
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::EguiPrimaryContextPass;
use bevy_persistent::prelude::*;
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::attack::{
    attack_phase_system, attack_spawner_observer, charge_phase_system, recover_phase_system,
};
use crate::controller::combo_system;
use crate::effects::spawn_shadows;
use crate::game_states::ingame::InGameSet::{Normal, Post};
use crate::manifest::floor::DomainData;
use crate::manifest::player::PlayerManifest;
use crate::map::debug::draw_tile_grid_gizmos;
use crate::map::generation::{build_3d_map_system, MapResource};
use crate::map::map::{generate_map, Map};
use crate::player::spawn_player;
//use crate::sorting::ysort;
use crate::stats::{level_up, revenge_mode};
use crate::ui::boss::{draw_boss_health, draw_domain_name};
use crate::ui::player::{draw_hud, draw_revenge_bar, draw_xp_bar};
use crate::{
    animation::AnimationPlugin,
    attack::{lifetimes, projectile_break, tick_cooldown},
    collision::CollisionPlugin,
    controller::{attack_ability, dash_ability, finish_dash, move_player},
    enemy::EnemyBehaviourPlugin,
    floor::FloorPlugin,
    helper::{follow_player_camera, helper_camera_controller, shake_system, IsometricCameraFollow},
    movement::movement::MovementPlugin,
    player::PlayerActions,
    statistics::{auto_save, statistics, Statistics},
    stats::{drop_xp_system, trigger_enemy_death},
    GameState,
};

pub struct InGamePlugin;

//TODO: Refactor this to something meaningful
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum InGameSet {
    Normal,
    Post,
}

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .add_plugins(InputManagerPlugin::<PlayerActions>::default())
            .add_plugins(CollisionPlugin)
            .add_plugins(AnimationPlugin)
            .add_plugins(EnemyBehaviourPlugin)
            .add_plugins(FloorPlugin)
            .add_plugins(MovementPlugin)
            //.add_plugins(FreeCameraPlugin)
            .add_systems(
                Update,
                (auto_save, statistics).run_if(in_state(GameState::InGame)),
            )
            .insert_resource(MapResource::new(generate_map(&DomainData {
                name: "Test domain".to_string(),
                floors: (1, 4),
                rooms: (3, 5),
                room_size: (3, 5),
                boss: "test".to_string(),
                enemies_count: (2, 10),
                enemies: vec![],
            })))
            .add_systems(
                OnEnter(GameState::InGame),
                (setup_game /*setup_map*/, spawn_player).chain(),
            )
            //TODO: Check system ordering and optimize it
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_hud,
                    draw_domain_name,
                    draw_xp_bar,
                    draw_revenge_bar,
                    draw_boss_health,
                )
                    .in_set(Normal)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_observer(attack_spawner_observer)
            .add_systems(
                Update,
                (
                    follow_player_camera,
                    helper_camera_controller,
                    move_player,
                    dash_ability,
                    attack_ability,
                    combo_system,
                    tick_cooldown,
                    shake_system,
                    build_3d_map_system,
                    draw_tile_grid_gizmos,
                    lifetimes,
                    projectile_break,
                    drop_xp_system,
                    level_up,
                    revenge_mode,
                    charge_phase_system,
                    attack_phase_system,
                    recover_phase_system,
                )
                    .in_set(Normal)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (
                    spawn_shadows,
                    finish_dash,
                    trigger_enemy_death,
                    //ysort,
                )
                    .in_set(Post)
                    .after(Normal)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn setup_game(mut commands: Commands) {
    let dir = dirs::data_dir().unwrap().join("relentless_revenge");

    commands.insert_resource(
        Persistent::<Statistics>::builder()
            .name("statistics")
            .format(StorageFormat::Bincode)
            .path(dir.join("statistic.bin"))
            .default(Statistics::default())
            .build()
            .expect("Failed to create persistent statistics"),
    );

    commands.spawn((
        FreeCamera::default(),
        IsometricCameraFollow {
            offset: Vec3::new(10.0, -10.0, 10.0),
            smoothness: 9.0,
        },
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            /*scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 30.0, // Clean zoom level
            },*/
            scale: 0.1,
            near: -1000.0, // Expands the front clipping plane
            far: 1000.0,   // Expands the back clipping plane
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(100.0, -100.0, 100.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Z),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        // Angle it slightly down so it hits your 3D faces cleanly
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
