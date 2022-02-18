use bevy::math::IVec2;

#[derive(Clone, Debug)]
pub struct Bridge {
    pub pos: Vec<IVec2>,
}

impl Bridge {
    pub fn new(pos: Vec<IVec2>) -> Self {
        Bridge { pos }
    }
}
