use crate::map::room::Room;
use bevy::input::Input;
use bevy::math::Vec2;
use bevy::prelude::{AssetServer, Commands, GlobalTransform, KeyCode, Res, Transform};
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
        MapSize(1, 1),
        ChunkSize(256, 256),
        TileSize(32.0, 32.0),
        TextureSize(128.0, 128.0),
    );

    layer_settings.grid_size = Vec2::new(32.0, 32.0 / 2.0);
    layer_settings.mesh_type = TilemapMeshType::Isometric(IsoType::Diamond);

    let (mut layer_builder, _) =
        LayerBuilder::<TileBundle>::new(&mut commands, layer_settings, 0u16, 0u16);

    layer_builder.set_all(TileBundle::default());

    let layer_entity = map_query.build_layer(&mut commands, layer_builder, texture_handle);

    map.add_layer(&mut commands, 0u16, layer_entity);

    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(0.0, 8.0 * 32.0, 0.0))
        .insert(GlobalTransform::default());
}

pub fn remake_map(mut commands: Commands, keys: Res<Input<KeyCode>>, mut map_query: MapQuery) {
    if keys.just_released(KeyCode::LControl) {
        map_query.despawn_layer_tiles(&mut commands, 0u16, 0u16);
        build_map(&mut map_query, &mut commands);
    }
}

fn build_map(map_query: &mut MapQuery, commands: &mut Commands) {
    for room in generate_rooms() {
        for x in room.x - room.radius..room.x + room.radius + 1 {
            for y in room.y - room.radius..room.y + room.radius + 1 {
                if Vec2::new(x as f32, y as f32).distance(Vec2::new(room.x as f32, room.y as f32))
                    > room.radius as f32
                {
                    continue;
                }

                let mut tile = Tile {
                    texture_index: 2,
                    ..Default::default()
                };

                if room.x == x && room.y == y {
                    tile.texture_index = 0;
                }

                let tile_pos = TilePos(x as u32, y as u32);

                let _ = map_query.set_tile(commands, tile_pos, tile, 0u16, 0u16);
                map_query.notify_chunk_for_tile(tile_pos, 0u16, 0u16);
            }
        }
    }
}

fn generate_rooms() -> Vec<Room> {
    let mut rng = rand::thread_rng();

    let mut rooms = Vec::<Room>::new();

    let num_rooms = rng.gen_range(5..7);
    let room_min_radius = 3;
    let room_max_radius = 6;

    let mut old_room = Room::new(128, 128, rng.gen_range(room_min_radius..room_max_radius));

    let main_direction = rng.gen_range(0..360) as f32;
    let angle_rande = 130.;

    while rooms.len() < num_rooms {
        let radius = rng.gen_range(room_min_radius..room_max_radius);
        let direction = main_direction + rng.gen_range(-angle_rande..angle_rande);
        let bridge_length = rng.gen_range(1..4);
        let distance = old_room.radius + bridge_length + radius;

        let x = old_room.x + (distance as f32 * direction.to_radians().cos()).round() as i32;
        let y = old_room.y + (distance as f32 * direction.to_radians().sin()).round() as i32;

        old_room = Room::new(x, y, radius);
        rooms.push(old_room);
    }
    rooms
}
