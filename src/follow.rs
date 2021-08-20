use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::{Entity, Query, Res, Time, Transform, Without};

pub struct Follow {
    pub speed: f32,
    pub target: FollowTarget,
}

pub enum FollowTarget {
    Transform(Entity),
    Position(Vec3),
}

//System for an entity to follow another
pub fn follow_entity_system(
    mut query: Query<(&mut Transform, &Follow)>,
    transform_query: Query<&Transform, Without<Follow>>,
    // rigid_query: Query<&RigidBodyPosition, Without<Follow>>,
    time: Res<Time>,
) {
    for (mut transform, follow) in query.iter_mut() {
        let pos: Vec2 = match follow.target {
            FollowTarget::Position(pos) => pos.xy(),
            FollowTarget::Transform(entity) => transform_query
                .get(entity)
                .expect("Tried to follow an entity without transform!")
                .translation
                .xy(),
        };

        if transform.translation.xy().distance(pos) > 0.5 {
            //TODO: Check distance threshold (This was added because of the parallax on the Changed)
            transform.translation = transform
                .translation
                .xy()
                .lerp(pos, follow.speed * time.delta_seconds())
                .extend(transform.translation.z);
        }
    }
}
