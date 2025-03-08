use std::collections::HashMap;

use glam::Mat4;

use crate::debug::write::write_data;

use super::{ani_model::BoneInfo, animation::{AnimationClip, Keyframe}};

#[derive(Debug, Clone)]
pub struct Animator {
    pub current_time: f32,
    pub animation_clip: AnimationClip,
}

impl Animator {
    pub fn new(animation_clip: AnimationClip) -> Self {
        Self {
            current_time: 0.0,
            animation_clip,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.current_time += delta_time;
        if self.current_time > self.animation_clip.duration {
            self.current_time = 0.0; // Loop animation
        }
    }

    pub fn get_bone_transforms(&self, bone_info_map: &HashMap<String, BoneInfo>) -> Vec<Mat4> {
        let mut bone_transforms = vec![Mat4::IDENTITY; bone_info_map.len()];
    
        // TODO: Time based animation might be better than this to optimize out some calculations
        let current_frame = (self.current_time * self.animation_clip.fps).floor() as usize;

        for bone_anim in &self.animation_clip.bone_animations {
            let bone_info = bone_info_map.get(&bone_anim.bone_name);
            if let Some(bone) = bone_info {
                let transform = self.interpolate_keyframes(&bone_anim.keyframes, current_frame);
                bone_transforms[bone.id as usize] = transform;
            }
        }

        // dbg!(&bone_transforms.len());

        bone_transforms
    }

    fn interpolate_keyframes(&self, keyframes: &Vec<Keyframe>, current_frame: usize) -> Mat4 {
        if keyframes.is_empty() {
            return Mat4::IDENTITY;
        }

        let prev_keyframe = keyframes.iter().rev().find(|kf| kf.frame_index <= current_frame);
        let next_keyframe = keyframes.iter().find(|kf| kf.frame_index > current_frame);

        if let (Some(prev), Some(next)) = (prev_keyframe, next_keyframe) {
            let factor = (current_frame as f32 - prev.frame_index as f32) / 
            (next.frame_index as f32 - prev.frame_index as f32).max(1.0);

            let interpolated_pos = prev.position.lerp(next.position, factor);
            let interpolated_rot = prev.rotation.slerp(next.rotation, factor);
            let interpolated_scale = prev.scale.lerp(next.scale, factor);

            Mat4::from_scale_rotation_translation(interpolated_scale, interpolated_rot, interpolated_pos)
        } else if let Some(single_frame) = prev_keyframe.or(next_keyframe) {
            Mat4::from_scale_rotation_translation(single_frame.scale, single_frame.rotation, single_frame.position)
        } else {
            Mat4::IDENTITY
        }
    }
}
