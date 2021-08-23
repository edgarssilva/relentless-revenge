use bevy::math::Vec2;

#[derive(PartialEq)]
pub enum Direction {
    NORTH,
    SOUTH,
    WEST,
    EAST,
}

impl Direction {
    pub fn vec(&self) -> Vec2 {
        match self {
            &Self::NORTH => Vec2::Y,
            &Self::SOUTH => -Vec2::Y,
            &Self::WEST => -Vec2::X,
            &Self::EAST => Vec2::X,
        }
    }

    pub fn values() -> [Self; 4] {
        [Self::NORTH, Self::SOUTH, Self::WEST, Self::EAST]
    }
}
