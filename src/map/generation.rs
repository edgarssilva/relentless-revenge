use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::floor::{FloorClearedMessage, GenerateFloorMessage, SpawnFloorEntitiesMessage};
use crate::layers::world_z;

use crate::map::map::Map;
use crate::map::map::Tile;
use crate::map::map::TileVariant;

pub const MAP_HEIGHT: f32 = 0.01;
pub const TILE_SIZE: f32 = 10.0;

fn tile_pos_to_world_2d(tile_pos: IVec2) -> Vec2 {
    Vec2::new(
        tile_pos.x as f32 * TILE_SIZE,
        tile_pos.y as f32 * TILE_SIZE,
    )
}

#[derive(Resource)]
pub struct MapResource {
    pub blueprint: Map,
    pub tiles: HashMap<IVec2, Tile>,
}

impl MapResource {
    pub fn new(blueprint: Map) -> Self {
        MapResource {
            tiles: blueprint.generate_tiles(),
            blueprint,
        }
    }

    pub fn get_tile(&self, pos: IVec2) -> Option<&Tile> {
        self.tiles.get(&pos)
    }

    pub fn tile_to_world_2d(&self, tile: &Tile) -> Vec2 {
        tile_pos_to_world_2d(tile.pos)
    }

    pub fn tile_to_world(&self, tile: &Tile) -> Vec3 {
        self.tile_to_world_2d(tile).extend(world_z::MAP)
    }

    pub fn world_to_tile(&self, world_pos: &Vec2) -> IVec2 {
        (world_pos / TILE_SIZE).round().as_ivec2() * TILE_SIZE as i32
    }

    pub fn get_tile_at_world_pos(&self, world_pos: &Vec2) -> Option<&Tile> {
        let tile_coords = self.world_to_tile(world_pos);
        self.get_tile(tile_coords)
    }

    pub fn get_aprox_tile(&self, world_pos_2d: &Vec2) -> Option<&Tile> {
        self.get_tile(self.world_to_tile(world_pos_2d))
    }
}

#[derive(Component)]
pub struct TileMarker;

#[derive(Component)]
pub struct LevelPortalTile;

#[derive(Component)]
pub struct PlayerSpawnTile;

pub fn build_3d_map_system(
    event: MessageReader<GenerateFloorMessage>,
    map_resource: Res<MapResource>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut event_writer: MessageWriter<SpawnFloorEntitiesMessage>,
) {
    if event.is_empty() {
        return;
    }

    let player_pos = map_resource
        .blueprint
        .rooms
        .first()
        .map(|room| tile_pos_to_world_2d(room.pos))
        .unwrap_or(Vec2::ZERO);
    let mut spawnable_pos = Vec::new();
    let portal_pos = map_resource
        .blueprint
        .rooms
        .last()
        .map(|room| tile_pos_to_world_2d(room.pos))
        .unwrap_or(Vec2::ZERO);

    let standard_mesh = meshes.add(Cuboid::new(TILE_SIZE, TILE_SIZE, MAP_HEIGHT));
    let standard_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.4, 0.4),
        ..Default::default()
    });
    let accented_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.3, 0.3),
        ..Default::default()
    });

    for tile in map_resource.tiles.values() {
        let world_pos_2d = map_resource.tile_to_world_2d(tile);
        let world_pos_3d = world_pos_2d.extend(world_z::MAP);

        let active_material = match tile.variant {
            TileVariant::Standard => standard_material.clone(),
            TileVariant::Accented => accented_material.clone(),
        };

        let mut ec = commands.spawn((
            TileMarker,
            Mesh3d(standard_mesh.clone()),
            MeshMaterial3d(active_material),
            Transform::from_translation(world_pos_3d),
        ));

        if tile.spawnable {
            spawnable_pos.push(world_pos_2d);
        }

        if tile.is_center {
            if tile.firt_room {
                ec.insert(PlayerSpawnTile);
            } else if tile.last_room {
                ec.insert(LevelPortalTile);
            }
        }
    }

    event_writer.write(SpawnFloorEntitiesMessage {
        spawnable_pos,
        player_pos,
        portal_pos,
    });
}

pub fn open_level_portal(mut events: MessageReader<FloorClearedMessage>) {
    if !events.is_empty() {
        //TODO: Set floor tile to red
        events.clear();
    }
}
