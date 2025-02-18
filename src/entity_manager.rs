use glam::{vec3, Mat4, Quat, Vec3};

use crate::{camera::Camera, enums_types::{EntityType, Transform}, gl_call, grid::Grid, lights::Lights, model::Model, shaders::Shader, some_data::{SHADOW_HEIGHT, SHADOW_WIDTH}, sparse_set::SparseSet};

pub struct EntityManager {
    pub next_entity_id: usize,
    pub transforms: SparseSet<Transform>,
    pub entity_types: SparseSet<EntityType>,
    pub models: SparseSet<Model>,
}

impl EntityManager {
    pub fn new(max_entities: usize) -> Self {
        Self {
            next_entity_id: 0,
            transforms: SparseSet::with_capacity(max_entities),
            entity_types: SparseSet::with_capacity(max_entities),
            models: SparseSet::with_capacity(max_entities)
        }
    }

    pub fn create_unit_cube(&mut self, position: Vec3, rotation: Mat4) {
    }

    pub fn create_entity(&mut self, entity_type: EntityType, position: Vec3, scale: Vec3, model_path: &str) {
        let transform = Transform {
            position,
            rotation: Mat4::IDENTITY,
            scale,
        };

        let mut model = Model::new();

        let mut found = false;
        for m in self.models.iter_mut() {
            if m.value().full_path == model_path.to_string() {
                model = m.value().clone();
                found = true;
            }
        }

        if !found {
            model = Model::load(model_path);
        }
        
        self.transforms.insert(self.next_entity_id, transform);
        self.entity_types.insert(self.next_entity_id, entity_type);
        self.models.insert(self.next_entity_id, model);

        self.next_entity_id += 1;
    }

    pub fn populate_floor_tiles(&mut self, grid: &Grid, model_path: &str) {
        for cell in grid.cells.iter() {
            let pos = cell.position;
            // pos.y -= 0.98;
            self.create_entity(EntityType::BlockGrass, pos, vec3(1.0, 1.0, 1.0), model_path);
        }
    }

    pub fn update(&mut self, delta: &f64) {
    }
}
