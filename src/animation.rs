use crate::{direction::Direction, state::State};
use bevy::{
    prelude::{
        App, Commands, Component, Entity, Plugin, Query, Res, TextureAtlasSprite, Time, Timer,
        With, Without,
    },
    utils::{Duration, HashMap},
};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animation_spawner)
            .add_system(animation_cycling)
            .add_system(animation_state);
    }
}

#[derive(Component)]
pub struct Animation {
    pub frames: Vec<usize>,
    pub current_frame: usize,
    pub timer: Timer,
}

impl Animation {
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.timer.reset();
    }

    pub fn is_finished(&self) -> bool {
        self.timer.finished() && self.current_frame == self.frames.len() - 1
    }
}

#[derive(Component)]
pub struct AnimationState {
    pub animations: HashMap<State, HashMap<Direction, Vec<usize>>>,
    pub duration: u64, //Milliseconds
    pub repeat: bool,
    previous_state: State,
    previous_direction: Direction,
}

impl AnimationState {
    pub fn new(
        animations: HashMap<State, HashMap<Direction, Vec<usize>>>,
        duration: u64,
        repeat: bool,
    ) -> Self {
        Self {
            animations,
            duration,
            repeat,
            previous_state: State::IDLE,
            previous_direction: Direction::NORTH,
        }
    }
}

pub fn animation_spawner(
    query: Query<Entity, (With<AnimationState>, Without<Animation>)>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Animation {
            frames: vec![0],
            current_frame: 0,
            timer: Timer::default(),
        });
    }
}

pub fn animation_state(
    mut query: Query<(&mut Animation, &mut AnimationState, &State, &Direction)>,
) {
    for (mut animation, mut animation_state, state, direction) in query.iter_mut() {
        if !animation_state.previous_state.equals(*state)
            || !animation_state.previous_direction.equals(direction)
        {
            if let Some(directions) = animation_state.animations.get(state) {
                if let Some(frames) = directions.get(direction) {
                    animation.frames = frames.clone();
                    animation
                        .timer
                        .set_duration(Duration::from_millis(animation_state.duration));
                    animation.timer.set_repeating(animation_state.repeat);
                    animation.reset();

                    animation_state.previous_state.set(*state);
                    animation_state.previous_direction.set(*direction);
                }
            }
        }
    }
}

pub fn animation_cycling(
    mut query: Query<(&mut Animation, &mut TextureAtlasSprite)>,
    time: Res<Time>,
) {
    for (mut animation, mut atlas) in query.iter_mut() {
        if animation.is_finished() {
            continue;
        }

        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            animation.current_frame += 1;
            animation.timer.reset();
        }

        if animation.current_frame >= animation.frames.len() {
            if animation.timer.repeating() {
                animation.current_frame = 0;
            } else {
                animation.current_frame = animation.frames.len() - 1;
            }
        }

        atlas.index = animation.frames[animation.current_frame];
    }
}
