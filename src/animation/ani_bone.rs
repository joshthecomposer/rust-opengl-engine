use glam::{Mat4, Quat, Vec3};
use russimp::animation::NodeAnim;

pub struct KeyPosition {
    position: Vec3,
    time_stamp: f64,
}

pub struct KeyRotation {
    orientation: Quat,
    time_stamp: f64,
}
pub struct KeyScale {
    scale: Vec3,
    time_stamp: f64,
}

pub struct AniBone {
    positions: Vec<KeyPosition>,
    rotations: Vec<KeyRotation>,
    scales: Vec<KeyScale>,

    local_transform: Mat4,
    name: String,
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

        let mut rotations = Vec::new();
        for rotation_key in channel.rotation_keys.iter() {
            let ai_quat = rotation_key.value;

            let data = KeyRotation {
                orientation: Quat::from_xyzw(ai_quat.x, ai_quat.y, ai_quat.z, ai_quat.w),
                time_stamp: rotation_key.time,
            };

            rotations.push(data);
        }

        let mut scales = Vec::new();
        for scale_key in channel.scaling_keys.iter() {
            let scale = scale_key.value;
            
            let data = KeyScale {
                scale: Vec3::new(scale.x, scale.y, scale.z),
                time_stamp: scale_key.time,
            };

            scales.push(data);
        }

        Self {
            positions,
            rotations,
            scales,

            local_transform: Mat4::IDENTITY,
            name,
            id,
        }
    }


    pub fn update(&mut self, animation_time: f32) {
    }

    pub fn get_scale_factor(last_time_stamp: f32, next_time_stamp: f32, animation_time: f32) {
    }

    pub fn interpolate_position(animation_time: f32) {
    }

    pub fn interpolate_rotation(animation_time: f32) {
    }

    pub fn interpolate_scale(animation_time: f32) {
    }
}
