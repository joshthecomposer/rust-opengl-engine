#![allow(dead_code, clippy::too_many_arguments)]
use std::collections::HashSet;

use glam::{vec3, Quat, Vec3};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{animation::animation::{import_bone_data, import_model_data, Animation, Animator, Bone, Model}, camera::Camera, collision_system, config::entity_config::{AnimationPropHelper, EntityConfig}, debug::gizmos::{Cuboid, Cylinder}, enums_types::{CameraState, CellType, EntityType, Faction, Parent, Rotator, SimState, Transform}, grid::Grid, movement::{handle_npc_movement, handle_player_movement, revolve_around_something}, some_data::{GRASSES, TREES}, sound::sound_manager::{ContinuousSound, OneShot}, sparse_set::SparseSet, state_machines::entity_sim_state_machine, terrain::Terrain};

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
    pub sim_states: SparseSet<SimState>,

    // Simulation/Behavior Components
    pub destinations: SparseSet<Vec3>,

    // Simulation gizmos
    pub cuboids: SparseSet<Cuboid>,
    pub cylinders: SparseSet<Cylinder>,

    pub parents: SparseSet<Parent>,
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
            sim_states: SparseSet::with_capacity(max_entities),

            destinations: SparseSet::with_capacity(max_entities),

            cuboids: SparseSet::with_capacity(max_entities),
            cylinders: SparseSet::with_capacity(max_entities),

            parents: SparseSet::with_capacity(max_entities),
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
                        entity.entity_type.clone(),
                    );
                },
                Faction::World | Faction::Static | Faction::Gizmo => {
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

    pub fn create_animated_entity(&mut self, faction: Faction, position: Vec3, scale: Vec3, rotation: Quat, model_path: &str, animation_path: &str, animation_props: &[AnimationPropHelper], entity_type: EntityType) {
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

        let rotator = Rotator {
            cur_rot: rotation,
            next_rot: rotation,
            blend_factor: 0.0, 
            blend_time: 0.11,
        };
        self.rotators.insert(self.next_entity_id, rotator);

        if faction != Faction::Player {
            self.destinations.insert(self.next_entity_id, position);
        }

        self.animators.insert(self.next_entity_id, animator);

        self.skellingtons.insert(self.next_entity_id, skellington.clone());
        self.transforms.insert(self.next_entity_id, transform);
        self.factions.insert(self.next_entity_id, faction.clone());
        self.ani_models.insert(self.next_entity_id, model);
        self.entity_types.insert(self.next_entity_id, entity_type.clone());

        let starting_state = match entity_type {
            EntityType::MooseMan => {
                SimState::Dancing
            },
            _ => {
                SimState::Waiting
            },
        };
        self.sim_states.insert(self.next_entity_id, starting_state);

        self.next_entity_id += 1;

        // TODO: Do not hard code cylinder sizes, put them in the config
        let cyl = match entity_type {
            EntityType::MooseMan => {
                Cylinder {
                    r: 0.5,
                    h: 2.0,
                }
            },
            _ => {
                Cylinder {
                    r: 0.22,
                    h: 1.6,
                }
            },
        };


        let cyl_mod = cyl.create_model(12);
        self.cylinders.insert(self.next_entity_id, cyl);
        
        self.models.insert(self.next_entity_id, cyl_mod);
        self.factions.insert(self.next_entity_id, Faction::Gizmo);
        self.entity_types.insert(self.next_entity_id, EntityType::Cylinder);
        self.transforms.insert(self.next_entity_id, Transform {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(1.0),
            original_rotation: Quat::IDENTITY,
        });

        self.parents.insert(self.next_entity_id, Parent{
            parent_id: self.next_entity_id - 1,
        });

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

    pub fn update(&mut self, pressed_keys: &HashSet<glfw::Key>, delta: f64, elapsed_time: f32, camera: &Camera, terrain: &Terrain) {
        handle_npc_movement(self, terrain, delta as f32);
        entity_sim_state_machine(self);
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
            animator.update(skellington, delta as f32);

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
                animator.update(skellington, delta as f32);
            }
        }

        // =============================================================
        // Gizmo Pass
        // =============================================================
        let mut transforms_to_update:Vec<(usize, usize)> = vec![];
        for faction in self.factions.iter() {
            if faction.value() == &Faction::Gizmo {
                let entity_key = faction.key();

                if let Some(parent) = self.parents.get(entity_key) {
                    transforms_to_update.push((entity_key, parent.parent_id))
                }
            }
        }

        for (child_id, parent_id) in transforms_to_update {
            let parent_transform = self.transforms.get(parent_id).unwrap().clone();
            let child_transform = self.transforms.get(child_id).unwrap().clone();
            
            // Some magic to make sure the cylinder is rotated properly despite the parent being originally offset in some way
            let adjusted_rotation = parent_transform.rotation
            * parent_transform.original_rotation.inverse()
            * child_transform.original_rotation.inverse();

            self.transforms.insert(child_id, Transform {
                position: parent_transform.position,
                rotation: adjusted_rotation,
                scale: child_transform.scale,
                original_rotation: child_transform.original_rotation,
            });
        }
    }
}
