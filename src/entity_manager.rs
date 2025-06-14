#![allow(clippy::too_many_arguments)]
use std::collections::HashSet;

use gl::PolygonMode;
use glam::{vec3, Mat4, Quat, Vec3};
use libc::EILSEQ;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{animation::{animation::{import_bone_data, import_model_data, Animation, Animator, Bone, Model}, animation_system}, camera::Camera, collision_system, config::{entity_config::{AnimationPropHelper, EntityConfig}, world_data::WorldData}, debug::gizmos::{Cuboid, Cylinder}, enums_types::{ActiveItem, CellType, EntityType, Faction, Inventory, Parent, Rotator, SimState, Transform, VisualEffect}, grid::Grid, movement_system, some_data::{GRASSES, TREES}, sound::sound_manager::{ContinuousSound, OneShot, SoundManager}, sparse_set::SparseSet, state_machines, terrain::Terrain};

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
    pub inventories: SparseSet<Inventory>,
    pub active_items: SparseSet<ActiveItem>,

    // Simulation/Behavior Components
    pub destinations: SparseSet<Vec3>,

    // Simulation gizmos
    // pub cuboids: SparseSet<Cuboid>,
    pub cylinders: SparseSet<Cylinder>,

    pub parents: SparseSet<Parent>,
    pub rng: ChaCha8Rng,

    pub selected: Vec<usize>,
    pub v_effects: SparseSet<VisualEffect>,
    pub entity_trashcan: Vec<usize>,
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
            inventories: SparseSet::with_capacity(max_entities),
            active_items: SparseSet::with_capacity(max_entities),

            destinations: SparseSet::with_capacity(max_entities),

            // cuboids: SparseSet::with_capacity(max_entities),
            cylinders: SparseSet::with_capacity(max_entities),

            parents: SparseSet::with_capacity(max_entities),
            rng: ChaCha8Rng::seed_from_u64(1),

            selected: Vec::new(),
            v_effects: SparseSet::with_capacity(max_entities),
            entity_trashcan: Vec::new(),
        }
    }

    pub fn populate_initial_entity_data(&mut self, ec: &mut EntityConfig, wd: &mut WorldData) {
        for instance in wd.entities.iter() {
            let archetype = ec.entity_types.get(&instance.entity_type).unwrap();
            let position = instance.position;
            let rotation = instance.rotation;
            let scale_correction = archetype.scale_correction;

            let rot_correction = match archetype.rot_correction.as_str() {
                "-FRAC_PI_2" => Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                _ => Quat::IDENTITY,
            };
            match instance.faction {
                Faction::Player | Faction::Enemy => {
                    self.create_animated_entity(
                        instance.faction.clone(),
                        position.into(), 
                        scale_correction.into(), 
                        rot_correction, 
                        Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]),
                        &archetype.mesh_path, 
                        &archetype.bone_path,
                        &archetype.animation_properties,
                        instance.entity_type.clone(),
                        archetype.hit_cyl.clone(),
                    );
                },
                Faction::World | Faction::Static | Faction::Gizmo | Faction::Item => {
                    self.create_static_entity(
                        instance.entity_type.clone(),
                        instance.faction.clone(),
                        position.into(), 
                        scale_correction.into(), 
                        rot_correction, 
                        Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]),
                        &archetype.mesh_path, 
                        archetype.hit_cyl.clone(),
                    );
                },
            }

        }


        {
            // Load a weapon for the player // TODO: don't hard code this
            let player_id = self.factions.iter().filter(|f| *f.value() == Faction::Player).last().unwrap().key();
            let weapon_id = self.next_entity_id;
            self.create_static_entity(
                EntityType::OrcSword, 
                Faction::Item, 
                Vec3::splat(0.0), 
                Vec3::splat(1.0), 
                Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2) * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2), 
                Quat::IDENTITY, 
                "resources/models/static/weapons/swords/001_double_axe.txt", 
                Cylinder {
                    r: 0.1,
                    h: 0.1
                }
            );

            self.active_items.insert(
                player_id,
                ActiveItem {
                    right_hand: Some(weapon_id),
                    left_hand: None,
                }
            );
        }

        {
            // Load an inventory weapon for the player // TODO: don't hard code this
            let player_id = self.factions.iter().filter(|f| *f.value() == Faction::Player).last().unwrap().key();
            let weapon_id = self.next_entity_id;
            self.create_static_entity(
                EntityType::OrcSword, 
                Faction::Item, 
                Vec3::splat(0.0), 
                Vec3::splat(1.0), 
                Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2) * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2), 
                Quat::IDENTITY, 
                "resources/models/static/weapons/swords/001_orc_sword.txt", 
                Cylinder {
                    r: 0.1,
                    h: 0.1
                }
            );

            self.inventories.insert(
                player_id,
                Inventory {
                    items: vec![weapon_id],
                },
            );
        }
    }

    pub fn create_static_entity(&mut self,entity_type: EntityType, faction: Faction, position: Vec3, scale: Vec3, rot_correction: Quat,rotation: Quat, model_path: &str, cylinder: Cylinder) {
        self.factions.insert(self.next_entity_id, faction);
        self.entity_types.insert(self.next_entity_id, entity_type);

        let transform = Transform {
            position,
            rotation: rotation * rot_correction,
            scale,

            original_rotation: rot_correction,
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

    pub fn create_animated_entity(&mut self, faction: Faction, position: Vec3, scale: Vec3, rot_correction: Quat, rotation: Quat, model_path: &str, animation_path: &str, animation_props: &[AnimationPropHelper], entity_type: EntityType, cylinder: Cylinder) {
        let transform = Transform {
            position,
            rotation: rotation * rot_correction,
            scale,
            
            original_rotation: rot_correction,
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

        let starting_rot = rotation * rot_correction;

        let rotator = Rotator {
            cur_rot: starting_rot,
            next_rot: starting_rot,
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

    pub fn update(&mut self, sm: &mut SoundManager) {
        self.delete_entities(sm);
    }

    pub fn delete_entities(&mut self, sm: &mut SoundManager) {
        for id in self.entity_trashcan.iter() {
            sm.cleanup_entity_sounds(*id);
            self.transforms.remove(*id);
            self.factions.remove(*id);
            self.entity_types.remove(*id);
            self.models.remove(*id);
            self.ani_models.remove(*id);
            self.animators.remove(*id);
            self.skellingtons.remove(*id);
            self.rotators.remove(*id);
            self.sim_states.remove(*id);
            self.destinations.remove(*id);
            self.cylinders.remove(*id);
            self.parents.remove(*id);
            self.v_effects.remove(*id);
        }

        self.entity_trashcan.clear();
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

    pub fn get_active_weapon_ids(&self) -> Vec<usize> {
        self.active_items
            .iter()
            .flat_map(|item| {
                let active = item.value();
                [active.right_hand, active.left_hand]
                    .into_iter()
                    .flatten()
            })
            .collect()
    }
}
