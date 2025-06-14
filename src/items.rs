use glam::Mat4;

use crate::{entity_manager::EntityManager, enums_types::Faction};
use std::{collections::HashMap, hash::Hash};

// pub fn update(em: &mut EntityManager) {
//     for a in em.active_items.iter_mut() {
//         let owner_id = a.key();
// 
//         let owner_trans = em.transforms.get(owner_id).unwrap();
//         let owner_model_trans = Mat4::from_scale_rotation_translation(
//             owner_trans.scale,
//             owner_trans.rotation,
//             owner_trans.position,
//         );
// 
//         let owner_skellington = em.skellingtons.get(owner_id).unwrap();
//         let animator = em.animators.get_mut(owner_id).unwrap();
//         let blend_factor = animator.blend_factor;
// 
//         let current_key = animator.current_animation.clone();
//         let next_key = animator.next_animation.clone();
//         let rh_name = "mixamorig:RightHand";
//         let rh_weapon_id = a.value().right_hand.unwrap();
// 
//         let mut current_anim = animator.animations.remove(&current_key).unwrap();
// 
//         let maybe_bone_world_model_space = if blend_factor > 0.0 {
//             let mut other_anim = animator.animations.remove(&next_key).unwrap();
// 
//             let result = current_anim.get_raw_global_bone_transform_by_name_blended(
//                 rh_name,
//                 owner_skellington,
//                 Mat4::IDENTITY,
//                 &mut other_anim,
//                 blend_factor,
//             );
// 
//             // Re-insert both animations
//             animator.animations.insert(next_key, other_anim);
//             result
//         } else {
//             current_anim.get_raw_global_bone_transform_by_name(
//                 rh_name,
//                 owner_skellington,
//                 Mat4::IDENTITY,
//             )
//         };
// 
//         animator.animations.insert(current_key, current_anim);
// 
//         if let Some(bone_world_model_space) = maybe_bone_world_model_space {
//             let bone_world_space = owner_model_trans * bone_world_model_space;
//             let (_, rot, pos) = bone_world_space.to_scale_rotation_translation();
// 
//             let weapon_trans = em.transforms.get_mut(rh_weapon_id).unwrap();
//             weapon_trans.position = pos;
//             weapon_trans.rotation = rot * weapon_trans.original_rotation;
//         }
//     }
// }


pub fn update(em: &mut EntityManager) {
    for a in em.active_items.iter_mut() {
        let owner_id = a.key();

        let owner_trans = em.transforms.get(owner_id).unwrap();
        let owner_model_trans = Mat4::from_scale_rotation_translation(
            owner_trans.scale,
            owner_trans.rotation,
            owner_trans.position,
        );

        let owner_skellington = em.skellingtons.get(owner_id).unwrap();
        let animator = em.animators.get_mut(owner_id).unwrap();
        let blend_factor = animator.blend_factor;

        let current_key = animator.current_animation.clone();
        let next_key = animator.next_animation.clone();
        let rh_name = "mixamorig:RightHand";
        let rh_weapon_id = a.value().right_hand.unwrap();

        let maybe_bone_world_model_space = if blend_factor > 0.0 && current_key != next_key {
            // SAFELY get both entries with mutable refs (no HashMap::remove)
            let (a1, a2) = {
                let (left, right) = animator.animations.get_pair_mut(&current_key, &next_key)
                    .expect("Both animations must exist");
                (left, right)
            };

            a1.get_raw_global_bone_transform_by_name_blended(
                rh_name,
                owner_skellington,
                Mat4::IDENTITY,
                a2,
                blend_factor,
            )
        } else {
            animator.animations
                .get_mut(&current_key)
                .unwrap()
                .get_raw_global_bone_transform_by_name(
                    rh_name,
                    owner_skellington,
                    Mat4::IDENTITY,
                )
        };

        if let Some(bone_world_model_space) = maybe_bone_world_model_space {
            let bone_world_space = owner_model_trans * bone_world_model_space;
            let (_, rot, pos) = bone_world_space.to_scale_rotation_translation();

            let weapon_trans = em.transforms.get_mut(rh_weapon_id).unwrap();
            weapon_trans.position = pos;
            weapon_trans.rotation = rot * weapon_trans.original_rotation;
        }
    }
}


pub trait HashMapGetPairMut<K: Eq + std::hash::Hash, V> {
    fn get_pair_mut(&mut self, k1: &K, k2: &K) -> Option<(&mut V, &mut V)>;
}

impl<K: Eq + Hash, V> HashMapGetPairMut<K, V> for HashMap<K, V> {
    fn get_pair_mut(&mut self, k1: &K, k2: &K) -> Option<(&mut V, &mut V)> {
        if k1 == k2 {
            return None;
        }

        let (v1_ptr, v2_ptr) = {
            let v1 = self.get_mut(k1)? as *mut V;
            let v2 = self.get_mut(k2)? as *mut V;
            (v1, v2)
        };

        // SAFETY: We ensure v1 and v2 are different by checking k1 != k2
        unsafe {
            Some((&mut *v1_ptr, &mut *v2_ptr))
        }
    }
}
