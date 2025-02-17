use glam::{vec3, Mat4, Quat, Vec3};

use crate::{camera::Camera, enums_types::{EntityType, Transform}, grid::Grid, lights::Lights, model::Model, shaders::Shader, sparse_set::SparseSet};

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

    pub fn create_entity(&mut self, entity_type: EntityType, position: Vec3, model_path: &str) {
        let transform = Transform {
            position,
            rotation: Mat4::IDENTITY,
            scale: vec3(1.2, 1.0, 1.2),
        };

        let model = Model::load(model_path);

        self.transforms.insert(self.next_entity_id, transform);
        self.entity_types.insert(self.next_entity_id, entity_type);
        self.models.insert(self.next_entity_id, model);

        self.next_entity_id += 1;
    }

    pub fn populate_floor_tiles(&mut self, grid: &Grid, model_path: &str) {
        for cell in grid.cells.iter() {
            let mut pos = cell.position;
            pos.y -= 0.98;
            self.create_entity(EntityType::BlockGrass, pos, model_path);
        }
    }

    pub fn update(&mut self, delta: &f64) {
    }

    pub fn draw(&mut self, shader: &mut Shader, camera: &mut Camera, light_manager: &Lights) {
        for model in self.models.iter() {
            let trans = self.transforms.get(model.key()).unwrap();

            camera.model = Mat4::IDENTITY * Mat4::from_translation(trans.position) * Mat4::from_scale(trans.scale);

            shader.activate();
            shader.set_mat4("model", camera.model);
            shader.set_mat4("view", camera.view);
            shader.set_mat4("projection", camera.projection);
            shader.set_mat4("light_space_mat", camera.light_space);
            shader.set_dir_light("dir_light", &light_manager.dir_light);

            model.value.draw(shader);
        }
    }
}
