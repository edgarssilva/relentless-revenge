use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::floor::{FloorClearedMessage, GenerateFloorMessage, SpawnFloorEntitiesMessage};
use crate::layers::world_z;

use crate::map::map::Map;
use crate::map::map::Tile;
use crate::map::map::TileVariant;

pub const MAP_HEIGHT: f32 = 0.01;
pub const TILE_SIZE: f32 = 10.0;

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

    //TODO: Check if this rounding is enough, or check within bounds of tile
    pub fn get_aprox_tile(&self, pos: Vec2) -> Option<&Tile> {
        self.get_tile(pos.round().as_ivec2())
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

    let mut player_pos = Vec2::ZERO;
    let mut spawnable_pos = Vec::new();
    let mut portal_pos = Vec2::ZERO;

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
        let world_pos_3d = Vec3::new(
            tile.pos.x as f32 * TILE_SIZE,
            tile.pos.y as f32 * TILE_SIZE,
            world_z::MAP,
        );
        let world_pos_2d = world_pos_3d.xy(); // Quick .xy() swizzle shortcut

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
                player_pos = world_pos_2d;
                ec.insert(PlayerSpawnTile);
            } else if tile.last_room {
                portal_pos = world_pos_2d;
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
