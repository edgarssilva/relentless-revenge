pub mod direction;
pub mod easing;
pub mod movement;

use bevy::prelude::Vec2;

pub const ISO_VEC: Vec2 = Vec2::new(1.0, 1.0);

pub fn iso_mul(v: Vec2) -> Vec2 {
    Vec2::new(v.x * ISO_VEC.x - v.y * ISO_VEC.y, v.x * ISO_VEC.y + v.y * ISO_VEC.x)
}
