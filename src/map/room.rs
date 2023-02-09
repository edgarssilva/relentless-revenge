use bevy::prelude::IVec2;

#[derive(Clone, Debug, PartialEq)]
pub struct Room {
    pub pos: IVec2,
    pub radius: i32,
}

impl Room {
    pub fn new(pos: IVec2, radius: i32) -> Self {
        Room { pos, radius }
    }
}
