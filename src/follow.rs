use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::{Commands, Entity, Query, Res, Time, Transform};

pub struct Follow {
    pub target: FollowTarget,
    pub speed: f32,
    pub continous: bool,
    pub treshhold: f32,
    pub(crate) on_target: bool,
}

impl Follow {
    pub fn new(target: FollowTarget, speed: f32, continous: bool) -> Self {
        Self {
            target,
            speed,
            continous,
            ..Default::default()
        }
    }

    pub fn on_target(self: &Self) -> bool {
        self.on_target
    }
}

impl Default for Follow {
    fn default() -> Follow {
        Follow {
            target: FollowTarget::Position(Vec3::default()),
            speed: 1.,
            continous: true,
            treshhold: 0.25,
            on_target: false,
        }
    }
}

pub enum FollowTarget {
    Transform(Entity),
    Position(Vec3),
}

//System for an entity to follow another
pub fn follow_entity_system(
    mut commands: Commands,
    mut query_followers: Query<(&mut Follow, Entity)>,
    mut transform_query: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (mut follow, entity) in query_followers.iter_mut() {
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
            if transform.translation.xy().distance(pos) > follow.treshhold {
                transform.translation = transform
                    .translation
                    .xy()
                    .lerp(pos, follow.speed * time.delta_seconds())
                    .extend(transform.translation.z);

                follow.on_target = false;
            } else {
                follow.on_target = true;

                if !follow.continous {
                    commands.entity(entity).remove::<Follow>();
                }
            }
        }
    }
}
