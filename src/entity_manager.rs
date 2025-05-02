#![allow(dead_code, clippy::too_many_arguments)]
use std::collections::HashSet;

use glam::{vec3, Quat, Vec3};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{animation::animation::{import_bone_data, import_model_data, Animation, Animator, Bone, Model, Vertex}, camera::Camera, collision_system, config::entity_config::{AnimationPropHelper, EntityConfig}, enums_types::{CameraState, CellType, EntityType, Faction, Rotator, Size3, Transform}, grid::Grid, movement::{handle_npc_movement, handle_player_movement, revolve_around_something}, some_data::{GRASSES, TREES}, sound::sound_manager::{ContinuousSound, OneShot}, sparse_set::SparseSet, terrain::Terrain};

pub struct EntityManager {
    pub next_entity_id: usize,
    pub transforms: SparseSet<Transform>,
    pub factions: SparseSet<Faction>,
    pub entity_types: SparseSet<EntityType>,
    pub models: SparseSet<Model>,
    pub ani_models: SparseSet<Model>,
    pub animators: SparseSet<Animator>,
    pub skellingtons: SparseSet<Bone>,
    pub rotators: SparseSet<Rotator>,
    pub hitboxes: SparseSet<Model>,
    pub sizes: SparseSet<Size3>,

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
            rotators: SparseSet::with_capacity(max_entities),
            hitboxes: SparseSet::with_capacity(max_entities),
            sizes: SparseSet::with_capacity(max_entities),

            rng: ChaCha8Rng::seed_from_u64(1)
        }
    }

    pub fn populate_initial_entity_data(&mut self, ec: &mut EntityConfig) {
        for entity in ec.entities.iter() {
            let rotation = match entity.rotation.as_str() {
                "-FRAC_PI_2" => Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                _ => Quat::IDENTITY,
            };
            match entity.faction {
                Faction::Player | Faction::Enemy => {
                    self.create_animated_entity(
                        entity.faction.clone(),
                        vec3(entity.position[0], entity.position[1], entity.position[2]), 
                        vec3(entity.scale[0], entity.scale[1], entity.scale[2]), 
                        rotation, 
                        &entity.mesh_path, 
                        &entity.bone_path,
                        &entity.animation_properties,
                    );
                },
                Faction::World | Faction::Static => {
                    self.create_static_entity(
                        entity.entity_type.clone(),
                        entity.faction.clone(),
                        vec3(entity.position[0], entity.position[1], entity.position[2]), 
                        vec3(entity.scale[0], entity.scale[1], entity.scale[2]), 
                        rotation, 
                        &entity.mesh_path, 
                    );
                },
            }
        }
    }

    pub fn create_static_entity(&mut self,entity_type: EntityType, faction: Faction, position: Vec3, scale: Vec3, rotation: Quat, model_path: &str) {
        self.factions.insert(self.next_entity_id, faction);
        self.entity_types.insert(self.next_entity_id, entity_type);

        let transform = Transform {
            position,
            rotation,
            scale,

            original_rotation: rotation,
        };
        self.transforms.insert(self.next_entity_id, transform);

        let mut model = Model::new();
        let mut found = false;
        for m in self.models.iter_mut() {
            if m.value().full_path == *model_path.to_string() {
                model = m.value().clone();
                found = true;
            }
        }

        if !found {
            model = import_model_data(model_path, &Animation::default());
        }
        self.models.insert(self.next_entity_id, model);

        
        self.next_entity_id += 1;
    }

    pub fn create_animated_entity(&mut self, faction: Faction, position: Vec3, scale: Vec3, rotation: Quat, model_path: &str, animation_path: &str, animation_props: &[AnimationPropHelper]) {
        let transform = Transform {
            position,
            rotation,
            scale,
            
            original_rotation: rotation,
        };

        let (skellington, mut animator, animation) = import_bone_data(animation_path);

        for prop in animation_props.iter() {
            let anim = animator.animations.get_mut(&prop.name).unwrap();
            for (k, v) in prop.one_shots.iter() {
                for frame in v.iter() {
                    anim.one_shots.push(OneShot {
                        sound_type: k.clone(),
                        segment: *frame,
                        triggered: false.into(),
                    });
                }
            }

            for cs in prop.continuous_sounds.iter() {
                anim.continuous_sounds.push(ContinuousSound {
                    sound_type: cs.clone(),
                    playing: false.into(),
                });
            }
        }

        let mut model = Model::new();
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
        }         

        let (mut hitbox, size) = model.create_bounding_box();
        // TODO: we need to not call this all over the place, 
        // we should just call it when creating a model at the end of the method
        hitbox.setup_opengl();

        let rotator = Rotator {
            cur_rot: rotation,
            next_rot: rotation,
            blend_factor: 0.0, 
            blend_time: 0.11,
        };
        self.rotators.insert(self.next_entity_id, rotator);

        self.animators.insert(self.next_entity_id, animator);

        self.skellingtons.insert(self.next_entity_id, skellington.clone());
        self.transforms.insert(self.next_entity_id, transform);
        self.factions.insert(self.next_entity_id, faction);
        self.ani_models.insert(self.next_entity_id, model);
        self.hitboxes.insert(self.next_entity_id, hitbox);
        self.sizes.insert(self.next_entity_id, size);

        self.next_entity_id += 1;
    }

    pub fn create_standalone_hitbox(
        &mut self,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
        min_z: f32,
        max_z: f32,
        position: Vec3,
    ) {

        let mut hitbox = Model::new();

        let vertices = vec![
            Vertex::new(Vec3::new(max_x, min_y, min_z)), // 0 
            Vertex::new(Vec3::new(max_x, max_y, min_z)), // 1
            Vertex::new(Vec3::new(max_x, max_y, max_z)), // 2
            Vertex::new(Vec3::new(max_x, min_y, max_z)), // 3
            Vertex::new(Vec3::new(min_x, max_y, max_z)), // 4
            Vertex::new(Vec3::new(min_x, min_y, max_z)), // 5
            Vertex::new(Vec3::new(min_x, max_y, min_z)), // 6
            Vertex::new(Vec3::new(min_x, min_y, min_z)), // 7
        ];

        let indices = vec![
            // Right
            0, 1, 2,    3, 0, 2,
            // Front
            3, 2, 4,    5, 3, 4,
            // Left
            5, 4, 7,    7, 4, 6,
            // Back
            7, 6, 0,    0, 7, 1,
            // Top
            2, 1, 6,    4, 2, 6,
            // Bottom
            0, 3, 5,    7, 3, 5,
        ];

        hitbox.vertices = vertices;
        hitbox.indices = indices;
        hitbox.setup_opengl();
            
        let size = Size3 {
            w: max_x - min_x,
            h: max_y - min_y,
            d: max_z - min_z,
        };

        let trans = Transform {
            position,
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quat::IDENTITY,

            original_rotation: Quat::IDENTITY,
        };

        self.hitboxes.insert(self.next_entity_id, hitbox);
        self.sizes.insert(self.next_entity_id, size);
        self.transforms.insert(self.next_entity_id, trans);
        self.factions.insert(self.next_entity_id, Faction::Enemy);
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

    pub fn update(&mut self, pressed_keys: &HashSet<glfw::Key>, delta: f64, elapsed_time: f32, camera: &Camera, terrain: &Terrain) {
        handle_npc_movement(self, terrain);
        collision_system::update(self);

        // =============================================================
        // Player Pass
        // =============================================================
        if let Some(player_entry) = self.factions.iter().find(|e| e.value() == &Faction::Player) {
            let player_key = player_entry.key();

            if camera.move_state != CameraState::Free {
                handle_player_movement(pressed_keys, self, player_key, delta, camera, terrain);
            }

            let animator = self.animators.get_mut(player_key).unwrap();
            let skellington = self.skellingtons.get_mut(player_key).unwrap();
            animator.update(elapsed_time, skellington, delta as f32);

            if let Some(donut) = self.entity_types.iter().find(|e| e.value() == &EntityType::Donut) {
                let donut_key = donut.key();

                let player_position = self.transforms.get(player_key).map(|t| t.position);

                if let Some(donut_transform) = self.transforms.get_mut(donut_key) {
                    if let Some(player_position) = player_position {
                        revolve_around_something(
                            &mut donut_transform.position,
                            &player_position,
                            elapsed_time,
                            2.0,
                            5.0
                        );
                    }
                }
            }
        }

        // =============================================================
        // Non-player pass
        // =============================================================
        for faction in self.factions.iter() {
            if faction.value() == &Faction::Player {
                continue;
            }
            let entity_key = faction.key();

            if let (Some(animator), Some(skellington)) = (
                self.animators.get_mut(entity_key), 
                self.skellingtons.get_mut(entity_key)
            ) {
                animator.update(elapsed_time, skellington, delta as f32);
            }
        }
    }
}
