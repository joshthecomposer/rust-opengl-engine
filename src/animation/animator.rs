use std::fs::OpenOptions;
use std::io::Write;

use glam::Mat4;

use crate::debug::write::write_data;

use super::animation::{Animation, AssimpNodeData};

pub struct Animator {
    pub final_bone_matrices: Vec<Mat4>,
    pub current_animation: Animation,
    pub current_time: f64,
}

impl Animator {
    pub fn new(animation: Animation) -> Self {
        let mut final_bone_matrices = Vec::new();
        for i in 0..animation.bone_info_map.keys().len() {
            final_bone_matrices.push(Mat4::IDENTITY);
        }


        Self {
            final_bone_matrices,
            current_animation: animation,
            current_time: 0.0,
        }
    }

    pub fn update(&mut self, delta: f64) {
        self.current_time += self.current_animation.ticks_per_second * delta;
        self.current_time = self.current_time % self.current_animation.duration;

        let mut root_node = self.current_animation.root_node.clone();

        write_data(&root_node, "root_node.txt");

        Self::calculate_bone_transformation(&mut root_node, self, Mat4::IDENTITY);

        // panic!();
    }

    pub fn calculate_bone_transformation(node: &mut AssimpNodeData, animator: &mut Animator, parent_transform: Mat4,) {
        let node_name = node.name.clone();
        let mut node_transform = node.transformation.clone();
        
        if let Some(bone) = animator.current_animation.find_bone(&node_name) {
            bone.update(animator.current_time);
            
            if bone.local_transform.is_finite() {
                node_transform = bone.local_transform;
            } else {
                write_data(bone, "bad_transform.txt");
            }


        }

        let global_transformation =  parent_transform * node_transform ;
        
        dbg!(&node_name);
        dbg!(&parent_transform);
        dbg!(&global_transformation);

        if let Some(bone_info) = animator.current_animation.bone_info_map.get(&node_name) {

            let offset = Self::correct_offset_matrix(bone_info.offset);

            *animator.final_bone_matrices.get_mut(bone_info.id as usize).unwrap() = global_transformation * offset;
            write_data(bone_info, "bone_info.txt");
        }

        for child in node.children.iter_mut() {
            Self::calculate_bone_transformation(child, animator, global_transformation);
        }
    }

    fn correct_offset_matrix(offset: Mat4) -> Mat4 {
        Mat4::from_cols(
            offset.x_axis,
            offset.z_axis,
            -offset.y_axis,
            offset.w_axis,
        )
    }
}
