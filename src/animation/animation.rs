use std::{collections::HashMap, fs};

use glam::{vec3, Mat4, Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Bone {
    pub name: String,
    pub parent_index: i32, // -1 means root
    pub offset_matrix: Mat4, // Model-space transformation
    pub bone_id: usize,
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub frame_index: usize,
    pub position: Vec3, // Bone translation
    pub rotation: Quat, // Bone rotation (quaternion)
    pub scale: Vec3,    // Bone scale
}

#[derive(Debug, Clone)]
pub struct BoneAnimation {
    pub bone_name: String, // Matches BoneInfo name
    pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    // TODO: We don't need a name for the animatioin clip
    pub name: String,
    pub total_frames: usize,
    pub fps: f32,
    pub bone_animations: Vec<BoneAnimation>, // Animation per bone
    pub duration: f32,
}

pub fn load_wise_animation(file_path: &str) -> AnimationClip {
    let data = std::fs::read_to_string(file_path).unwrap();
    let mut lines = data.lines();

    let mut bone_animations = Vec::new();
    let mut fps = 0.0;
    let mut total_frames = 0;
    let mut name = String::new();
    let mut bone_count = 0;

    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "WiseModel" => {
                name = "DefaultAnimation".to_string();
            }
            "FPS:" => {
                fps = parts[1].parse::<f32>().unwrap();
                assert!(fps > 0.0);
            }
            "BONECOUNT:" => {
                bone_count = parts[1].parse::<usize>().unwrap();
            }
            "BONE_NAME:" => {
                let bone_name = parts[1].to_string();
                bone_animations.push(BoneAnimation {
                    bone_name,
                    keyframes: Vec::new(),
                });
            }
            _ => {}
        }
    }

    // Reset the iterator to read keyframes
    let mut lines = data.lines();
    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        if parts[0] == "KEYFRAME:" {
            let frame_index: usize = parts[1].parse().unwrap();

            for bone_index in 0..bone_count {
                let position: Vec3 = parse_vec3(lines.next().unwrap());
                let rotation: Quat = parse_quat(lines.next().unwrap());
                let scale: Vec3 = parse_vec3(lines.next().unwrap());

                let keyframe = Keyframe {
                    frame_index,
                    position,
                    rotation,
                    scale,
                };

                if bone_index < bone_animations.len() {
                    bone_animations[bone_index].keyframes.push(keyframe);
                }

                lines.next();
            }

            total_frames = total_frames.max(frame_index + 1);
        }
    }

    let duration = total_frames as f32 / fps;

    AnimationClip {
        name,
        total_frames,
        fps,
        bone_animations,
        duration,
    }
}

fn parse_vec3(line: &str) -> Vec3 {
    let parts: Vec<f32> = line.split_whitespace().map(|v| v.parse().unwrap()).collect();
    vec3(parts[0], parts[1], parts[2])
}

fn parse_quat(line: &str) -> Quat {
    let parts: Vec<f32> = line.split_whitespace().map(|v| v.parse().unwrap()).collect();
    Quat::from_xyzw(parts[1], parts[2], parts[3], parts[0]) // Order (w, x, y, z)
}
