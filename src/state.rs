use bevy::prelude::Component;

#[derive(Component)]
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
}
