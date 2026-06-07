use bevy::asset::Assets;
use bevy::color::Color;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::math::primitives::Capsule3d;
use bevy::math::Quat;
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::MouseButton;
use bevy::prelude::{Component, KeyCode, Transform};
use bevy::reflect::Reflect;
use leafwing_input_manager::prelude::InputMap;

use crate::controller::Controlled;
use crate::layers::world_z;
use crate::manifest::player::PlayerManifest;
use crate::sorting::{self, YSort};
use crate::Progression;
use crate::{
    attack::Damageable,
    movement::direction::Direction,
    state::State,
    stats::{Cooldown, Damage, Health, MovementSpeed, StatsBundle, XP},
};

use crate::stats::{Level, Revenge};
use leafwing_input_manager::Actionlike;

#[derive(Component)]
pub struct Player;

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    manifest: Res<PlayerManifest>,
) {
    let data = &manifest.player_data;

    commands.spawn((
        Player,
        Mesh3d(meshes.add(Capsule3d::new(5.0, 5.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.7, 0.2), // Bright green placeholder
            ..Default::default()
        })),
        Transform::from_xyz(80., 80., world_z::PLAYER)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        Controlled::default(),
        Direction::SOUTH,
        State::Idle,
        StatsBundle {
            health: Health::new(data.health),
            damage: Damage::new(data.damage),
            speed: MovementSpeed::new(data.speed),
            xp: XP::new(data.xp),
            cooldown: Cooldown::new(data.cooldown),
        },
        Progression::new(data.base_xp, data.xp_multiplier),
        Level::default(),
        Revenge {
            amount: 0.,
            decay: 4.5,
            active_decay: 8.,
            active: false,
            total: 75.,
        },
        Damageable,
        default_keybindings(),
        YSort(sorting::ENTITIES_LAYER),
        //Shadow,
    ));
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerActions {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Attack,
    Dash,
}

impl PlayerActions {
    pub const DIRECTIONS: [Self; 4] = [
        Self::MoveUp,
        Self::MoveDown,
        Self::MoveLeft,
        Self::MoveRight,
    ];

    pub fn direction(&self) -> Option<Direction> {
        match self {
            PlayerActions::MoveUp => Some(Direction::NORTH),
            PlayerActions::MoveDown => Some(Direction::SOUTH),
            PlayerActions::MoveLeft => Some(Direction::WEST),
            PlayerActions::MoveRight => Some(Direction::EAST),
            _ => None,
        }
    }
}

fn default_keybindings() -> InputMap<PlayerActions> {
    //TODO: Check best keybindings
    use PlayerActions::*;
    let mut input_map = InputMap::default();

    input_map
        .insert(PlayerActions::MoveUp, KeyCode::KeyW)
        .insert(MoveDown, KeyCode::KeyS)
        .insert(MoveLeft, KeyCode::KeyA)
        .insert(MoveRight, KeyCode::KeyD)
        .insert(Attack, KeyCode::KeyJ)
        .insert(Attack, MouseButton::Left)
        .insert(Dash, KeyCode::Space);

    input_map
}
