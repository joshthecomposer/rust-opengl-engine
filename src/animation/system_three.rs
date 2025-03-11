use std::{collections::HashMap, ptr, str::Lines};

use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use std::{ffi::CString, mem::{self, offset_of}};

use crate::{debug::write::write_data, gl_call, shaders::Shader, some_data::MAX_BONE_INFLUENCE};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,

    pub bone_ids: [i32; MAX_BONE_INFLUENCE],
    pub bone_weights: [f32; MAX_BONE_INFLUENCE],
}

#[derive(Debug, Clone)]
pub struct Model {
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,

    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            vao: 0,
            vbo: 0,
            ebo: 0,

            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn setup_opengl(&mut self) {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::GenBuffers(1, &mut self.vbo);
            gl::GenBuffers(1, &mut self.ebo);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER, 
                (mem::size_of::<Vertex>() * self.vertices.len()) as isize,
                self.vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mem::size_of::<u32>() * self.indices.len()) as isize,
                self.indices.as_ptr().cast(),
                gl::STATIC_DRAW
            );

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<Vertex>() as i32,
                ptr::null(),
            );

            gl_call!(gl::EnableVertexAttribArray(1));
            gl_call!(gl::VertexAttribPointer(
                1, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, normal) as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(2));
            gl_call!(gl::VertexAttribPointer(
                2, 
                2, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<Vertex>() as i32, 
                offset_of!(Vertex, uv) as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(3));
            gl_call!(gl::VertexAttribIPointer( 
                3,
                4,
                gl::INT,
                mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, bone_ids) as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(4));
            gl_call!(gl::VertexAttribPointer(
                4,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, bone_weights) as *const _
            ));

            gl::BindVertexArray(0);
        }
    }

    pub fn draw(&self, shader: &mut Shader) {
        unsafe {
            gl_call!(gl::BindVertexArray(self.vao));
             gl_call!(gl::DrawElements(
                gl::TRIANGLES, 
                self.indices.len() as i32, 
                gl::UNSIGNED_INT, 
                ptr::null(), 
            ));

            gl_call!(gl::BindVertexArray(0));
        }
    }

    pub fn shadow_pass(shader: &mut Shader) {
    }
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

#[derive(Debug, Clone)]
pub struct GPUBoneInfo {
    name: String,
    offset: Mat4,
}

#[derive(Debug, Clone)]
pub struct BoneTransformTrack {
    position_timestamps: Vec<f32>,
    rotation_timestamps: Vec<f32>,
    scale_timestamps: Vec<f32>,

    positions: Vec<Vec3>,
    rotations: Vec<Quat>,
    scales: Vec<Vec3>,
}

impl BoneTransformTrack {
    pub fn default() -> Self {
        Self {
            position_timestamps: vec![],
            rotation_timestamps: vec![],
            scale_timestamps: vec![],

            positions: vec![],
            rotations: vec![],
            scales: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    duration: f32,
    ticks_per_second: f32,
    gpu_bone_info: Vec<GPUBoneInfo>,
    // Bone name is the key.
    // It makes sense to keep this away from the bone because we could have multiple animations
    bone_transforms: HashMap<String, BoneTransformTrack>
}

impl Animation {
    pub fn default() -> Self {
        Self {
            duration: 0.0,
            ticks_per_second: 0.0,
            gpu_bone_info: vec![],
            bone_transforms: HashMap::new(),
        }
    }
}

pub fn import_bone_data(file_path: &str) -> (Bone, Animation) {
    let data = std::fs::read_to_string(file_path).unwrap();
    let mut lines = data.lines();
    
    let mut bones_no_children = Vec::new();
    let mut bone_idx = 0;
    let mut bone_count: u32 = 0;


    // =============================================================
    // Get Starting Bones
    // ============================================================
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
                bone_count = parts[1].parse().unwrap();
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
                    id: bone_idx,
                    parent_index,
                    name,
                    offset,
                    children: vec![],
                });
                
                bone_idx += 1;
            }
            _ => {}
        }
    }

    let bone = build_bone_hierarchy_top_down(bones_no_children.clone());
    write_data(&bone, "skellington.txt");

    // =============================================================
    // Get Animation Data
    // ============================================================
    lines = data.lines();
    let mut animation = Animation::default();

    // Get gpu bone info to use for later to gather a final matrix array
    let mut gpu_bone_info = vec![];

    for b in bones_no_children {
        gpu_bone_info.push(
            GPUBoneInfo {
                name: b.name.clone(),
                offset: b.offset,
            }
        );

        assert!(gpu_bone_info[b.id as usize].name == b.name);
    }

    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "DURATION:" => {
                animation.duration = parts[1].parse().unwrap()
            }
            "FPS:" => {
                animation.ticks_per_second = parts[1].parse().unwrap()
            }
            "TIMESTAMP:" => {
                let time_stamp = parts[1].parse().unwrap();

                for i in 0..bone_count {
                    let bone_name = gpu_bone_info[i as usize].name.clone();

                    let track = animation
                        .bone_transforms
                        .entry(bone_name)
                        .or_insert_with(BoneTransformTrack::default);

                    track.position_timestamps.push(time_stamp);
                    track.rotation_timestamps.push(time_stamp);
                    track.scale_timestamps.push(time_stamp);

                    let position = parse_vec3(lines.next().unwrap());
                    let rotation = parse_quat(lines.next().unwrap());
                    let scale = parse_vec3(lines.next().unwrap());

                    track.positions.push(position);
                    track.rotations.push(rotation);
                    track.scales.push(scale);

                    lines.next();
                }

            }
            _ => {}
        }
    }

    animation.gpu_bone_info = gpu_bone_info;

    write_data(&animation, "animation_out.txt");

    (bone, animation)
}

pub fn import_model_data(file_path: &str, animation: &Animation) -> Model {
    let data = std::fs::read_to_string(file_path).unwrap();
    let mut lines = data.lines();

    let mut model = Model::new();

    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "MEME" => {}
            "VERT:" => {
                let mut vertex = Vertex {
                    position: parse_vec3(lines.next().unwrap()),
                    normal: parse_vec3(lines.next().unwrap()),
                    uv: parse_vec2(lines.next().unwrap()),
                    bone_ids: [-1; MAX_BONE_INFLUENCE],
                    bone_weights: [0.0; MAX_BONE_INFLUENCE],
                };

                let weight_parts: Vec<&str> = lines.next().unwrap().split_whitespace().collect();

                for (i, pair) in weight_parts.chunks(2).enumerate() {
                    let bone_name = pair[0];
                    let weight: f32 = pair[1].parse().unwrap_or(0.0);

                    let mut bone_id: i32 = -1;

                    for (j, info) in animation.gpu_bone_info.iter().enumerate() {
                        if info.name == bone_name {
                            bone_id = j as i32;
                        }
                    }

                    vertex.bone_ids[i] = bone_id;
                    vertex.bone_weights[i] = weight;
                }
                
                model.vertices.push(vertex);
            }
            "INDEX_COUNT:" => {
                let index_count: u32 = parts[1].parse().unwrap();
                let indices: Vec<u32> = lines.next().unwrap().split_whitespace().map(|n| n.parse().unwrap()).collect();

                assert!(index_count == indices.len() as u32);
                model.indices = indices;
            }
            _ => {}
        }
    }

    model
}

fn parse_bone_offset(lines: &mut Lines<'_>) -> Mat4 {
    Mat4 {
        x_axis: parse_vec4(lines.next().unwrap()),
        y_axis: parse_vec4(lines.next().unwrap()),
        z_axis: parse_vec4(lines.next().unwrap()),
        w_axis: parse_vec4(lines.next().unwrap()),
    }
}

fn parse_vec4(input: &str) -> Vec4 {
    let parts: Vec<&str> = input.split_whitespace().collect();
    Vec4::new( 
        parts[0].parse().unwrap(),
        parts[1].parse().unwrap(),
        parts[2].parse().unwrap(),
        parts[3].parse().unwrap(),
    )
}

fn parse_vec3(input: &str) -> Vec3 {
    let parts: Vec<&str> = input.split_whitespace().collect();
    Vec3::new( 
        parts[0].parse().unwrap(),
        parts[1].parse().unwrap(),
        parts[2].parse().unwrap(),
    )
}

fn parse_vec2(input: &str) -> Vec2 {
    let parts: Vec<&str> = input.split_whitespace().collect();
    Vec2::new( 
        parts[0].parse().unwrap(),
        parts[1].parse().unwrap(),
    )
}

fn parse_quat(input: &str) -> Quat {
    let parts: Vec<&str> = input.split_whitespace().collect();
    Quat::from_xyzw(
        parts[0].parse().unwrap(),
        parts[1].parse().unwrap(),
        parts[2].parse().unwrap(),
        parts[3].parse().unwrap(),
    )
}

fn build_bone_hierarchy_top_down(mut bones: Vec<Bone>) -> Bone {
    let mut children_of = vec![Vec::new(); bones.len()];

    for bone in &bones {
        if let Some(parent_id) = bone.parent_index {
            children_of[parent_id as usize].push(bone.id);
        }
    }
  
    let root_id = bones
        .iter()
        .find(|b| b.parent_index.is_none())
        .expect("No root bone found!")
        .id;
  
    build_tree_node(root_id, &bones, &children_of)
}

fn build_tree_node(
    bone_id: u32,
    bones: &[Bone],
    children_of: &[Vec<u32>],
) -> Bone {
    let original = &bones[bone_id as usize];
    let mut node = Bone {
        id: original.id,
        parent_index: original.parent_index,
        name: original.name.clone(),
        offset: original.offset,
        children: Vec::new(),
    };

    for &child_id in &children_of[bone_id as usize] {
        let child = build_tree_node(child_id, bones, children_of);
        node.children.push(child);
    }

    node
}

pub fn pose_bones_recursively() {
}
