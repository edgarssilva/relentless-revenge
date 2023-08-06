use bevy::{math::Vec2, prelude::Component};

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Direction {
    NORTH,
    SOUTH,
    WEST,
    EAST,
}

impl Direction {
    pub fn vec(&self) -> Vec2 {
        match *self {
            Self::NORTH => Vec2::Y,
            Self::SOUTH => -Vec2::Y,
            Self::WEST => -Vec2::X,
            Self::EAST => Vec2::X,
        }
    }

    pub fn from_vec2(vec: Vec2) -> Option<Self> {
        if vec.x > 0.0 {
            Some(Self::EAST)
        } else if vec.x < 0.0 {
            Some(Self::WEST)
        } else if vec.y > 0.0 {
            Some(Self::NORTH)
        } else if vec.y < 0.0 {
            Some(Self::SOUTH)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn values() -> [Self; 4] {
        [Self::NORTH, Self::SOUTH, Self::WEST, Self::EAST]
    }

    pub fn set(&mut self, direction: Direction) {
        *self = direction;
    }

    pub fn equals(&self, other: &Self) -> bool {
        self == other
    }
}
