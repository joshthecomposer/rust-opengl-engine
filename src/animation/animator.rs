use glam::Mat4;

use super::animation::Animation;

pub struct Animator<'a> {
    final_bone_matrices: Vec<Mat4>,
    current_animation: &'a mut Animation,
    current_time: f64,
}

impl<'a> Animator<'a> {
    pub fn new(animation: &mut Animation) -> Self {
        let mut final_bone_matrices = Vec::new();
        for i in 0..200 {
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
    }

    pub fn calculate_bone_transformation(node: &mut AssimpNodeData, parent_transform: Mat4) {

    }
}
