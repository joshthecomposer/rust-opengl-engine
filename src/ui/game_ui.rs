use glam::{vec4, Vec4};

use crate::{gl_call, shaders::Shader};

use super::color::hex_to_vec4;

pub struct UiContainer {
    // w,h are a percentage of the parent, if top level it is a percentage of the framebuffer
    pub w: f32,
    pub h: f32,
    pub x: f32,
    pub y: f32,
    pub g: f32,

    pub bg_color: Vec4,
    pub text: Option<String>,

    pub id: u32,
    pub parent: Option<u32>,
}

pub struct GameUi {
    pub next_id: u32,
    pub containers: Vec<UiContainer>,

    pub vao: u32,
    pub vbo: u32,
}

impl GameUi {
    pub fn new() -> Self {
        let mut game_ui = Self {
            next_id: 1,
            containers: vec![],
            vao: 0,
            vbo: 0,
        };
        game_ui.create_container(
                0.5,
                0.5,
                0.1,
                0.1,
                "#030712",
                None,
        );
        game_ui.setup_buffers();

        game_ui
    }

    pub fn create_container(&mut self, w: f32, h: f32, x: f32, y: f32, bg_color: &str, text: Option<&str>) {
        let final_text = match text {
            Some(t) => Some(t.to_string()),
            None => None,
        };

        self.containers.push(UiContainer {
            w,
            h,
            x,
            y,
            g: 0.0,
            bg_color: hex_to_vec4(bg_color),
            text: final_text,
            id: self.next_id,
            parent: None,
        });

        self.next_id += 1;

        for i in 0..3 {
            self.containers.push(UiContainer {
                w: 0.8,
                h: 0.1,
                x: 0.05,
                y: 0.05 + (0.1 * i as f32),
                g: 5.0, // pixels.. change this
                bg_color: hex_to_vec4("#111827"),
                text: None,
                id: self.next_id,
                parent: Some(1),
            });

            self.next_id += 1;
        }
    }

    pub fn setup_buffers(&mut self) {
        unsafe {
            gl_call!(gl::GenVertexArrays(1, &mut self.vao));
            gl_call!(gl::GenBuffers(1, &mut self.vbo));

            gl_call!(gl::BindVertexArray(self.vao));
            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo));
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (6 * 7 * std::mem::size_of::<f32>()) as isize, // 7 * 6 floats each
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            ));

            gl_call!(gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 7 * std::mem::size_of::<f32>() as i32, std::ptr::null()));
            gl_call!(gl::EnableVertexAttribArray(0));

            gl_call!(gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 7 * std::mem::size_of::<f32>() as i32, (3 * std::mem::size_of::<f32>()) as *const _));
            gl_call!(gl::EnableVertexAttribArray(1));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_call!(gl::BindVertexArray(0));
        }
    }

    pub fn draw(&self, fb_width: f32, fb_height: f32, shader: &mut Shader) {
        shader.activate();
        unsafe {
            gl_call!(gl::BindVertexArray(self.vao));
            gl_call!(gl::Disable(gl::DEPTH_TEST));
        }

        for con in self.containers.iter() {
            let (xpos, ypos, w, h) = self.get_absolute_position(con, fb_width, fb_height);

            let sx = 2.0 / fb_width;
            let sy = 2.0 / fb_height;

            let x0 = xpos * sx - 1.0;
            let x1 = (xpos + w) * sx - 1.0 ;
            let y0 = 1.0 - ypos * sy;
            let y1 = 1.0 - (ypos + h) * sy;

            let c: [f32; 4] = con.bg_color.into();

                let vertices: [f32; 42] = [
                    x0, y0, 0.0, c[0], c[1], c[2], c[3],
                    x0, y1, 0.0, c[0], c[1], c[2], c[3],
                    x1, y1, 0.0, c[0], c[1], c[2], c[3],
                    x0, y0, 0.0, c[0], c[1], c[2], c[3],
                    x1, y1, 0.0, c[0], c[1], c[2], c[3],
                    x1, y0, 0.0, c[0], c[1], c[2], c[3],
                ];

            unsafe {
                gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo));

                // TODO: unless the ui changes a bunch, we can probably just set up the buffer with the
                // vertices one time in the setup function instead of here.
                gl_call!(gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (vertices.len() * std::mem::size_of::<f32>()) as isize,
                    vertices.as_ptr() as *const _,
                    // gl::STATIC_DRAW,
                ));
                gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6));

            }
        }
        unsafe {
            gl_call!(gl::Enable(gl::DEPTH_TEST));
        }
    }

    pub fn get_absolute_position(&self, con: &UiContainer, fb_width: f32, fb_height:f32) -> (f32, f32, f32, f32) {
        if let Some(parent_id) = con.parent {
            if let Some(parent) = self.containers.iter().find(|c| c.id == parent_id) {
                let (px, py, pw, ph) = self.get_absolute_position(parent, fb_width, fb_height);

                let abs_w = pw * con.w;
                let abs_h = ph * con.h;
                let abs_x = px + pw * con.x;
                let abs_y = py + ph * con.y;

                (abs_x, abs_y, abs_w, abs_h)
            } else {
                // fallback if parent is missing
                (con.x, con.y, con.w, con.h)
            }
        } else {
            // top-level container: treat x/y/w/h as relative to framebuffer
            (
                con.x *  fb_width,
                con.y *  fb_height,
                con.w *  fb_width,
                con.h *  fb_height,
            )
        }
    }
}
