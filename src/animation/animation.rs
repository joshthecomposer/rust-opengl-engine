#![allow(dead_code)]
use std::{collections::HashMap, ffi::c_void, path::Path, ptr, str::Lines};

use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use image::GenericImageView;
use russimp::material::TextureType;
use std::mem::{self, offset_of};

use crate::{debug::write::write_data, gl_call, mesh::Texture, shaders::Shader, some_data::MAX_BONE_INFLUENCE};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct AniVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,

    pub bone_ids: [i32; MAX_BONE_INFLUENCE],
    pub bone_weights: [f32; MAX_BONE_INFLUENCE],
}

#[derive(Debug, Clone)]
pub struct AniModel {
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,

    pub vertices: Vec<AniVertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,

    pub directory: String,
}

impl AniModel {
    pub fn new() -> Self {
        Self {
            vao: 0,
            vbo: 0,
            ebo: 0,

            vertices: vec![],
            indices: vec![],
            textures: vec![],

            directory: String::new(),
        }
    }

    pub fn setup_opengl(&mut self) {
        unsafe {
            gl_call!(gl::GenVertexArrays(1, &mut self.vao));
            gl_call!(gl::GenBuffers(1, &mut self.vbo));
            gl_call!(gl::GenBuffers(1, &mut self.ebo));

            gl_call!(gl::BindVertexArray(self.vao));
            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo));

            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER, 
                (mem::size_of::<AniVertex>() * self.vertices.len()) as isize,
                self.vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            ));

            gl_call!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo));
            gl_call!(gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mem::size_of::<u32>() * self.indices.len()) as isize,
                self.indices.as_ptr().cast(),
                gl::STATIC_DRAW
            ));

            gl_call!(gl::EnableVertexAttribArray(0));
            gl_call!(gl::VertexAttribPointer(
                0, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<AniVertex>() as i32,
                ptr::null(),
            ));

            gl_call!(gl::EnableVertexAttribArray(1));
            gl_call!(gl::VertexAttribPointer(
                1, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<AniVertex>() as i32,
                offset_of!(AniVertex, normal) as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(2));
            gl_call!(gl::VertexAttribPointer(
                2, 
                2, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<AniVertex>() as i32, 
                offset_of!(AniVertex, uv) as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(3));
            gl_call!(gl::VertexAttribIPointer( 
                3,
                4,
                gl::INT,
                mem::size_of::<AniVertex>() as i32,
                offset_of!(AniVertex, bone_ids) as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(4));
            gl_call!(gl::VertexAttribPointer(
                4,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<AniVertex>() as i32,
                offset_of!(AniVertex, bone_weights) as *const _
            ));

            gl::BindVertexArray(0);
        }
    }

    pub fn draw(&self, shader: &mut Shader) {
        shader.activate();

        let mut diffuse_nr = 1;
        let mut specular_nr = 1;

        for (i, texture) in self.textures.iter().enumerate() {
            unsafe {
                gl_call!(gl::ActiveTexture(gl::TEXTURE0 + i as u32));
            }

            let mut number = "".to_string();
            let name = &texture._type;
            if name == "texture_diffuse" {
                number = diffuse_nr.to_string();

                diffuse_nr += 1;
            } else if name == "texture_specular" {
                number = specular_nr.to_string();
                specular_nr += 1;
            }

            let final_str = ("material.".to_string() + name.as_str()) + number.as_str();
            shader.store_uniform_location(final_str.as_str());
            shader.set_int(final_str.as_str(), i as u32);

            unsafe {
                gl_call!(gl::BindTexture(gl::TEXTURE_2D, texture.id));
            }
        }

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

    // pub fn shadow_pass(shader: &mut Shader) {
    // }
}

#[derive(Debug, Clone)]
pub struct Bone {
    // id will be the position in the final bone array as well.
    id: u32,
    parent_index: Option<u32>,
    name: String,
    offset: Mat4,
    children: Vec<Bone>,
}

#[derive(Debug, Clone)]
pub struct BoneJoinInfo {
    name: String,
    // offset: Mat4,
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

pub struct Animator {
    pub current_animation: String,
    pub animations: HashMap<String, Animation>,
}

impl Animator {
    pub fn new(animation: Animation) -> Self {
        let mut animations = HashMap::new();
        animations.insert("Run".to_string(), animation);
        Self {
            current_animation: "Run".to_string(),
            animations,
        }
    }

    pub fn set_current_animation(&mut self, input: &str) {
        self.current_animation = input.to_string();
    }

    pub fn update(&mut self, elapsed_time: f32, skellington: &mut Bone) {
        if let Some(animation) = &mut self.animations.get_mut(&self.current_animation) {
            animation.update(elapsed_time, skellington);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    duration: f32,
    ticks_per_second: f32,
    model_animation_join: Vec<BoneJoinInfo>,
    bone_transforms: HashMap<String, BoneTransformTrack>,
    pub current_pose: Vec<Mat4>,
}

impl Animation {
    pub fn default() -> Self {
        Self {
            duration: 0.0,
            ticks_per_second: 0.0,
            model_animation_join: vec![],
            bone_transforms: HashMap::new(),
            current_pose: vec![],
        }
    }

    pub fn calculate_pose(
        &mut self,
        skeleton: &mut Bone,
        elapsed_time: f32,
        parent_transform: Mat4,
        global_inverse_transform: Mat4,
    ) {
        let btt = self.bone_transforms.get(&skeleton.name).unwrap();
        let dt = elapsed_time % self.duration;

        let (segment, fraction) = get_time_fraction(&btt.position_timestamps, dt);

        let local_transform = if segment == 0 {
            // Use the first keyframe
            let position = btt.positions[0];
            let rotation = btt.rotations[0];
            let scale = btt.scales[0];
            Mat4::from_scale_rotation_translation(scale, rotation, position)
        } else {
            // Get the two keyframes to interpolate between
            let prev_idx = segment - 1;
            let next_idx = segment.min(btt.positions.len() as u32 - 1); // Prevent out-of-bounds

            let prev_position = btt.positions[prev_idx as usize];
            let next_position = btt.positions[next_idx as usize];

            let prev_rotation = btt.rotations[prev_idx as usize];
            let next_rotation = btt.rotations[next_idx as usize];

            let prev_scale = btt.scales[prev_idx as usize];
            let next_scale = btt.scales[next_idx as usize];

            // Perform linear interpolation for position and scale
            let interpolated_position = prev_position.lerp(next_position, fraction);
            let interpolated_scale = prev_scale.lerp(next_scale, fraction);

            // Perform spherical interpolation (slerp) for rotation
            let interpolated_rotation = prev_rotation.slerp(next_rotation, fraction);

            Mat4::from_scale_rotation_translation(interpolated_scale, interpolated_rotation, interpolated_position)
        };

        let global_transform = parent_transform * local_transform;

        self.current_pose[skeleton.id as usize] =
            match 0 {
                0 => global_inverse_transform * global_transform * skeleton.offset,
                1 => global_transform * skeleton.offset * global_inverse_transform,
                2 => global_transform * skeleton.offset,
                3 => global_inverse_transform * global_transform,
                4 => global_inverse_transform * skeleton.offset * global_transform,
                5 => global_transform * global_inverse_transform * skeleton.offset,
                10 => Mat4::IDENTITY,
                _ => global_inverse_transform * global_transform * skeleton.offset,
            };

        for child in skeleton.children.iter_mut() {
            self.calculate_pose(child, dt, global_transform, global_inverse_transform);
        }
    }

    pub fn update(&mut self, elapsed_time: f32, skellington: &mut Bone) {
        self.calculate_pose(
            skellington, 
            elapsed_time, 
            Mat4::IDENTITY,
            Mat4::IDENTITY, 
        );
    }
}

pub fn get_time_fraction(times: &[f32], dt: f32) -> (u32, f32) {
    let mut segment = 0;

    while dt > times[segment] {
        segment += 1;
    }

    if segment == 0 {
        return (0, 0.0); // Avoid accessing times[-1], return first segment with no interpolation
    }

    let start = times[segment - 1];
    let end = times[segment];
    let frac = (dt - start) / (end - start);

    (segment as u32, frac)
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
                let parsed_parent: i32 = lines.next().unwrap().split_whitespace().collect::<Vec<&str>>()[1].parse().unwrap();

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
    // =============================================================
    // Get Animation Data
    // ============================================================
    lines = data.lines();
    let mut animation = Animation::default();

    // Get gpu bone info to use for later to gather a final matrix array
    let mut model_animation_join = vec![];

    for b in bones_no_children {
        model_animation_join.push(
            BoneJoinInfo {
                name: b.name.clone(),
                // offset: b.offset,
            }
        );

        animation.current_pose.push(b.offset);

        assert!(model_animation_join[b.id as usize].name == b.name);
        assert!(model_animation_join.len() == animation.current_pose.len());
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
                    let bone_name = model_animation_join[i as usize].name.clone();

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

    animation.model_animation_join = model_animation_join;

    write_data(&animation, "animation_out.txt");

    (bone, animation)
}

pub fn import_model_data(file_path: &str, animation: &Animation) -> AniModel {
    let data = std::fs::read_to_string(file_path).unwrap();
    let mut lines = data.lines();

    let mut model = AniModel::new();

    let directory = Path::new(file_path).parent().unwrap().to_str().unwrap();
    println!("Directory of AniModel is: {}", &directory);
    println!("=============================================================");

    model.directory = directory.to_string();

    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "MEME" => {}
            "VERT:" => {
                let mut vertex = AniVertex {
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

                    for (j, info) in animation.model_animation_join.iter().enumerate() {
                        if info.name == bone_name {
                            bone_id = j as i32;
                        }
                    }

                    vertex.bone_ids[i] = bone_id;
                    vertex.bone_weights[i] = weight;

                    let total_weight = vertex.bone_weights.iter().sum::<f32>();
                    if total_weight > 0.0 {
                        for w in vertex.bone_weights.iter_mut() {
                            *w /= total_weight;
                        }
                    }
                }

                model.vertices.push(vertex);
            }
            "INDEX_COUNT:" => {
                let index_count: u32 = parts[1].parse().unwrap();
                let indices: Vec<u32> = lines.next().unwrap().split_whitespace().map(|n| n.parse().unwrap()).collect();
                
                dbg!(indices.len());
                dbg!(index_count);
                assert!(index_count == indices.len() as u32);
                model.indices = indices;
            }
            "TEXTURE:" => {
                let path = parts[1].to_string();
                texture_from_file(&mut model, path);
            }
            _ => {}
        }
    }

    model
}

fn texture_from_file(model: &mut AniModel, path: String) {
    println!("texture is {}", &path);
    let file_name = model.directory.clone() + "/" + path.as_str();

    dbg!(&path);
    dbg!(&file_name);

    let mut texture_id = 0;
    unsafe {
        gl_call!(gl::GenTextures(1, &mut texture_id));

        let img = image::open(file_name.clone()).unwrap();
        let (img_width, img_height) = img.dimensions();
        let rgba = img.to_rgba8();
        let raw = rgba.as_raw();

        gl_call!(gl::BindTexture(gl::TEXTURE_2D, texture_id));
        gl_call!(gl::TexImage2D(
            gl::TEXTURE_2D, 
            0, 
            gl::RGBA as i32, 
            img_width as i32, 
            img_height as i32, 
            0, 
            gl::RGBA, 
            gl::UNSIGNED_BYTE, 
            raw.as_ptr() as *const c_void
        ));

        gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32));
        gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32));
        gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32));
        gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32));
        gl_call!(gl::GenerateMipmap(gl::TEXTURE_2D));
    }

    let texture = Texture {
        id: texture_id,
        _type: "texture_diffuse".to_string(),
        path: file_name,
    };

    model.textures.push(texture);
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
        parts[0].parse::<f32>().unwrap(),
        parts[1].parse::<f32>().unwrap(),
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

fn build_bone_hierarchy_top_down(bones: Vec<Bone>) -> Bone {
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
