use bevy::{
    prelude::{default, Bundle, Component, KeyCode, Transform, Vec3},
    sprite::SpriteSheetBundle,
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
    animation::AnimationState,
    attack::Damageable,
    collision::BodyLayers,
    controller::Controlled,
    movement::direction::Direction,
    state::State,
    stats::{Cooldown, Damage, Health, MovementSpeed, StatsBundle, XP},
    PLAYER_Z,
};

use leafwing_input_manager::Actionlike;
use crate::metadata::PlayerMeta;

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
    #[bundle]
    stats: StatsBundle,
    damageable: Damageable,
    #[bundle]
    input: InputManagerBundle<PlayerActions>,
}

impl PlayerBundle {
    pub fn new(meta: &PlayerMeta) -> Self {
        let mut player_animations = HashMap::new();

        let mut idle_animations = HashMap::new();
        idle_animations.insert(Direction::SOUTH, (0..7).collect());
        idle_animations.insert(Direction::EAST, (8..15).collect());
        idle_animations.insert(Direction::NORTH, (16..23).collect());
        idle_animations.insert(Direction::WEST, (24..31).collect());

        let mut walk_animations = HashMap::new();
        walk_animations.insert(Direction::SOUTH, (32..39).collect());
        walk_animations.insert(Direction::EAST, (40..47).collect());
        walk_animations.insert(Direction::NORTH, (48..55).collect());
        walk_animations.insert(Direction::WEST, (56..63).collect());

        let mut attack_animations = HashMap::new();
        attack_animations.insert(Direction::SOUTH, (96..101).collect());
        attack_animations.insert(Direction::EAST, (104..109).collect());
        attack_animations.insert(Direction::NORTH, (112..117).collect());
        attack_animations.insert(Direction::WEST, (120..125).collect());

        let mut dash_animations = HashMap::new();
        dash_animations.insert(Direction::SOUTH, (64..71).collect());
        dash_animations.insert(Direction::EAST, (72..79).collect());
        dash_animations.insert(Direction::NORTH, (80..87).collect());
        dash_animations.insert(Direction::WEST, (88..95).collect());

        player_animations.insert(State::IDLE, idle_animations);
        player_animations.insert(State::WALKING, walk_animations);
        player_animations.insert(State::ATTACKING, attack_animations);
        player_animations.insert(State::DASHING, dash_animations);

        PlayerBundle {
            player: Player,
            sprite_bundle: SpriteSheetBundle {
                texture_atlas: meta.texture.atlas_handle.clone(),
                transform: Transform {
                    translation: Vec3::new(0., 0., PLAYER_Z),
                    scale: Vec3::new(0.75, 0.75, 0.75),
                    ..default()
                },
                ..default()
            },
            controlled: Controlled { move_to: None },
            collider: Collider::cuboid(meta.hitbox.x / 2., meta.hitbox.y / 2.),
            rigid_body: RigidBody::KinematicPositionBased,
            animation_state: AnimationState::new(player_animations, meta.texture.duration, true),
            collision_events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::all(),
            collision_groups: CollisionGroups::new(
                BodyLayers::PLAYER,
                BodyLayers::XP_LAYER | BodyLayers::ENEMY_ATTACK,
            ),
            direction: Direction::SOUTH,
            state: State::IDLE,
            stats: StatsBundle {
                health: Health::new(meta.health),
                damage: Damage::new(meta.damage),
                speed: MovementSpeed::new(meta.speed),
                xp: XP::new(meta.xp),
                cooldown: Cooldown::new(meta.cooldown),
            },
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
