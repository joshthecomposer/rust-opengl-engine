#![allow(clippy::too_many_arguments)]
use std::collections::HashSet;

use glam::{vec3, Mat4, Quat, Vec3};
use libc::EILSEQ;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{animation::{animation::{import_bone_data, import_model_data, Animation, Animator, Bone, Model}, animation_system}, camera::Camera, collision_system, config::entity_config::{AnimationPropHelper, EntityConfig}, debug::gizmos::{Cuboid, Cylinder}, enums_types::{CellType, EntityType, Faction, Parent, Rotator, SimState, Transform}, grid::Grid, movement_system, some_data::{GRASSES, TREES}, sound::sound_manager::{ContinuousSound, OneShot}, sparse_set::SparseSet, state_machines, terrain::Terrain};

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
    pub selected: SparseSet<bool>,

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
            selected: SparseSet::with_capacity(max_entities),

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
                        entity.hit_cyl.clone(),
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
                        entity.hit_cyl.clone(),
                    );
                },
            }
        }
    }

    pub fn create_static_entity(&mut self,entity_type: EntityType, faction: Faction, position: Vec3, scale: Vec3, rotation: Quat, model_path: &str, cylinder: Cylinder) {
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
        self.selected.insert(self.next_entity_id, false);
        
        self.next_entity_id += 1;

        // TODO: Should foliage be a child of the tree trunk?? Then when doing things we iterate up the parent tree?

        // CYLINDER PASS
        let cyl = cylinder;

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

    pub fn create_animated_entity(&mut self, faction: Faction, position: Vec3, scale: Vec3, rotation: Quat, model_path: &str, animation_path: &str, animation_props: &[AnimationPropHelper], entity_type: EntityType, cylinder: Cylinder) {
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
        self.selected.insert(self.next_entity_id, false);
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

        let cyl = cylinder;

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

    pub fn get_ids_for_faction(&self, faction: Faction) -> Vec<usize> {
        let result: Vec<usize> = self.factions
            .iter()
            .filter_map(|f|
                if *f.value() == faction {
                    Some(f.key())
                } else {
                    None
                }
            )
            .collect();

            result
    }

    pub fn get_ids_for_type(&self, entity_type: EntityType) -> Vec<usize> {
        let result: Vec<usize> = self.entity_types
            .iter()
            .filter_map(|f|
                if *f.value() == entity_type {
                    Some(f.key())
                } else {
                    None
                }
            )
            .collect();

            result
    }
}
