use std::{collections::HashMap, fs::read_to_string};

use serde::Deserialize;

use crate::{debug::gizmos::Cylinder, enums_types::{EntityType, Faction}};

#[derive(Deserialize, Debug)]
pub struct EntityConfig {
    pub entities: Vec<EntityInstanceHelper>,
}

impl EntityConfig {
    pub fn load_from_file(file_name: &str) -> EntityConfig {
        println!("loading entity configuration from {}", &file_name);
        let config_str = read_to_string(file_name).unwrap();

        serde_json::from_str(&config_str).expect("The entity config file was missing")
    }
}


// =============================================================
// Helpers
// =============================================================

#[derive(Deserialize, Debug)]
pub struct AnimationPropHelper {
    pub name: String,
    pub one_shots: HashMap<String, Vec<u32>>,
    pub continuous_sounds: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct EntityInstanceHelper {
    pub entity_type: EntityType,
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub rotation: String,
    pub faction: Faction,
    pub mesh_path: String,
    pub bone_path: String,
    pub hit_cyl: Cylinder,
    pub animation_properties: Vec<AnimationPropHelper>,
}
