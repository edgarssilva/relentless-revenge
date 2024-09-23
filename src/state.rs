use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum State {
    Idle,
    Walking,
    Attacking(u32), //Index of the attack in a combo
    Dashing,
    _Dying,
}

impl State {
    pub fn set(&mut self, state: State) {
        *self = state;
    }

    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }
}
