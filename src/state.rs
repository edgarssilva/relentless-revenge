use bevy::prelude::Component;

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy)]
pub enum State {
    IDLE,
    WALKING,
    ATTACKING,
    DYING,
}

impl State {
    pub fn set(&mut self, state: State) {
        *self = state;
    }

    pub fn equals(&self, other: &Self) -> bool {
        self == other
    }
}
