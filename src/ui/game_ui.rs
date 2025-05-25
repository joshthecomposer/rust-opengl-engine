use glam::{vec4, Vec2, Vec4};

use crate::{enums_types::ShaderType, gl_call, renderer::Renderer, shaders::Shader};

use super::{color::hex_to_vec4, font::FontManager, message_queue::{MessageQueue, UiMessage}};

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: Vec4,
    pub text: String,
}

pub fn do_ui(fb_width: f32, fb_height: f32, mouse_pos: Vec2, fm: &mut FontManager, shader: &Shader, font_shader: &Shader, mq: &mut MessageQueue) {
    let mut rects = Vec::new();
    let mut w = fb_width  * 0.25;
    let mut h = fb_height * 0.45;

    let mut pause_container = Rect {
        x: (fb_width / 2.0) - (w / 2.0),
        y: (fb_height / 2.0) - (h / 2.0),
        w,
        h,
        color: Vec4::splat(1.0),
        text: "".to_string(),
    };

    w = pause_container.w * 0.95; 
    h = pause_container.h * 0.95; 

    let button_h = pause_container.h * 0.15;
    let  exit_button_h = button_h / 3.0;
    let gap = 15.0; // Pixels

    let mut exit_button = Rect {
        x: (pause_container.x + pause_container.w) - (exit_button_h + gap),
        y: pause_container.y + gap,
        w: exit_button_h,
        h: exit_button_h,
        color: Vec4::splat(1.0),
        text: "X".to_string(),
    };

    let mut ph1 = Rect {
        x: pause_container.x + (pause_container.w / 2.0) - (w / 2.0),
        y: pause_container.y + h - ((button_h * 3.0) + (gap * 2.0)),
        w,
        h: button_h,
        color: Vec4::splat(1.0),
        text: "Placeholder".to_string(),
    };

    let mut ph2 = Rect {
        x: pause_container.x + (pause_container.w / 2.0) - (w / 2.0),
        y: pause_container.y + h - ((button_h * 2.0) + (gap * 1.0)),
        w,
        h: button_h,
        color: Vec4::splat(1.0),
        text: "Placeholder".to_string(),
    };

    let mut quit_button = Rect {
        x: pause_container.x + (pause_container.w / 2.0) - (w / 2.0),
        y: pause_container.y + h - button_h,
        w,
        h: button_h,
        color: Vec4::splat(1.0),
        text: "Quit Game".to_string(),
    };

    let hovering_exit_button = mouse_pos.x >= exit_button.x 
    && mouse_pos.y >= exit_button.y
    && mouse_pos.x <= exit_button.x + exit_button.w 
    && mouse_pos.y <= exit_button.h + exit_button.y;

    let hovering_quit_button = mouse_pos.x >= quit_button.x 
    && mouse_pos.y >= quit_button.y
    && mouse_pos.x <= quit_button.x + quit_button.w 
    && mouse_pos.y <= quit_button.h + quit_button.y;

    let hovering_ph1 = mouse_pos.x >= ph1.x 
    && mouse_pos.y >= ph1.y
    && mouse_pos.x <= ph1.x + ph1.w 
    && mouse_pos.y <= ph1.h + ph1.y;

    let hovering_ph2 = mouse_pos.x >= ph2.x 
    && mouse_pos.y >= ph2.y
    && mouse_pos.x <= ph2.x + ph2.w 
    && mouse_pos.y <= ph2.h + ph2.y;

    let color_950 = hex_to_vec4("#0c0a09");
    let color_900 = hex_to_vec4("#1c1917");
    let color_800 = hex_to_vec4("#292524");

    pause_container.color = color_950;

    if hovering_exit_button {
        if mq.queue.contains(&UiMessage::LeftMouseClicked) {
            mq.send(UiMessage::PauseToggle);
        }
        exit_button.color = color_800;
    } else {
        exit_button.color =color_900;
    };

    if hovering_quit_button {
        if mq.queue.contains(&UiMessage::LeftMouseClicked) {
            mq.send(UiMessage::WindowShouldClose);
        }
        quit_button.color = color_800
    } else {
        quit_button.color = color_900
    };

    if hovering_ph1 {
        ph1.color = color_800
    } else {
        ph1.color = color_900
    };

    if hovering_ph2 {
        ph2.color = color_800
    } else {
        ph2.color = color_900
    };

    rects.push(pause_container);
    rects.push(exit_button);
    rects.push(quit_button);
    rects.push(ph2);
    rects.push(ph1);

    draw_rects(rects, shader, fb_width, fb_height, fm, font_shader);
}

fn draw_rects(rects: Vec<Rect>, shader: &Shader, fb_width: f32, fb_height: f32, fm: &mut FontManager, font_shader: &Shader) {
    shader.activate();
    let mut vertices: Vec<f32> = Vec::with_capacity(rects.len() * 6 * 6);

    for rect in rects.iter() {
        let x0 = (rect.x / fb_width) * 2.0 - 1.0; // You should pass in fb_width/fb_height here
        let y0 = 1.0 - (rect.y / fb_height) * 2.0;
        let x1 = ((rect.x + rect.w) / fb_width) * 2.0 - 1.0;
        let y1 = 1.0 - ((rect.y + rect.h) / fb_height) * 2.0;

        let c = rect.color.to_array();

        // First triangle
        vertices.extend_from_slice(&[x0, y0, c[0], c[1], c[2], c[3]]);
        vertices.extend_from_slice(&[x1, y0, c[0], c[1], c[2], c[3]]);
        vertices.extend_from_slice(&[x1, y1, c[0], c[1], c[2], c[3]]);
        // Second triangle
        vertices.extend_from_slice(&[x1, y1, c[0], c[1], c[2], c[3]]);
        vertices.extend_from_slice(&[x0, y1, c[0], c[1], c[2], c[3]]);
        vertices.extend_from_slice(&[x0, y0, c[0], c[1], c[2], c[3]]);
    }

    let mut vbo = 0;
    let mut vao = 0;
    unsafe {
        gl_call!(gl::Disable(gl::DEPTH_TEST));
        gl_call!(gl::GenVertexArrays(1, &mut vao));
        gl_call!(gl::GenBuffers(1, &mut vbo));

        gl_call!(gl::BindVertexArray(vao));
        gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
        gl_call!(gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            gl::DYNAMIC_DRAW,
        ));

        gl_call!(gl::EnableVertexAttribArray(0));
        gl_call!(gl::VertexAttribPointer(
            0, 2, gl::FLOAT, gl::FALSE, 6 * 4, std::ptr::null()
        ));

        gl_call!(gl::EnableVertexAttribArray(1));
        gl_call!(gl::VertexAttribPointer(
            1,
            4,
            gl::FLOAT,
            gl::FALSE,
            6 * 4,
            (2 * 4) as *const _,
        ));

        gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, (vertices.len() / 6) as i32));

        gl_call!(gl::DeleteBuffers(1, &vbo));
        gl_call!(gl::DeleteVertexArrays(1, &vao));
        gl_call!(gl::Enable(gl::DEPTH_TEST));

        // Render font on top:
        for rect in &rects {
            let target_font_height = if rect.text == "X" {
                rect.h * 0.9
            } else {
                rect.h * 0.4
            };

            let scale = target_font_height / fm.font_pixel_size;

            fm.render_phrase_centered(&rect.text, rect, fb_width, fb_height, font_shader, scale);
        }

    }
}

