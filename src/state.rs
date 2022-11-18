use bevy::prelude::Component;

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum State {
    IDLE,
    WALKING,
    ATTACKING,
    DASHING,
    _DYING,
}

impl State {
    pub fn set(&mut self, state: State) {
        *self = state;
    }

    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }
}
