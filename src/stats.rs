pub struct Stats {
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
}

impl Stats {
    pub fn new(health: u32, damage: u32, speed: u32) -> Self {
        Stats {
            health,
            damage,
            speed,
        }
    }
}
