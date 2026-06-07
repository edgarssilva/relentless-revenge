use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::{IVec2, Vec2},
};
use turborand::{rng::Rng, TurboRand};

use crate::{
    manifest::floor::DomainData, map::generation::TILE_SIZE, movement::direction::Direction,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Room {
    pub pos: IVec2,
    pub radius: i32,
}

impl Room {
    pub fn new(pos: IVec2, radius: i32) -> Self {
        Room { pos, radius }
    }
}

#[derive(Clone, Debug)]
pub struct Bridge {
    pub pos: Vec<IVec2>,
}

impl Bridge {
    pub fn new(pos: Vec<IVec2>) -> Self {
        Bridge { pos }
    }
}

#[derive(Clone, Debug)]
pub enum TileVariant {
    Standard,
    Accented,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub pos: IVec2,
    pub walkable: bool,
    pub is_center: bool,
    pub empty_neighbors: Vec<Direction>,
    pub variant: TileVariant,
    pub spawnable: bool,
    pub firt_room: bool,
    pub last_room: bool,
}

#[derive(Clone, Debug)]
pub struct Map {
    pub rooms: Vec<Room>,
    pub bridges: Vec<Bridge>,
}

impl Map {
    pub(crate) fn generate_tiles(&self) -> HashMap<IVec2, Tile> {
        let mut tiles = HashMap::<IVec2, Tile>::new();

        for (i, room) in self.rooms.iter().enumerate() {
            for x in -room.radius..=room.radius {
                for y in -room.radius..=room.radius {
                    //Remove corners
                    if x.abs() == room.radius && y.abs() == room.radius {
                        continue;
                    }

                    let pos = room.pos + IVec2::new(x, y);
                    let world_pos = pos * TILE_SIZE as i32;
                    //let walkable = x * x + y * y <= room.radius * room.radius;
                    let is_center = x == 0 && y == 0;

                    tiles.insert(
                        world_pos,
                        Tile {
                            pos,
                            walkable: true,
                            is_center,
                            empty_neighbors: Vec::new(),
                            variant: TileVariant::Standard,
                            spawnable: true,
                            firt_room: i == 0,
                            last_room: i == self.rooms.len() - 1,
                        },
                    );
                }
            }
        }

        for bridge in &self.bridges {
            for pos in &bridge.pos {
                tiles.insert(
                    pos * TILE_SIZE as i32,
                    Tile {
                        pos: pos.clone(),
                        walkable: true,
                        is_center: false,
                        empty_neighbors: Vec::new(),
                        variant: TileVariant::Accented,
                        spawnable: false, //TODO: Should be fine to make the bridges spawnable
                        firt_room: false,
                        last_room: false,
                    },
                );
            }
        }

        let existing_positions: HashSet<IVec2> = tiles.keys().cloned().collect();

        for (pos, tile) in tiles.iter_mut() {
            tile.empty_neighbors = Direction::values()
                .iter()
                .filter(|direction| {
                    let neighbor_pos = *pos + direction.vec().as_ivec2();
                    !existing_positions.contains(&neighbor_pos)
                })
                .cloned() // Copy the Direction enum into the vector
                .collect();

            if !tile.empty_neighbors.is_empty() || tile.is_center {
                tile.spawnable = false;
            }
        }

        tiles
    }
}

pub fn generate_map(domain_data: &DomainData) -> Map {
    let mut rand = Rng::new();
    let mut rooms = Vec::<Room>::new();
    let mut bridges = Vec::<Bridge>::new();

    let num_rooms = rand.u32(domain_data.rooms.0..=domain_data.rooms.1);
    let room_min_radius = domain_data.room_size.0;
    let room_max_radius = domain_data.room_size.1;

    let mut old_room = Room::new(
        IVec2::new(80, 80),
        rand.u32(room_min_radius..=room_max_radius) as i32,
    );

    let angle_range = 180;
    let main_direction = rand.i32(0..360);

    while rooms.len() < num_rooms as usize {
        let radius = rand.u32(room_min_radius..=room_max_radius) as i32;
        let direction = main_direction + rand.i32(-angle_range..angle_range);
        let direction = (direction as f32).to_radians();
        let bridge_length = rand.i32(1..=3);
        let distance = old_room.radius + bridge_length + (radius / 2);

        let new_room = Room::new(
            (Vec2::new(direction.cos(), direction.sin()) * distance as f32)
                .round()
                .as_ivec2()
                + old_room.pos,
            radius,
        );

        if !rooms.is_empty() {
            bridges.push(generate_bridge(old_room.pos, new_room.pos, &mut rand));
        }

        rooms.push(new_room.clone());
        old_room = new_room;
    }

    Map { rooms, bridges }
}

fn generate_bridge(from: IVec2, to: IVec2, rand: &mut Rng) -> Bridge {
    let mut current = from.clone().as_vec2();
    let to = to.as_vec2();

    let mut positions = Vec::<IVec2>::new();
    let directions = [-90., 0., 90.];

    while current != to {
        let mut dir = (to.y - current.y).atan2(to.x - current.x).to_degrees();
        dir += *rand.sample(&directions).expect("No direction");
        dir = (dir / 90.).round() * 90.; //Increments of 90
        dir = dir.to_radians();

        current += Vec2::new(dir.cos(), dir.sin()).round();

        positions.push(current.as_ivec2());
    }

    Bridge::new(positions)
}
