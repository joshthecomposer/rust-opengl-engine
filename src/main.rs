mod shaders;

use std::{mem, ptr};
use glfw::{Context, Action, Key};

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw");
    
    let mut error_count = 0;
    glfw.set_error_callback(move |_, description| {
        println!("GLFW error {}: {}",error_count, description);
        error_count += 1;
    });

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3)); // OpenGL 3.3
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut width,mut height):(i32, i32) = (1280, 720);

let (mut window, events) = glfw
        .create_window(width as u32, height as u32, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    window.set_key_polling(true);
    window.make_current();

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    unsafe {
        gl::Viewport(0, 0, 1280, 720);
    }

    let vertices:[f32; 9] = [
        -0.5, -0.5, 0.0,  // Bottom left
         0.5, -0.5, 0.0,  // Bottom right
         0.0,  0.5, 0.0   // Top center
    ];

    let indices:[u32;3] = [
        0, 1, 2
    ];

    let mut vao = 0;
    let mut vbo = 0;
    let mut ebo = 0;

    let shader_program = shaders::init_shader_program("resources/shaders/shader.vs", "resources/shaders/shader.fs");

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            (mem::size_of::<f32>() * vertices.len()) as isize,
            vertices.as_ptr().cast(),
            gl::STATIC_DRAW
        );

        // Bind and set EBO data
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER, 
            (mem::size_of::<u32>() * indices.len()) as isize, 
            indices.as_ptr().cast(),
            gl::STATIC_DRAW
        );

        // Configure vertex attributes
        gl::VertexAttribPointer(
            0, 
            3, 
            gl::FLOAT, 
            gl::FALSE, 
            (3 * mem::size_of::<f32>()) as i32,
            0 as *const _
        );
        gl::EnableVertexAttribArray(0);
        gl::BindVertexArray(0);
    }

    while !window.should_close() {

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::FramebufferSize(w, h) => {
                    width = w;
                    height = h;
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
                }
                _ => handle_window_event(&mut window, event),
            }
        }

        unsafe {
            gl::ClearColor(0.1, 0.2, 0.3, 1.0); // Dark blue
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 3, gl::UNSIGNED_INT, ptr::null());

            gl::BindVertexArray(0);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        }
        _ => {}
    }
}


