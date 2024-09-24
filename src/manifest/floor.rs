use std::convert::Infallible;

use bevy::{asset::Asset, ecs::system::Resource, reflect::TypePath, utils::HashMap};
use leafwing_manifest::{
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DomainData {
    pub name: String,
    pub floors: (u32, u32),
    pub rooms: (u32, u32),
    pub room_size: (u32, u32),
    pub boss: String, //Boss name
    pub enemies_count: (u32, u32),
    pub enemies: Vec<(u32, String)>, // [(Spawn Weight, Enemy Name)]
}

#[derive(Debug, Asset, TypePath, Serialize, Deserialize, PartialEq)]
pub struct RawDomainManifest {
    domains: Vec<DomainData>,
}

#[derive(Debug, Resource, PartialEq)]
pub struct DomainManifest {
    pub domains: HashMap<Id<DomainData>, DomainData>,
}

impl Manifest for DomainManifest {
    type RawManifest = RawDomainManifest;

    type RawItem = DomainData;

    type Item = DomainData;

    type ConversionError = Infallible;

    const FORMAT: ManifestFormat = ManifestFormat::Yaml;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        _world: &mut bevy::prelude::World,
    ) -> Result<Self, Self::ConversionError> {
        let domains: bevy::utils::hashbrown::HashMap<Id<DomainData>, DomainData> = raw_manifest
            .domains
            .iter()
            .map(|raw| (Id::from_name(&raw.name.as_str()), raw.clone()))
            .collect();

        Ok(DomainManifest { domains })
    }

    fn get(&self, id: Id<Self::Item>) -> Option<&Self::Item> {
        self.domains.get(&id)
    }
}
