use bevy::math::Vec3Swizzles;
use bevy::prelude::{Color, Gizmos, IVec2, Query, Res, Result, Vec3, With};
use bevy::transform::components::Transform;

use crate::controller::Controlled;
use crate::layers::world_z;
use crate::map::generation::{MapResource, TILE_SIZE};
use crate::player::Player;

pub fn draw_tile_grid_gizmos(
    map_resource: Res<MapResource>,
    player_query: Query<(&Transform, &Controlled), With<Player>>,
    mut gizmos: Gizmos,
) -> Result {
    let (transform, controlled) = player_query.single()?;

    let current_tile = map_resource.get_aprox_tile(transform.translation.xy());
    let next_tile = controlled
        .move_to
        .and_then(|pos| map_resource.get_aprox_tile(pos));

    for tile in map_resource.tiles.values() {
        let z = world_z::DEBUG_GRID;
        let center = Vec3::new(tile.pos.x as f32, tile.pos.y as f32, z);

        let is_current_tile = current_tile.map_or_else(|| false, |p| p.pos == tile.pos);
        let is_next_tile = next_tile.map_or_else(|| false, |p| p.pos == tile.pos);

        let color = if is_current_tile {
            Color::srgb(1.0, 0.95, 0.1)
        } else if is_next_tile {
            Color::srgb(0.1, 0.95, 1.0)
        } else if tile.walkable {
            Color::srgb(0.0, 0.9, 0.9)
        } else {
            Color::srgb(1.0, 0.2, 0.2)
        };

        draw_tile_outline(&mut gizmos, center, TILE_SIZE, color);

        if is_current_tile {
            draw_tile_cross(&mut gizmos, center, TILE_SIZE, Color::WHITE);
            gizmos.line(
                Vec3::new(center.x, center.y, center.z),
                Vec3::new(center.x, center.y, center.z),
                Color::WHITE,
            );
        }
    }

    Ok(())
}

fn draw_tile_outline(gizmos: &mut Gizmos, center: Vec3, half_size: f32, color: Color) {
    let top_left = Vec3::new(center.x - half_size, center.y + half_size, center.z);
    let top_right = Vec3::new(center.x + half_size, center.y + half_size, center.z);
    let bottom_right = Vec3::new(center.x + half_size, center.y - half_size, center.z);
    let bottom_left = Vec3::new(center.x - half_size, center.y - half_size, center.z);

    gizmos.line(top_left, top_right, color);
    gizmos.line(top_right, bottom_right, color);
    gizmos.line(bottom_right, bottom_left, color);
    gizmos.line(bottom_left, top_left, color);
}

fn draw_tile_cross(gizmos: &mut Gizmos, center: Vec3, half_size: f32, color: Color) {
    let top_left = Vec3::new(center.x - half_size, center.y + half_size, center.z);
    let top_right = Vec3::new(center.x + half_size, center.y + half_size, center.z);
    let bottom_right = Vec3::new(center.x + half_size, center.y - half_size, center.z);
    let bottom_left = Vec3::new(center.x - half_size, center.y - half_size, center.z);

    gizmos.line(top_left, bottom_right, color);
    gizmos.line(top_right, bottom_left, color);
}
