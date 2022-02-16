pub mod generation;
mod room;

/*  let max_rooms = 20;
    let min_room_width = 6;
    let max_room_width = 12;
    let min_room_height = 6;
    let max_room_height = 12;
    let map_width = layer_settings.map_size.0 * layer_settings.chunk_size.0;
    let map_height = layer_settings.map_size.1 * layer_settings.chunk_size.1;
    let mut rooms = Vec::<Room>::new();

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
        let room = Room::new(x, y, width, height);

        // check all other Rooms we've placed to see if this one
        // collides with them
        for other_room in &rooms {
            if room.intersects(&other_room) {
                collides = true;
                break;
            }
        }

        // if the new Room doesn't collide, add it to the level
        if !collides {
            for row in 0..room.height {
                for col in 0..room.width {
                    let y = room.y + row;
                    let x = room.x + col;

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

            rooms.push(room);
        }
    } */