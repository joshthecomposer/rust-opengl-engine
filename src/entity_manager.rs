#![allow(dead_code)]
use glam::{vec3, Mat4, Vec3};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{animation::animation::{import_bone_data, import_model_data, AniModel, Animator, Bone}, enums_types::{CellType, EntityType, Transform}, grid::Grid, model::Model, some_data::{GRASSES, TREES}, sparse_set::SparseSet};

pub struct EntityManager {
    pub next_entity_id: usize,
    pub transforms: SparseSet<Transform>,
    pub entity_types: SparseSet<EntityType>,
    pub models: SparseSet<Model>,
    pub ani_models: SparseSet<AniModel>,
    pub animators: SparseSet<Animator>,
    pub skellingtons: SparseSet<Bone>,
    pub rng: ChaCha8Rng,
}

impl EntityManager {
    pub fn new(max_entities: usize) -> Self {
        Self {
            next_entity_id: 0,
            transforms: SparseSet::with_capacity(max_entities),
            entity_types: SparseSet::with_capacity(max_entities),
            models: SparseSet::with_capacity(max_entities),
            ani_models: SparseSet::with_capacity(max_entities),
            animators: SparseSet::with_capacity(max_entities),
            skellingtons: SparseSet::with_capacity(max_entities),
            rng: ChaCha8Rng::seed_from_u64(1)
        }
    }

    pub fn create_static_entity(&mut self, entity_type: EntityType, position: Vec3, scale: Vec3, model_path: &str) {
        let transform = Transform {
            position,
            rotation: Mat4::IDENTITY,
            scale,
        };

        let mut model = Model::new();

        let mut found = false;
        for m in self.models.iter_mut() {
            if m.value().full_path == *model_path.to_string() {
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

    pub fn create_animated_entity(&mut self, entity_type: EntityType, position: Vec3, scale: Vec3, model_path: &str, animation_path: &str) {
        let transform = Transform {
            position,
            rotation: Mat4::IDENTITY,
            scale,
        };

        // let mut model = AniModel::new();
        // let mut found = false;
        // for m in self.ani_models.iter_mut() {
        //     if m.value().full_path == *model_path.to_string() {
        //         model = m.value().clone();
        //         found = true;
        //     }
        // }

        // if !found {
        let (skellington, animation) = import_bone_data(animation_path);
        dbg!(&skellington);
        let mut model = import_model_data(model_path, &animation);
        model.setup_opengl();
        let mut animator = Animator::new(animation);
        animator.set_current_animation("Run");
        // }
        
        self.transforms.insert(self.next_entity_id, transform);
        self.entity_types.insert(self.next_entity_id, entity_type);
        self.ani_models.insert(self.next_entity_id, model);
        self.skellingtons.insert(self.next_entity_id, skellington.clone());
        self.animators.insert(self.next_entity_id, animator);

        self.next_entity_id += 1;
    }

    pub fn populate_floor_tiles(&mut self, grid: &Grid, model_path: &str) {
        for cell in grid.cells.iter() {
            let pos = cell.position;
            // pos.y -= 0.98;
            self.create_static_entity(EntityType::BlockGrass, pos, vec3(1.0, 1.0, 1.0), model_path);
        }
    }

    pub fn populate_cell_rng(&mut self, grid: &Grid) {
        for cell in grid.cells.iter() {

            let (entity_data, subtile_size, entity_type) = match cell.cell_type {
                CellType::Tree => (TREES, 3.0, EntityType::Tree),
                CellType::Grass => (GRASSES, 3.0, EntityType::Grass),
                _=> continue,
            };

            let within = grid.cell_size / subtile_size;

            let cell_pos = cell.position;
            for x in -1..=1 {
                for z in -1..=1 {
                    let num = self.rng.random_range(0..entity_data.len() + 1);
                    let scale = match entity_type {
                        EntityType::Grass => self.rng.random_range(20..=45) as f32 / 100.0,
                        EntityType::Tree => self.rng.random_range(90..=110) as f32 / 100.0,
                        _=> 1.0,
                };
                    let smoff = self.rng.random_range(-0.1..=0.1);

                    let offset_x = x as f32 * within;
                    let offset_z = z as f32 * within;

                    if num < entity_data.len() {
                        self.create_static_entity(
                            entity_type.clone(),
                            vec3(cell_pos.x + offset_x + smoff, 0.0, cell_pos.z + offset_z + smoff),
                            Vec3::splat(scale),
                            entity_data[num],
                        );
                    }
                }
            }
        }

    }

    pub fn update(&mut self, delta: &f64, elapsed_time: f32) {
        for animator in self.animators.iter_mut() {
            if let Some(skellington) = self.skellingtons.get_mut(animator.key()) {
                animator.value.update(elapsed_time, skellington);
            }         
        }
    }
}
