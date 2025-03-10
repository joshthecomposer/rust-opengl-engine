use std::{collections::HashMap, str::Lines};

use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{debug::write::write_data, some_data::MAX_BONE_INFLUENCE};

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

#[derive(Debug, Clone)]
pub struct Bone {
    // id will be the position in the final bone array as well.
    id: u32,
    // TODO: This field is completely useless, it might be better to make a hashmap temporarily in the parse function
    // and avoid this field altogether.
    parent_index: Option<u32>,
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
                let mut parsed_parent: i32 = lines.next().unwrap().split_whitespace().collect::<Vec<&str>>()[1].parse().unwrap();
                
                let parent_index = match parsed_parent {
                    -1 => None,
                    _ => Some(parsed_parent as u32),
                };

                lines.next();
                let offset = parse_bone_offset(&mut lines);

                bones_no_children.push(Bone {
                    id: bone_count,
                    parent_index,
                    name,
                    offset,
                    children: vec![],
                });
                
                bone_count += 1;
            }
            _ => {}
        }
    }

    let bone = populate_bone_children(bones_no_children);

    write_data(bone, "bones_no_children.txt");

    unimplemented!()
}

fn parse_bone_offset(lines: &mut Lines<'_>) -> Mat4 {
    let x_axis: Vec<&str> = lines.next().unwrap().split_whitespace().collect();
    let y_axis: Vec<&str> = lines.next().unwrap().split_whitespace().collect();
    let z_axis: Vec<&str> = lines.next().unwrap().split_whitespace().collect();
    let w_axis: Vec<&str> = lines.next().unwrap().split_whitespace().collect();

    Mat4 {
        x_axis: parse_vec4(x_axis),
        y_axis: parse_vec4(y_axis),
        z_axis: parse_vec4(z_axis),
        w_axis: parse_vec4(w_axis),
    }
}

fn parse_vec4(input: Vec<&str>) -> Vec4 {
    Vec4::new( 
        input[0].parse().unwrap(),
        input[1].parse().unwrap(),
        input[2].parse().unwrap(),
        input[3].parse().unwrap(),
    )
}

fn populate_bone_children(bones: Vec<Bone>) -> Bone {
    let mut by_id: HashMap<u32, Bone> =
    bones.into_iter().map(|b| (b.id, b)).collect();

    // Find the single root bone (assuming your data has exactly one root)
    let root_id = by_id
        .values()
        .find(|bone| bone.parent_index.is_none())
        .expect("No root bone found!")
    .id;

    // We have to collect IDs in advance so we can mutate the map inside the loop.
    let all_ids: Vec<u32> = by_id.keys().cloned().collect();

    // Attach each child bone to its parent's `children` Vec
    for child_id in all_ids {
        // If the bone has a parent, move it out of `by_id` and into its parent's `children`.
        if let Some(parent_id) = by_id[&child_id].parent_index {
            let child_bone = by_id
                .remove(&child_id)
                .expect("Bone vanished from map unexpectedly!");

            by_id
                .get_mut(&parent_id)
                .expect("Parent bone not found in map!")
                .children
                .push(child_bone);
        }
    }

    // At this point, `by_id` should only contain the root (and anything else that lacked a valid parent).
    by_id
        .remove(&root_id)
        .expect("Root bone vanished from map?")
}

pub fn import_model_data() -> Model {
    // Import the vertices and indices. Create vao, vbo, ebo with OpenGL.
    unimplemented!()
}

pub fn pose_bones_recursively() {
}
