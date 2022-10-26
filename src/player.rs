use bevy::{
    prelude::{
        default, AssetServer, Assets, Bundle, Component, KeyCode, Res, ResMut, Transform, Vec2,
        Vec3,
    },
    sprite::{SpriteSheetBundle, TextureAtlas},
    utils::HashMap,
};
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, RigidBody,
};
use leafwing_input_manager::{
    prelude::{ActionState, InputMap},
    InputManagerBundle,
};

use crate::{
    animation::AnimationState, attack::Damageable, collision::BodyLayers, controller::Controlled,
    movement::direction::Direction, state::State, stats::Stats, PLAYER_Z,
};

use leafwing_input_manager::Actionlike;

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    #[bundle]
    sprite_bundle: SpriteSheetBundle,
    collider: Collider,
    controlled: Controlled,
    rigid_body: RigidBody,
    animation_state: AnimationState,
    collision_events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    direction: Direction,
    state: State,
    stats: Stats,
    damageable: Damageable,
    #[bundle]
    input: InputManagerBundle<PlayerActions>,
}

impl PlayerBundle {
    pub fn new(
        asset_server: Res<AssetServer>,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    ) -> Self {
        //Player Creation
        let player_size = Vec2::new(64., 64.);

        //Load the textures
        let texture_handle = asset_server.load("tiny_hero.png");
        let texture_atlas = TextureAtlas::from_grid(texture_handle, player_size, 8, 8);
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        let player_size = player_size / 3.75;

        let mut player_animations = HashMap::new();

        let mut idle_animations = HashMap::new();
        idle_animations.insert(Direction::SOUTH, (16..19).collect());
        idle_animations.insert(Direction::NORTH, (28..31).collect());
        idle_animations.insert(Direction::EAST, (24..27).collect());
        idle_animations.insert(Direction::WEST, (20..23).collect());

        let mut walk_animations = HashMap::new();
        walk_animations.insert(Direction::SOUTH, (48..51).collect());
        walk_animations.insert(Direction::NORTH, (60..63).collect());
        walk_animations.insert(Direction::EAST, (56..59).collect());
        walk_animations.insert(Direction::WEST, (52..55).collect());

        let mut attack_animations = HashMap::new();
        attack_animations.insert(Direction::SOUTH, (0..3).collect());
        attack_animations.insert(Direction::NORTH, (13..16).collect());
        attack_animations.insert(Direction::EAST, (8..12).collect());
        attack_animations.insert(Direction::WEST, (4..7).collect());

        let mut dash_animations = HashMap::new();
        dash_animations.insert(Direction::SOUTH, (32..35).collect());
        dash_animations.insert(Direction::NORTH, (45..48).collect());
        dash_animations.insert(Direction::EAST, (40..44).collect());
        dash_animations.insert(Direction::WEST, (36..39).collect());

        player_animations.insert(State::IDLE, idle_animations);
        player_animations.insert(State::WALKING, walk_animations);
        player_animations.insert(State::ATTACKING, attack_animations);
        player_animations.insert(State::DASHING, dash_animations);

        PlayerBundle {
            player: Player,
            sprite_bundle: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                transform: Transform {
                    translation: Vec3::new(0., 0., PLAYER_Z),
                    scale: Vec3::new(1.25, 1.25, 1.),
                    ..default()
                },
                ..default()
            },
            controlled: Controlled { move_to: None },
            collider: Collider::cuboid(player_size.x / 2., player_size.y / 2.),
            rigid_body: RigidBody::KinematicPositionBased,
            animation_state: AnimationState::new(player_animations, 200, true),
            collision_events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::all(),
            collision_groups: CollisionGroups::new(
                BodyLayers::PLAYER,
                BodyLayers::XP_LAYER | BodyLayers::ENEMY_ATTACK,
            ),
            direction: Direction::SOUTH,
            state: State::IDLE,
            stats: Stats::new(100, 20, 75, 0.5, 0),
            damageable: Damageable,
            input: InputManagerBundle::<PlayerActions> {
                action_state: ActionState::default(),
                input_map: Self::default_keybindings(),
            },
        }
    }

    fn default_keybindings() -> InputMap<PlayerActions> {
        //TODO: Check best keybindings
        let mut input_map = InputMap::default();

        use PlayerActions::*;
        input_map.insert(KeyCode::W, MoveUp);
        input_map.insert(KeyCode::S, MoveDown);
        input_map.insert(KeyCode::A, MoveLeft);
        input_map.insert(KeyCode::D, MoveRight);

        input_map.insert(KeyCode::K, Attack);
        input_map.insert(KeyCode::Space, Dash);

        input_map
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
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
