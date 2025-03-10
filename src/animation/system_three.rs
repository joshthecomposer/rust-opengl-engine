use std::{collections::HashMap, str::Lines};

use glam::{Mat4, Quat, Vec2, Vec3};

use crate::some_data::MAX_BONE_INFLUENCE;

#[repr(C)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,

    bone_ids: [u32; MAX_BONE_INFLUENCE],
    bone_weights: [f32; MAX_BONE_INFLUENCE],
}

pub struct Model {
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,

    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct Bone {
    // id will be the position in the final bone array as well.
    id: u32,
    // TODO: This field is completely useless, it might be better to make a hashmap temporarily in the parse function
    // and avoid this field altogether.
    parent_index: i32,
    name: String,
    offset: Mat4,
    children: Vec<Bone>,
}

pub struct BoneTransformTrack {
    position_timestamps: Vec<f32>,
    rotation_timestamps: Vec<f32>,
    scale_timestamps: Vec<f32>,

    positions: Vec3,
    rotations: Quat,
    scales: Vec3,
}

pub struct Animation {
    duration: f32,
    ticks_per_second: f32,
    // Bone name is the key.
    // It makes sense to keep this away from the bone because we could have multiple animations
    bone_transforms: HashMap<String, BoneTransformTrack>
}

pub fn import_bone_data(file_path: &str) -> Bone {
    let data = std::fs::read_to_string(file_path).unwrap();
    let mut lines = data.lines();
    
    let mut bones_no_children = Vec::new();
    let mut bone_count = 0;

    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "WiseModel" => {
                // name = "DefaultAnimation".to_string();
            }
            "BONECOUNT:" => {
                // bone_count = parts[1].parse::<usize>().unwrap();
            }
            "BONE_NAME:" => {
                let name = parts[1].to_string();
                let parent_index: i32 = lines.next().unwrap().split_whitespace().collect::<Vec<&str>>()[1].parse().unwrap();
                lines.next();
                let offset = parse_bone_offset(&mut lines);

                bones_no_children.push(Bone {
                    id: bone_count,
                    parent_index,
                    name,
                    offset,
                    children: vec![],
                });
            }
            _ => {}
        }
    }

    unimplemented!()
}

fn parse_bone_offset(lines: &mut Lines<'_>) -> Mat4 {
    let x_axis: Vec<&str> = lines.next();
    
    unimplemented!()
}

fn parse_vec3(input: &str) {

}

pub fn import_model_data() -> Model {
    // Import the vertices and indices. Create vao, vbo, ebo with OpenGL.
    unimplemented!()
}

pub fn pose_bones_recursively() {
}
