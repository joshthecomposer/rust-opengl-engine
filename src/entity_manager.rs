#![allow(dead_code)]
use std::collections::HashSet;

use glam::{vec3, Mat4, Quat, Vec3};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{animation::animation::{import_bone_data, import_model_data, AniModel, Animator, Bone}, camera::Camera, enums_types::{CellType, EntityType, Faction, Transform}, grid::Grid, model::Model, movement::handle_player_movement, some_data::{GRASSES, TREES}, sparse_set::SparseSet};

pub struct EntityManager {
    pub next_entity_id: usize,
    pub transforms: SparseSet<Transform>,
    pub factions: SparseSet<Faction>,
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
            factions: SparseSet::with_capacity(max_entities),
            entity_types: SparseSet::with_capacity(max_entities),
            models: SparseSet::with_capacity(max_entities),
            ani_models: SparseSet::with_capacity(max_entities),
            animators: SparseSet::with_capacity(max_entities),
            skellingtons: SparseSet::with_capacity(max_entities),
            rng: ChaCha8Rng::seed_from_u64(1)
        }
    }

    pub fn create_static_entity(&mut self,entity_type: EntityType, faction: Faction, position: Vec3, scale: Vec3, rotation: Quat, model_path: &str) {
        let transform = Transform {
            position,
            rotation,
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
        self.factions.insert(self.next_entity_id, faction);
        self.entity_types.insert(self.next_entity_id, entity_type);
        self.models.insert(self.next_entity_id, model);

        self.next_entity_id += 1;
    }

    pub fn create_animated_entity(&mut self, faction: Faction, position: Vec3, scale: Vec3, rotation: Quat, model_path: &str, animation_path: &str) {
        let transform = Transform {
            position,
            rotation,
            scale,
        };

        let (skellington, mut animator, animation) = import_bone_data(animation_path);

        let mut model = AniModel::new();
        let mut found = false;
        for m in self.ani_models.iter_mut() {
            if m.value().full_path == *model_path.to_string() {
                println!("FOUND DUPLICATE MODEL, CLONING");
                model = m.value().clone();
                found = true;
            }
        }

        if !found {
            model = import_model_data(model_path, &animation);
            model.setup_opengl();
        }         

        self.animators.insert(self.next_entity_id, animator);

        self.skellingtons.insert(self.next_entity_id, skellington.clone());
        self.transforms.insert(self.next_entity_id, transform);
        self.factions.insert(self.next_entity_id, faction);
        self.ani_models.insert(self.next_entity_id, model);

        self.next_entity_id += 1;
    }

    // TODO: This should be in grid
    pub fn populate_floor_tiles(&mut self, grid: &Grid, model_path: &str) {
        for cell in grid.cells.iter() {
            let pos = cell.position;
            self.create_static_entity(EntityType::BlockGrass, Faction::World, pos, vec3(1.0, 1.0, 1.0), Quat::IDENTITY, model_path);
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
                            Faction::World,
                            vec3(cell_pos.x + offset_x + smoff, 0.0, cell_pos.z + offset_z + smoff),
                            Vec3::splat(scale),
                            Quat::IDENTITY,
                            entity_data[num],
                        );
                    }
                }
            }
        }

    }

    pub fn update(&mut self, pressed_keys: &HashSet<glfw::Key>, delta: f64, elapsed_time: f32, camera: &Camera) {
        if !camera.free {
            if let Some(player_entry) = self.factions.iter().find(|e| e.value() == &Faction::Player) {
                let player_key = player_entry.key();

                handle_player_movement(pressed_keys, self, player_key, delta);
            }
        }

        for animator in self.animators.iter_mut() {
            if let Some(skellington) = self.skellingtons.get_mut(animator.key()) {
                animator.value.update(elapsed_time, skellington);
            }         
        }
    }
}
