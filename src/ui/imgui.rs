use glfw::{Action, MouseButton, PWindow, WindowEvent};

use crate::gl_call;

pub struct ImguiManager {
    pub imgui: imgui::Context,
    pub renderer: imgui_opengl_renderer::Renderer,
}

impl ImguiManager {
    pub fn new(window: &mut PWindow) -> Self {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
             window.get_proc_address(s) as *const _
         });
        Self {
            imgui,
            renderer,
        }
    }

    pub fn handle_imgui_event(&mut self, event: &WindowEvent) {
        let io = self.imgui.io_mut();
        match *event {
            // Mouse Buttons
            WindowEvent::MouseButton(btn, action, _) => {
                let pressed = action != Action::Release;
                match btn {
                    MouseButton::Button1 => {
                        io.mouse_down[0] = pressed;
                        println!("presed a thing");
                    },
                    MouseButton::Button2 => io.mouse_down[1] = pressed,
                    MouseButton::Button3 => io.mouse_down[2] = pressed,
                    _ => {}
                }
            }
            // Mouse Position
            WindowEvent::CursorPos(x, y) => {
                io.mouse_pos = [x as f32, y as f32];
            }
            // Scroll Wheel
            WindowEvent::Scroll(_x, scroll_y) => {
                io.mouse_wheel = scroll_y as f32;
            }
            // Text input
            WindowEvent::Char(ch) => {
                io.add_input_character(ch);
            }
            // Key press/release
            WindowEvent::Key(_key, _, action, _mods) => {
                let _pressed = action != Action::Release;
                // If you want to track ImGui’s internal key map, do something like:
                // io.keys_down[imgui_key_index] = pressed;
                // or handle advanced shortcuts, etc.
            }

            _ => {}
        }
    }

    pub fn draw(&mut self, window: &mut PWindow, width: f32, height: f32, delta: f64) {
        window.set_cursor_mode(glfw::CursorMode::Normal);

        {
            let io = self.imgui.io_mut();
            io.display_size = [width, height];
            io.delta_time   = delta as f32;
        }

        let ui = self.imgui.frame();
        ui.window("A window")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .position([50.0, 50.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text("some text");
                ui.separator();
                ui.button("Button");
                if ui.slider("slider", 0.0, 1.0, &mut 0.0) {
                    println!("slid that shiz");
                };
            });

        self.renderer.render(&mut self.imgui);
    }
}
