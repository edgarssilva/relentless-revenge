use bevy::prelude::UVec2;

#[derive(Clone, Debug)]
pub struct Bridge {
    pub pos: Vec<UVec2>,
}

impl Bridge {
    pub fn new(pos: Vec<UVec2>) -> Self {
        Bridge { pos }
    }
}
