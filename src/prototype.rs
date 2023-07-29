use std::any::Any;
use crate::player::PlayerBundle;
use bevy::prelude::{Component, FromReflect, Vec2};
use bevy::reflect::{Reflect, ReflectMut, ReflectOwned, ReflectRef, Typed, TypeInfo};
use bevy_proto::prelude::{DependenciesBuilder, Schematic, SchematicContext};

#[derive(Reflect, FromReflect)]
pub struct StatsProto {
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
    pub cooldown: u32,
    pub xp: u32,
}

#[derive(Reflect, FromReflect)]
pub struct PlayerProto {
    name: String,
    stats: StatsProto,
    size: Vec2,
    hitbox: Vec2,
}

impl Schematic for PlayerBundle {
    type Input = PlayerProto;

    fn apply(input: &Self::Input, context: &mut SchematicContext) {}

    fn remove(input: &Self::Input, context: &mut SchematicContext) {}

    fn preload_dependencies(input: &mut Self::Input, dependencies: &mut DependenciesBuilder) {}
}
