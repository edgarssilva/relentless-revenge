use std::usize;

use bevy::prelude::MouseButton;
use bevy::reflect::Reflect;
use bevy::sprite::{SpriteBundle, TextureAtlas};
use bevy::{
    prelude::{default, Bundle, Component, KeyCode, Transform, Vec3},
    utils::HashMap,
};
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, RigidBody,
};
use bevy_spritesheet_animation::prelude::SpritesheetAnimation;
use leafwing_input_manager::{
    prelude::{ActionState, InputMap},
    InputManagerBundle,
};

use crate::animation::{Animations, DirectionalAnimations};
use crate::effects::Shadow;
use crate::manifest::player::PlayerData;
use crate::sorting::{self, FeetOffset, YSort};
use crate::Progression;
use crate::{
    attack::Damageable,
    collision::BodyLayers,
    controller::Controlled,
    movement::direction::Direction,
    state::State,
    stats::{Cooldown, Damage, Health, MovementSpeed, StatsBundle, XP},
    PLAYER_Z,
};

use crate::stats::{Level, Revenge};
use leafwing_input_manager::Actionlike;

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    sprite: SpriteBundle,
    atlas: TextureAtlas,
    collider: Collider,
    controlled: Controlled,
    rigid_body: RigidBody,
    animations: DirectionalAnimations,
    animation: SpritesheetAnimation,
    collision_events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    direction: Direction,
    state: State,
    level: Level,
    revenge: Revenge,
    stats: StatsBundle,
    progression: Progression,
    damageable: Damageable,
    input: InputManagerBundle<PlayerActions>,
    ysort: YSort,
    feet_offset: FeetOffset,
    shadow: Shadow,
}

impl PlayerBundle {
    pub fn new(data: &PlayerData) -> Self {
        /* let mut player_animations = HashMap::new();

                let mut idle_animations = HashMap::new();
                idle_animations.insert(Direction::SOUTH, (0..7).collect());
                idle_animations.insert(Direction::EAST, (10..17).collect());
                idle_animations.insert(Direction::NORTH, (20..27).collect());
                idle_animations.insert(Direction::WEST, (30..37).collect());

                let mut walk_animations = HashMap::new();
                walk_animations.insert(Direction::SOUTH, (40..47).collect());
                walk_animations.insert(Direction::EAST, (50..57).collect());
                walk_animations.insert(Direction::NORTH, (60..67).collect());
                walk_animations.insert(Direction::WEST, (70..77).collect());

                let mut dash_animations = HashMap::new();
                dash_animations.insert(Direction::SOUTH, (80..87).collect());
                dash_animations.insert(Direction::EAST, (90..97).collect());
                dash_animations.insert(Direction::NORTH, (100..107).collect());
                dash_animations.insert(Direction::WEST, (110..117).collect());

                let mut attack_animations = HashMap::new();
                attack_animations.insert(Direction::SOUTH, (120..125).collect());
                attack_animations.insert(Direction::EAST, (130..135).collect());
                attack_animations.insert(Direction::NORTH, (140..145).collect());
                attack_animations.insert(Direction::WEST, (150..155).collect());

                let mut attack_animations_1 = HashMap::new();
                attack_animations_1.insert(Direction::SOUTH, (160..165).collect());
                attack_animations_1.insert(Direction::EAST, (170..175).collect());
                attack_animations_1.insert(Direction::NORTH, (180..185).collect());
                attack_animations_1.insert(Direction::WEST, (190..195).collect());

                let mut attack_animations_2 = HashMap::new();
                attack_animations_2.insert(Direction::SOUTH, (200..209).collect());
                attack_animations_2.insert(Direction::EAST, (210..219).collect());
                attack_animations_2.insert(Direction::NORTH, (220..229).collect());
                attack_animations_2.insert(Direction::WEST, (230..239).collect());

                player_animations.insert(State::Idle, idle_animations);
                player_animations.insert(State::Walking, walk_animations);
                player_animations.insert(State::Attacking(0), attack_animations);
                player_animations.insert(State::Attacking(1), attack_animations_1);
                player_animations.insert(State::Attacking(2), attack_animations_2);
                player_animations.insert(State::Dashing, dash_animations);
        */

        let feet_offset = data.feet_offset.unwrap_or_default();

        PlayerBundle {
            player: Player,
            atlas: TextureAtlas {
                layout: data.atlas.clone(),
                index: 0,
            },
            sprite: SpriteBundle {
                texture: data.texture.clone(),
                transform: Transform {
                    translation: Vec3::new(0., 0., PLAYER_Z),
                    scale: Vec3::new(0.75, 0.75, 0.75),
                    ..default()
                },
                ..default()
            },
            controlled: Controlled { move_to: None },
            collider: Collider::cuboid(data.hitbox.x / 2., data.hitbox.y / 2.),
            rigid_body: RigidBody::KinematicPositionBased,
            //animation_state: AnimationState::new(player_animations, data.frame_duration, true),
            animations: data.animations.clone(),
            //TODO: Fix this to have a default animation or be insert later on
            animation: SpritesheetAnimation::from_id(
                *data
                    .animations
                    .0
                    .get(&State::Idle)
                    .expect("No IDLE animation found for player")
                    .get(&Direction::SOUTH)
                    .expect("No SOUTH animation found for player"),
            ),
            collision_events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::all(),
            collision_groups: CollisionGroups::new(
                BodyLayers::PLAYER,
                BodyLayers::XP_LAYER | BodyLayers::ENEMY_ATTACK,
            ),
            direction: Direction::SOUTH,
            state: State::Idle,
            stats: StatsBundle {
                health: Health::new(data.health),
                damage: Damage::new(data.damage),
                speed: MovementSpeed::new(data.speed),
                xp: XP::new(data.xp),
                cooldown: Cooldown::new(data.cooldown),
            },
            progression: Progression::new(data.base_xp, data.xp_multiplier),
            level: Level::default(),
            revenge: Revenge {
                amount: 0.,
                decay: 4.5,
                active_decay: 8.,
                active: false,
                total: 75.,
            },
            damageable: Damageable,
            input: InputManagerBundle::<PlayerActions> {
                action_state: ActionState::default(),
                input_map: Self::default_keybindings(),
            },
            ysort: YSort(sorting::ENTITIES_LAYER),
            feet_offset: FeetOffset(feet_offset),
            shadow: Shadow,
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
