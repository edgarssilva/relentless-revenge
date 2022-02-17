#[derive(Clone, Copy, Debug)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub radius: i32,
}

impl Room {
    pub fn new(x: i32, y: i32, radius: i32) -> Self {
        Room {
            x,
            y,
            radius,
        }
    }
}
