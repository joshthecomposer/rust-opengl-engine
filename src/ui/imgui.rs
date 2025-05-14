use glam::{Quat, Vec3};
use glfw::{Action, MouseButton, PWindow, WindowEvent};
use imgui::Drag;

use crate::{animation::animation::Animator, camera::Camera, entity_manager::EntityManager, enums_types::CameraState, gl_call, lights::Lights, renderer::Renderer, sound::sound_manager::SoundManager};

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

    pub fn draw(&mut self, window: &mut PWindow, width: f32, height: f32, delta: f32, lm: &mut Lights, rdr: &mut Renderer, sm: &mut SoundManager, camera: &Camera, em: &mut EntityManager) {
        {
            let io = self.imgui.io_mut();
            io.display_size = [width, height];
            io.delta_time   = delta;
        }
        let ui = self.imgui.frame();

        if camera.move_state == CameraState::Locked {
            window.set_cursor_mode(glfw::CursorMode::Normal);

            ui.window("Lights")
                .size([500.0, 200.0], imgui::Condition::FirstUseEver)
                .position([50.0, 50.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.text("Controls for Various Lights");
                    ui.separator();
                    // ui.input_float("Dir Light distance", &mut lm.dir_light.distance).build();
                    if ui.slider("Dir Light X", -1.0, 1.0, &mut lm.dir_light.direction.x) {
                        lm.dir_light.view_pos.x = lm.dir_light.direction.x * lm.dir_light.distance;
                    };                                                      
                    if ui.slider("Dir Light Y", -1.0, 1.0, &mut lm.dir_light.direction.y) {
                        lm.dir_light.view_pos.y = lm.dir_light.direction.y * lm.dir_light.distance;
                    };                                                      
                    if ui.slider("Dir Light Z", -1.0, 1.0, &mut lm.dir_light.direction.z) {
                        lm.dir_light.view_pos.z = lm.dir_light.direction.z * lm.dir_light.distance;
                    };
                    if ui.slider("Dir Light distance",0.0, 100.0, &mut lm.dir_light.distance) {
                    };

                    ui.checkbox("Shadow Debug",&mut rdr.shadow_debug);


                    ui.separator();

                    if ui.slider("Ortho Near", 0.0, 10.0, &mut lm.near) {
                    };
                    if ui.slider("Ortho Far", 0.0, 500.0, &mut lm.far) {
                    };
                    if ui.slider("Bounds", 0.0, 100.0, &mut lm.bounds) {
                    };

                    if ui.slider("Bias Scalar", 0.0, 0.3, &mut lm.bias_scalar) {
                    };

                    lm.dir_light.view_pos.x = lm.dir_light.direction.x * lm.dir_light.distance;
                    lm.dir_light.view_pos.y = lm.dir_light.direction.y * lm.dir_light.distance;
                    lm.dir_light.view_pos.z = lm.dir_light.direction.z * lm.dir_light.distance;


                });

            ui.window("Sound")
                .size([500.0, 200.0], imgui::Condition::FirstUseEver)
                .position([550.0, 50.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.text("Controls Fmod Sounds");
                    ui.separator();

                    if ui.button("Pause") {
                        sm.stop_sound("music");
                    }

                    if ui.button("Play") {
                        sm.play_sound_2d("music".to_string());
                    }

                    if ui.slider("Volume", 0.0, 1.0, &mut sm.master_volume) {
                        sm.set_master_volume("music");
                    }

                });

            ui.window("Entity Editing")
                .size([500.0, 200.0], imgui::Condition::FirstUseEver)
                .position([50.0, 250.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.separator();

                    for i in em.selected.iter() {
                        if let Some(trans) = em.transforms.get_mut(*i) {
                            ui.text(format!("Entity: {}, Type: {}", i, em.entity_types.get(*i).unwrap()));

                            let mut position = [trans.position.x, trans.position.y, trans.position.z];
                            let mut scale = [trans.scale.x];

                            // convert quat to euler angle degrees
                            let euler_degrees = trans.rotation.to_euler(glam::EulerRot::YXZ);
                            let mut rotation_deg = [
                                euler_degrees.0.to_degrees(),
                                euler_degrees.1.to_degrees(),
                                euler_degrees.2.to_degrees(),
                            ];

                            // position
                            if Drag::new("Position").speed(0.1).build_array(ui, &mut position) {
                                trans.position = Vec3::from(position);
                            }

                            //  scale
                            if Drag::new("Scale").speed(0.001).build_array(ui, &mut scale) {
                                trans.scale = Vec3::splat(scale[0]);
                            }

                            // rotation
                            if Drag::new("Rotation").speed(0.5).build_array(ui, &mut rotation_deg) {
                                let (y, x, z) = (
                                    rotation_deg[0].to_radians(),
                                    rotation_deg[1].to_radians(),
                                    rotation_deg[2].to_radians(),
                                );
                                trans.rotation = Quat::from_euler(glam::EulerRot::YXZ, y, x, z);
                            }
                        }

                        ui.separator();
                    }

                });

        } else {
            window.set_cursor_mode(glfw::CursorMode::Disabled);
        }


        ui.window("Some Info")
            .size([400.0, 150.0], imgui::Condition::FirstUseEver)
            .position([1100.0, 50.0], imgui::Condition::FirstUseEver)
            .build(|| {
                let string = format!("x: {:.3}, y: {:.3}, z: {:.3}", camera.position.x, camera.position.y, camera.position.z);
                ui.label_text("Camera Position", string);

                let string = format!("x: {:.3}, y: {:.3}, z: {:.3}", camera.forward.x, camera.forward.y, camera.forward.z);
                ui.label_text("Camera Forward", string);
            });

        self.renderer.render(&mut self.imgui);
    }
}
