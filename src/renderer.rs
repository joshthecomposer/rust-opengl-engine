use std::{collections::HashMap, ffi::c_void, mem, ptr::null_mut};

use glam::{vec3, vec4, Mat4};
use image::GenericImageView;

use crate::{camera::Camera, entity_manager::EntityManager, enums_types::{FboType, ShaderType, VaoType}, gl_call, grid::Grid, lights::Lights, shaders::Shader, some_data::{FACES_CUBEMAP, POINT_LIGHT_POSITIONS, SHADOW_HEIGHT, SHADOW_WIDTH, SKYBOX_INDICES, SKYBOX_VERTICES, UNIT_CUBE_VERTICES}};

pub struct Renderer {
    pub shaders: HashMap<ShaderType, Shader>, // TODO: make this an enum
    pub vaos: HashMap<VaoType, u32>,
    pub fbos: HashMap<FboType, u32>,
    pub depth_map: u32,
    pub cubemap_texture: u32,
}

impl Renderer {
    pub fn new() -> Self {
        // =============================================================
        // Setup Shaders
        // =============================================================

        let mut shaders = HashMap::new();
        let mut vaos = HashMap::new();
        let mut fbos = HashMap::new();

        let mut skybox_shader = Shader::new("resources/shaders/skybox.vs", "resources/shaders/skybox.fs");

        skybox_shader.store_uniform_location("view");
        skybox_shader.store_uniform_location("projection");
        skybox_shader.store_uniform_location("skybox");
        
        let mut debug_light_shader = Shader::new("resources/shaders/point_light.vs", "resources/shaders/point_light.fs");
        debug_light_shader.store_uniform_location("model");
        debug_light_shader.store_uniform_location("view");
        debug_light_shader.store_uniform_location("projection");
        debug_light_shader.store_uniform_location("LightColor");

        let mut depth_shader = Shader::new("resources/shaders/depth_shader.vs","resources/shaders/depth_shader.fs");
        depth_shader.store_uniform_location("light_space_mat");
        depth_shader.store_uniform_location("model");

        let mut model_shader = Shader::new("resources/shaders/model.vs", "resources/shaders/model.fs");
        model_shader.store_uniform_location("projection");
        model_shader.store_uniform_location("view");
        model_shader.store_uniform_location("model");
        model_shader.store_dir_light_location("dir_light");
        model_shader.store_uniform_location("light_space_mat");
        model_shader.store_uniform_location("shadow_map");

        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        let mut container_diffuse = 0;
        let mut container_specular = 0;
        let mut cubemap_texture = 0;
        // =============================================================
        // Skybox memes
        // =============================================================
        unsafe {
            skybox_shader.activate();
            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));
            gl_call!(gl::GenBuffers(1, &mut ebo));

            vaos.insert(VaoType::Skybox, vao);

            println!("vao skybox: {}", vao);

            gl_call!(gl::BindVertexArray(vao));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER, 
                (mem::size_of::<f32>() * SKYBOX_VERTICES.len()) as isize,
                SKYBOX_VERTICES.as_ptr().cast(),
                gl::STATIC_DRAW
            ));

            gl_call!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo));
            gl_call!(gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mem::size_of::<u32>() * SKYBOX_INDICES.len()) as isize,
                SKYBOX_INDICES.as_ptr().cast(),
                gl::STATIC_DRAW
            ));

            gl_call!(gl::VertexAttribPointer(
                0, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                (3 * mem::size_of::<f32>()) as i32, 
                0 as *const _
            ));
            gl_call!(gl::EnableVertexAttribArray(0));

            gl_call!(gl::BindVertexArray(0));
            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_call!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));

            // SKYBOX TEXTURES
            gl_call!(gl::GenTextures(1, &mut cubemap_texture));
            gl_call!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, cubemap_texture));
            gl_call!(gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32));
            // These are very important to prevent seams
            gl_call!(gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32));

            for i in 0..FACES_CUBEMAP.len() {
                let img = match image::open(FACES_CUBEMAP[i]) {
                    Ok(img) => img,
                    _=> panic!("Error opening {}", FACES_CUBEMAP[i]),
                };
                let (img_width, img_height) = img.dimensions();
                let rgba = img.to_rgb8();
                let raw = rgba.as_raw();

                gl_call!(gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32, 
                    0, 
                    gl::RGB as i32, 
                    img_width as i32, 
                    img_height as i32, 
                    0, 
                    gl::RGB, 
                    gl::UNSIGNED_BYTE, 
                    raw.as_ptr().cast()
                ));
            }
        }

        // =============================================================
        // Debug point light setup
        // =============================================================
        unsafe {
            debug_light_shader.activate();

            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));

            vaos.insert(VaoType::DebugLight, vao);

            gl_call!(gl::BindVertexArray(vao));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER, 
                (mem::size_of::<f32>() * UNIT_CUBE_VERTICES.len()) as isize, 
                UNIT_CUBE_VERTICES.as_ptr().cast(), 
                gl::STATIC_DRAW
            ));

            // Position 
            gl_call!(gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                0 as *const _
            ));
            gl_call!(gl::EnableVertexAttribArray(0));
        
            // Normal
            gl_call!(gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                (5 * mem::size_of::<f32>()) as *const c_void
            ));
            gl_call!(gl::EnableVertexAttribArray(1));
        } 

        // =============================================================
        // Shadow Mapping
        // =============================================================
        // The general idea is that we need to create a depth map rendered 
        // from the perspective of the light source. In this case one 
        // directional light.
        // We can do this using a "framebuffer". We have been using a 
        // framebuffer all along, just the "default" one given to us.
        let mut fbo = 0;
        let mut depth_map = 0;
        unsafe {
            gl_call!(gl::GenFramebuffers(1, &mut fbo));

            fbos.insert(FboType::DepthMap, fbo);

            gl_call!(gl::GenTextures(1, &mut depth_map));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, depth_map));
            gl_call!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as i32, SHADOW_WIDTH, SHADOW_HEIGHT, 0, gl::DEPTH_COMPONENT, gl::FLOAT, null_mut()));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32));
            gl_call!(gl::TexParameterfv(
                gl::TEXTURE_2D, 
                gl::TEXTURE_BORDER_COLOR, 
                [1.0, 1.0, 1.0, 1.0].as_ptr().cast() 
            ));

            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, fbo));
            gl_call!(gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_map, 0));
            gl_call!(gl::DrawBuffer(gl::NONE));
            gl_call!(gl::ReadBuffer(gl::NONE));
            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
        }

        let mut debug_depth_quad = Shader::new("resources/shaders/debug_depth_quad.vs","resources/shaders/debug_depth_quad.fs");

        debug_depth_quad.activate();
        debug_depth_quad.store_uniform_location("depth_map");
        debug_depth_quad.set_int("depth_map", 0);

        shaders.insert(ShaderType::Model, model_shader);
        shaders.insert(ShaderType::Skybox, skybox_shader);
        shaders.insert(ShaderType::DebugLight, debug_light_shader);
        shaders.insert(ShaderType::Depth, depth_shader);
        shaders.insert(ShaderType::DebugShadowMap, debug_depth_quad);

        Self {
            shaders,
            vaos,
            fbos,
            depth_map,

            cubemap_texture,
        }
    }

    pub fn draw(&mut self, em: &EntityManager, camera: &mut Camera, light_manager: &Lights, grid: &mut Grid, fb_width: u32, fb_height: u32) {
        self.shadow_pass(em,  camera, light_manager, fb_width, fb_height);
        unsafe {
            gl_call!(gl::ClearColor(0.0, 0.0, 0.0, 1.0));
            gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        }
        let debug = false;
        if debug {
            unsafe {
                let depth_debug_quad = self.shaders.get(&ShaderType::DebugShadowMap).unwrap();
                depth_debug_quad.activate();
                gl_call!(gl::ActiveTexture(gl::TEXTURE0));
                gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.depth_map));
            }
            self.render_quad();
            return;
        }
        // SHADOW MUST GO FIRST
        self.skybox_pass(camera, fb_width, fb_height);
        // self.debug_light_pass(camera);
        self.grid_pass(grid, camera, light_manager, fb_width, fb_height);
        
        camera.reset_matrices(fb_width as f32 / fb_height as f32);
        let shader = self.shaders.get_mut(&ShaderType::Model).unwrap();
        shader.activate();
        for model in em.models.iter() {
            let trans = em.transforms.get(model.key()).unwrap();
            camera.model = Mat4::IDENTITY * Mat4::from_translation(trans.position) * Mat4::from_scale(trans.scale);

            shader.set_mat4("model", camera.model);
            shader.set_mat4("view", camera.view);
            shader.set_mat4("projection", camera.projection);
            shader.set_mat4("light_space_mat", camera.light_space);
            shader.set_dir_light("dir_light", &light_manager.dir_light);
            unsafe {
                // TODO: This could clash, we need to make sure we reserve texture0 in our dynamic shader code.
                gl_call!(gl::ActiveTexture(gl::TEXTURE2));
                gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.depth_map));
                shader.set_int("shadow_map", 2);
        }

            model.value.draw(shader);

            unsafe {
                gl_call!(gl::ActiveTexture(gl::TEXTURE0));
                gl_call!(gl::BindTexture(gl::TEXTURE_2D, 0));
            }
        }
    }

    fn grid_pass(&mut self,grid: &mut Grid, camera: &mut Camera, light_manager: &Lights, fb_width: u32, fb_height: u32) {
        camera.reset_matrices(fb_width as f32 / fb_height as f32);

        let shader = self.shaders.get_mut(&ShaderType::Model).unwrap();
        shader.activate();
        shader.set_mat4("model", camera.model);
        shader.set_mat4("view", camera.view);
        shader.set_mat4("projection", camera.projection);
        shader.set_mat4("light_space_mat", camera.light_space);
        shader.set_dir_light("dir_light", &light_manager.dir_light);

        grid.draw(shader);
    }

    fn skybox_pass(&mut self, camera: &mut Camera, fb_width: u32, fb_height: u32) {
        camera.reset_matrices(fb_width as f32 / fb_height as f32);
        
        unsafe {
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                println!("Framebuffer incomplete: {}", status);
            }
            let skybox_shader_prog = self.shaders.get(&ShaderType::Skybox).unwrap();

            gl_call!(gl::ClearColor(0.14, 0.13, 0.15, 1.0));
            gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));

            let view_no_translation = Mat4 {
                x_axis: camera.view.x_axis.clone(),
                y_axis: camera.view.y_axis.clone(),
                z_axis: camera.view.z_axis.clone(),
                w_axis: vec4(0.0, 0.0, 0.0, 1.0),
            };
            gl_call!(gl::DepthFunc(gl::LEQUAL));

            skybox_shader_prog.activate();
            skybox_shader_prog.set_mat4("view", view_no_translation);
            skybox_shader_prog.set_mat4("projection", camera.projection);

            gl_call!(gl::BindVertexArray(*self.vaos.get(&VaoType::Skybox).unwrap()));
            gl_call!(gl::ActiveTexture(gl::TEXTURE0));
            gl_call!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.cubemap_texture));
            gl_call!(gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, 0 as *const _));
            gl_call!(gl::BindVertexArray(0));

            gl_call!(gl::DepthFunc(gl::LESS));
        }
    }

    fn shadow_pass(&mut self, em: &EntityManager, camera: &mut Camera, light_manager: &Lights, fb_width: u32, fb_height: u32) {
        let shader = self.shaders.get_mut(&ShaderType::Depth).unwrap();
        let near_plane = 1.0;
        let far_plane = 10.0;
        let light_projection = Mat4::orthographic_rh_gl(-10.0, 10.0, -10.0, 10.0, near_plane, far_plane);
        let light_view = Mat4::look_at_rh(light_manager.dir_light.view_pos, vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));

        camera.light_space = light_projection * light_view;
        shader.activate();
        shader.set_mat4("light_space_mat", camera.light_space);

        unsafe {
            gl_call!(gl::Viewport(0, 0, SHADOW_WIDTH, SHADOW_HEIGHT));
            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, *self.fbos.get(&FboType::DepthMap).unwrap()));
            gl_call!(gl::Clear(gl::DEPTH_BUFFER_BIT));
            // Render scene
            self.render_sample_depth(em);
            // End render
            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER,0));
            gl_call!(gl::Viewport(0, 0, fb_width as i32, fb_height as i32));
        }
    }

    fn render_sample_depth(&mut self, em: &EntityManager) {
        let depth_shader = self.shaders.get(&ShaderType::Depth).unwrap();
        depth_shader.activate();

        for model in em.models.iter() {
            let trans = em.transforms.get(model.key()).unwrap();
            let model_model = Mat4::IDENTITY * Mat4::from_translation(trans.position) * Mat4::from_scale(trans.scale);
            for mesh in model.value().meshes.iter() {
                unsafe {
                    gl::BindVertexArray(mesh.vao);
                }
                depth_shader.set_mat4("model", model_model);
                unsafe {
                    gl_call!(gl::DrawElements(
                        gl::TRIANGLES, 
                        mesh.indices.len() as i32, 
                        gl::UNSIGNED_INT, 
                        0 as *const _
                    ));

                    gl_call!(gl::BindVertexArray(0));
                }
            }

        }
    }
    
    fn debug_light_pass(&mut self, camera: &mut Camera) {
        let debug_light_shader = self.shaders.get(&ShaderType::DebugLight).unwrap();
        debug_light_shader.activate();
        debug_light_shader.set_mat4("view", camera.view);
        debug_light_shader.set_mat4("projection", camera.projection);

        unsafe {
            gl_call!(gl::BindVertexArray(*self.vaos.get(&VaoType::DebugLight).unwrap()));
            for i in 0..POINT_LIGHT_POSITIONS.len() {
                camera.model = Mat4::IDENTITY;
                camera.model *= Mat4::from_translation(POINT_LIGHT_POSITIONS[i]);
                camera.model *= Mat4::from_scale(vec3(0.2, 0.2, 0.2)); 

                debug_light_shader.set_mat4("model", camera.model);
                debug_light_shader.set_vec3("LightColor", vec3(1.0, 1.0, 1.0));

                gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 36));
            }
        }
    }

    pub fn render_quad(&self) {
        let mut vao = 0;
        let mut vbo = 0;

        let quad_vertices: [f32; 30] = [
            // Positions      // Texture Coords
            -1.0,  1.0, 0.0,  0.0, 1.0,
            -1.0, -1.0, 0.0,  0.0, 0.0,
            1.0, -1.0, 0.0,  1.0, 0.0,

            -1.0,  1.0, 0.0,  0.0, 1.0,
            1.0, -1.0, 0.0,  1.0, 0.0,
            1.0,  1.0, 0.0,  1.0, 1.0
        ];

        unsafe {
            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));
            gl_call!(gl::BindVertexArray(vao));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (quad_vertices.len() * std::mem::size_of::<f32>()) as isize,
                quad_vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            ));

            let stride = (5 * std::mem::size_of::<f32>()) as i32;

            // Position Attribute
            gl_call!(gl::EnableVertexAttribArray(0));
            gl_call!(gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null()));

            // Texture Coordinate Attribute
            gl_call!(gl::EnableVertexAttribArray(1));
            gl_call!(gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * std::mem::size_of::<f32>()) as *const _));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_call!(gl::BindVertexArray(0));
        }

        // Draw the quad
        unsafe {
            gl_call!(gl::BindVertexArray(vao));
            gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6));
            gl_call!(gl::BindVertexArray(0));
        }
    }

}
