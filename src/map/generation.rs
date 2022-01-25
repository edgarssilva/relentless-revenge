use bevy::math::Vec2;
use bevy::prelude::{AssetServer, Commands, GlobalTransform, Res, Transform};
use bevy_ecs_tilemap::{
    ChunkSize, IsoType, LayerBuilder, LayerSettings, Map, MapQuery, MapSize, TextureSize, Tile,
    TileBundle, TilePos, TileSize, TilemapMeshType,
};
use rand;
use rand::prelude::*;

pub fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let texture_handle = asset_server.load("tileset.png");

    let mut layer_settings = LayerSettings::new(
        MapSize(2, 2),
        ChunkSize(16, 16),
        TileSize(32.0, 32.0),
        TextureSize(128.0, 128.0),
    );

    layer_settings.grid_size = Vec2::new(32.0, 32.0 / 2.0);
    layer_settings.mesh_type = TilemapMeshType::Isometric(IsoType::Diamond);

    let (mut layer_builder, _) =
        LayerBuilder::<TileBundle>::new(&mut commands, layer_settings, 0u16, 0u16);

    // layer_builder.set_all(TileBundle::default());

    let mut rng = rand::thread_rng();

    let max_rooms = 20;
    let min_room_width = 6;
    let max_room_width = 12;
    let min_room_height = 6;
    let max_room_height = 12;
    let map_width = layer_settings.map_size.0 * layer_settings.chunk_size.0;
    let map_height = layer_settings.map_size.1 * layer_settings.chunk_size.1;
    let mut chunks = Vec::<Chunk>::new();

    for _ in 0..max_rooms {
        // place up to max_rooms - if it collides with another, it won't get placed
        let mut x = rng.gen_range(0..map_width);
        let mut y = rng.gen_range(0..map_height);

        let width = rng.gen_range(min_room_width..max_room_width);
        let height = rng.gen_range(min_room_height..max_room_height);

        // if it's off the board, shift it back on again
        if x + width > map_width {
            x = map_width - width;
        }

        if y + height > map_height {
            y = map_height - height;
        }

        let mut collides = false;
        let chunk = Chunk::new(x, y, width, height);

        // check all other chunks we've placed to see if this one
        // collides with them
        for other_chunk in &chunks {
            if chunk.intersects(&other_chunk) {
                collides = true;
                break;
            }
        }

        // if the new chunk doesn't collide, add it to the level
        if !collides {
            for row in 0..chunk.height {
                for col in 0..chunk.width {
                    let y = chunk.y + row;
                    let x = chunk.x + col;

                    let _ = layer_builder.set_tile(
                        TilePos(x, y),
                        TileBundle {
                            tile: Tile {
                                texture_index: 0,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    );
                }
            }

            chunks.push(chunk);
        }
    }

    let layer_entity = map_query.build_layer(&mut commands, layer_builder, texture_handle);

    map.add_layer(&mut commands, 0u16, layer_entity);

    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(0.0, 8.0 * 32.0, 0.0))
        .insert(GlobalTransform::default());
}

#[derive(Clone, Copy, Debug)]
struct Chunk {
    pub x: u32,
    pub y: u32,
    pub x2: u32,
    pub y2: u32,
    pub width: u32,
    pub height: u32,
    pub center: (u32, u32),
}

impl Chunk {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Chunk {
            x,
            y,
            x2: x + width,
            y2: y + height,
            width,
            height,
            center: (x + (width / 2), y + (height / 2)),
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.x <= other.x2 && self.x2 >= other.x && self.y <= other.y2 && self.y2 >= other.y
    }
}
