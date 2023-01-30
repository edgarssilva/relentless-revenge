use bevy::prelude::UVec2;

#[derive(Clone, Debug, PartialEq)]
pub struct Room {
    pub pos: UVec2,
    pub radius: u32,
}

impl Room {
    pub fn new(pos: UVec2, radius: u32) -> Self {
        Room { pos, radius }
    }
}
