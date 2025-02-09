use glam::{Mat4, Quat, Vec3};

use crate::{enums_types::{EntityType, Transform}, lights::Lights, sparse_set::SparseSet};

pub struct EntityManager {
    pub next_entity_id: usize,
    pub transforms: SparseSet<Transform>,
    pub entity_types: SparseSet<EntityType>,
}

impl EntityManager {
    pub fn new(max_entities: usize) -> Self {
        Self {
            next_entity_id: 0,
            transforms: SparseSet::with_capacity(max_entities),
            entity_types: SparseSet::with_capacity(max_entities)
        }
    }

    pub fn create_unit_cube(&mut self, position: Vec3, rotation: Mat4) {
    }

    pub fn create_entity(&mut self) {
    }

    pub fn update(&mut self, delta: &f64) {
    }
}
