use bevy::prelude::{Component, Query, Transform};

//pub const MAP_LAYER: f32 = 0.0;
pub const ENTITIES_LAYER: f32 = 100.0;

const MAX_Y: f32 = 10000.0;

#[derive(Component)]
pub struct YSort(pub f32);

#[derive(Component)]
pub struct FeetOffset(pub f32);

pub fn ysort(mut query: Query<(&YSort, &mut Transform, Option<&FeetOffset>)>) {
    for (ysort, mut transform, offset) in query.iter_mut() {
        let offset = offset.map(|x| x.0).unwrap_or(0.0);
        transform.translation.z = ysort.0 - ((transform.translation.y - offset) / MAX_Y);
    }
}
