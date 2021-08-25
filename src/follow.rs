use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::{Commands, Entity, Query, Res, Time, Transform};

pub struct Follow {
    pub speed: f32,
    pub target: FollowTarget,
    pub continous: bool,
}

pub enum FollowTarget {
    Transform(Entity),
    Position(Vec3),
}

//System for an entity to follow another
pub fn follow_entity_system(
    mut commands: Commands,
    query_followers: Query<(&Follow, Entity)>,
    mut transform_query: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (follow, entity) in query_followers.iter() {
        let pos: Vec2 = match follow.target {
            FollowTarget::Position(pos) => pos.xy(),
            FollowTarget::Transform(e) => transform_query
                .get_mut(e)
                .expect("Tried to follow an entity without transform!")
                .translation
                .xy(),
        };

        //Workaround for nested queries
        if let Ok(mut transform) = transform_query.get_mut(entity) {
            //TODO: Check distance threshold (This was added because of Changed<>)
            if transform.translation.xy().distance(pos) > 0.25 {
                transform.translation = transform
                    .translation
                    .xy()
                    .lerp(pos, follow.speed * time.delta_seconds())
                    .extend(transform.translation.z);
            } else if !follow.continous {
                commands.entity(entity).remove::<Follow>();
            }
        }
    }
}
