use glam::{vec4, Vec2, Vec4};
use glfw::CursorMode;

use crate::{enums_types::{CameraState, ShaderType}, gl_call, renderer::Renderer, shaders::Shader};

use super::{color::hex_to_vec4, font::FontManager, message_queue::{MessageQueue, UiMessage}};

#[derive(Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: Vec4,
    pub text: String,
}

pub fn do_ui(fb_width: f32, fb_height: f32, mouse_pos: Vec2, fm: &mut FontManager, shader: &Shader, font_shader: &Shader, mq: &mut MessageQueue, paused: bool, cm: CursorMode, cs: &CameraState) {
    let mut rects = vec![];
    // =============================================================
    // PAUSE PANEL
    // =============================================================
    if paused {
        let mut w = fb_width  * 0.25;
        let h = fb_height * 0.45;

        let main_container = Rect {
            x: (fb_width / 2.0) - (w / 2.0),
            y: (fb_height / 2.0) - (h / 2.0),
            w,
            h,
            color: hex_to_vec4("#030712"),
            text: "".to_string(),
        };
        
        rects.push(main_container.clone());

        let button_h = main_container.h * 0.15;
        w = main_container.w * 0.95;

        let x = main_container.x + (main_container.w / 2.0) - (w / 2.0);
        // Bottom to top layout
        let mut y = main_container.y + main_container.h - button_h;

        let gap = 15.0; // Pixels

        y -= gap;
        if button("Quit Game", x, y, w, button_h, mouse_pos, mq, &mut rects, cm) {
            mq.send(UiMessage::WindowShouldClose);
        }

        y -= button_h + gap;
        if button("Placeholder 2", x, y, w, button_h, mouse_pos, mq, &mut rects, cm) {
            println!("PH2 clicked");
        }

        y -= button_h + gap;
        if button("Placeholder 1", x, y, w, button_h, mouse_pos, mq, &mut rects, cm) {
            println!("PH1 clicked");
        }

        // x button (close window)
        let exit_size = button_h / 3.0;
        let ex = (main_container.x + main_container.w) - (exit_size + gap);
        let ey = main_container.y + gap;

        if button("X", ex, ey, exit_size, exit_size, mouse_pos, mq, &mut rects, cm) {
            mq.send(UiMessage::PauseToggle);
        }
    }

    // =============================================================
    // LOWER RIGHT BOX
    // =============================================================
    // Main panel w/h
    if *cs == CameraState::Third {
        let mut w = fb_width * 0.15;
        let h = w;

        let main_container = Rect {
            x: fb_width - w,
            y: fb_height - h,
            w,
            h,
            color: hex_to_vec4("#030712"),
            text: "".to_string(),
        };

        rects.push(main_container.clone());

        let button_h = main_container.h * 0.15;

        w = main_container.w * 0.95;

        let x = main_container.x + (main_container.w / 2.0) - (w / 2.0);

        let gap = 10.0; // Pixels

        let mut y = main_container.y;
        
        // Top-down instead of bottom up for this one
        y += gap;
        if button("Placeholder 1", x, y, w, button_h, mouse_pos, mq, &mut rects, cm) {
            println!("PH1 clicked");
        }

        y += gap + button_h;
        if button("Placeholder 2", x, y, w, button_h, mouse_pos, mq, &mut rects, cm) {
            println!("PH2 clicked");
        }
    }

    // =============================================================
    // DRAW ALL BOXES AT THE END
    // =============================================================
    draw_rects(rects, shader, fb_width, fb_height, fm, font_shader);
}

// =============================================================
// Rendering
// =============================================================
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

        gl_call!(gl::Enable(gl::BLEND));
        gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));

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
        gl_call!(gl::Disable(gl::BLEND));

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

// =============================================================
// UI Elements
// =============================================================
pub fn button(
    label: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    mouse_pos: Vec2,
    mq: &mut MessageQueue,
    rects: &mut Vec<Rect>,
    cm: CursorMode,
) -> bool {
    let color_900 = hex_to_vec4("#1c1917");
    let color_800 = hex_to_vec4("#292524");
    let mut clicked = false;
    let mut final_color = color_900;

    if cm == CursorMode::Normal {
        let hovered = mouse_pos.x >= x
        && mouse_pos.y >= y
        && mouse_pos.x <= x + w
        && mouse_pos.y <= y + h;

        clicked = hovered && mq.queue.contains(&UiMessage::LeftMouseClicked);
        final_color = if hovered { color_800 } else { color_900 };
    }

    rects.push(Rect {
        x,
        y,
        w,
        h,
        color: final_color,
        text: label.to_string(),
    });

    clicked
}

