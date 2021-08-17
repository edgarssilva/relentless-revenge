use bevy::math::{Vec3, Vec3Swizzles};
use bevy::prelude::{Entity, Query, Res, Time, Transform, Without};

pub struct Follow {
    pub speed: f32,
    pub target: FollowTarget,
}

pub enum FollowTarget {
    Entity(Entity),
    Position(Vec3),
}

//System for an entity to follow another
pub fn follow_entity_system(
    mut query: Query<(&mut Transform, &Follow)>,
    transform_query: Query<&Transform, Without<Follow>>,
    time: Res<Time>,
) {
    for (mut transform, follow) in query.iter_mut() {
        let pos = match follow.target {
            FollowTarget::Position(pos) => pos,
            FollowTarget::Entity(entity) => {
                transform_query
                    .get(entity)
                    .expect("Tried to follow an entity without position!")
                    .translation
            }
        };

        if transform.translation.xy().distance(pos.xy()) > 0.5 {
            //TODO: Check distance threshold (This was added because of the parallax on the Changed)
            transform.translation = transform
                .translation
                .xy()
                .lerp(pos.xy(), follow.speed * time.delta_seconds())
                .extend(transform.translation.z);
        }
    }
}
