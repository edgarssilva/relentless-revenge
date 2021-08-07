pub struct FollowEntity {
    pub entity: Entity,
    pub lerp_speed: f32,
}

//System for an entity to follow another
pub fn follow_entity_system(
    mut query: Query<(&mut Transform, &FollowEntity)>,
    transform_query: Query<&Transform, Without<FollowEntity>>,
    time: Res<Time>,
) {
    for (mut transform, follow_entity) in query.iter_mut() {
        if let Ok(follow_transform) = transform_query.get(follow_entity.entity) {
            transform.translation =
                transform.translation.xy()
                    .lerp(follow_transform.translation.xy(), follow_entity.lerp_speed * time.delta_seconds())
                    .extend(transform.translation.z);
        }
    }
}