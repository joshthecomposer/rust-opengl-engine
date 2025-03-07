use std::fs::OpenOptions;
use std::io::Write;

use glam::{vec3, Mat4, Quat, Vec3};
use russimp::animation::NodeAnim;

use crate::debug::write::write_data;

#[derive(Debug, Clone)]
pub struct KeyPosition {
    position: Vec3,
    time_stamp: f64,
}

#[derive(Debug, Clone)]
pub struct KeyRotation {
    orientation: Quat,
    time_stamp: f64,
}

#[derive(Debug, Clone)]
pub struct KeyScale {
    scale: Vec3,
    time_stamp: f64,
}

#[derive(Debug, Clone)]
pub struct AniBone {
    positions: Vec<KeyPosition>,
    rotations: Vec<KeyRotation>,
    scales: Vec<KeyScale>,

    pub local_transform: Mat4,
    pub name: String,
    id: usize,
}

impl AniBone {
    pub fn new(name: String, id: usize, channel: &NodeAnim) -> Self {
        let mut positions = Vec::new();
        for position_key in channel.position_keys.iter() {
            let ai_position = position_key.value;

            let data = KeyPosition {
                position: Vec3::new(ai_position.x, ai_position.y, ai_position.z),
                time_stamp: position_key.time,
            };

            positions.push(data);
        };
        write_data(positions.clone(), "positions.txt");

        let mut rotations = channel.rotation_keys
            .iter()
            .filter(|rotation_key| {
                let time = rotation_key.time;

                // Reject subnormal numbers (extremely tiny values like 5e-324)
                if time.abs() < 1e-300 {
                    return false;
                }

                // Check if it is a perfect multiple of 3
                let multiple_of_3 = (time / 3.0).round() == (time / 3.0);

                multiple_of_3
            })
            .map(|rotation_key| KeyRotation {
                orientation: Quat::from_xyzw(rotation_key.value.x, rotation_key.value.y, rotation_key.value.z, rotation_key.value.w),
                time_stamp: rotation_key.time,
            })
            .collect::<Vec<_>>();

        // let mut rotations = Vec::new();
        // for rotation_key in channel.rotation_keys.iter() {
        //     let ai_quat = rotation_key.value;

        //     let data = KeyRotation {
        //         orientation: Quat::from_xyzw(ai_quat.x, ai_quat.y, ai_quat.z, ai_quat.w),
        //         time_stamp: rotation_key.time,
        //     };
        //     
        //     rotations.push(data);
        // }

        write_data(rotations.clone(), "rotations.txt");
        write_data(channel.rotation_keys.clone(), "channel_rots.txt");
        write_data(channel.clone(), "channel.txt");

        let mut scales = Vec::new();
        for scale_key in channel.scaling_keys.iter() {
            let scale = scale_key.value;
            
            let data = KeyScale {
                scale: Vec3::new(scale.x, scale.y, scale.z),
                time_stamp: scale_key.time,
            };

            scales.push(data);
        }
        write_data(scales.clone(), "scales.txt");

        Self {
            positions,
            rotations,
            scales,

            local_transform: Mat4::IDENTITY,
            name,
            id,
        }
    }


    pub fn update(&mut self, animation_time: f64) {
        let translation = self.interpolate_position(animation_time);
        let rotation = self.interpolate_rotation(animation_time);
        let scale = self.interpolate_scaling(animation_time);

        self.local_transform = translation * rotation * scale;
    }

    pub fn get_scale_factor(last_time_stamp: f64, next_time_stamp: f64, animation_time: f64) -> f64 {
        let midway_len = animation_time - last_time_stamp;
        let frame_diff = next_time_stamp - last_time_stamp;

        midway_len / frame_diff
    }
    
    pub fn get_position_index(&mut self, animation_time: f64) -> usize {
        for i in 0..self.positions.len() - 1 {
            if let Some(pos) = self.positions.get(i + 1) {
                if animation_time < pos.time_stamp {
                    return i;
                }
            }
        }
        panic!("We shouldn't have gotten here");
    }

    pub fn get_rotation_index(&mut self, animation_time: f64) -> usize {
        for i in 0..self.rotations.len().saturating_sub(1) {
            if let Some(rot) = self.rotations.get(i + 1) {
                if animation_time < rot.time_stamp {
                    return i;
                }
            }
        }
        
        // return self.rotations.len() - 2;
        panic!("By rights we shouldn't even be here");
    }

    pub fn get_scale_index(&mut self, animation_time: f64) -> usize {
        for i in 0..self.scales.len() - 1 {
            if let Some(scale) = self.scales.get(i + 1) {
                if animation_time < scale.time_stamp {
                    return i;
                }
            }
        }
        panic!("We shouldn't have gotten here");
    }

    pub fn interpolate_position(&mut self, animation_time: f64) -> Mat4 {
        if 1 == self.positions.len() {
            return Mat4::from_translation(self.positions.first().unwrap().position);
        }

        // TODO: THIS FEELS VERY FUCKING DUMB AND HACKY LET'S FIND A BETTER WAY
        let p0_index = self.get_position_index(animation_time);
        
        let p0 = self.positions.get(p0_index).unwrap();
        let p1 = self.positions.get(p0_index + 1).unwrap();

        let scale_factor = Self::get_scale_factor(p0.time_stamp, p1.time_stamp, animation_time);
        let final_position = p0.position.lerp(p1.position, scale_factor as f32);
        return Mat4::from_translation(final_position);
    }

    pub fn interpolate_rotation(&mut self, animation_time: f64) -> Mat4 {
        if self.rotations.len() == 1 {
            let rotation = self.rotations[0].orientation;
            return Mat4::from_quat(rotation);
        }

        let p0_index = self.get_rotation_index(animation_time);

        let p0 = &self.rotations[p0_index];
        let p1 = &self.rotations[p0_index + 1];

        let scale_factor = Self::get_scale_factor(p0.time_stamp, p1.time_stamp, animation_time);
        let final_rotation = p0.orientation.slerp(p1.orientation, scale_factor as f32);
        Mat4::from_quat(final_rotation)
    }

    pub fn interpolate_scaling(&mut self, animation_time: f64) -> Mat4{
        if 1 == self.scales.len() {
            return Mat4::from_scale(self.scales.first().unwrap().scale);
        }

        let p0_index = self.get_scale_index(animation_time);

        let p0 = self.scales.get(p0_index).unwrap();
        let p1 = self.scales.get(p0_index + 1).unwrap();

        let scale_factor = Self::get_scale_factor(p0.time_stamp, p1.time_stamp, animation_time);
        let final_scale = p0.scale.lerp(p1.scale, scale_factor as f32);
        return Mat4::from_scale(final_scale);
    }
}
