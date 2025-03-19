        // TODO: clean this up, we shouldn't need to cast to f32. 

        // let radius = 0.5;
        // let speed = 1.0;
        // let angle = (self.elapsed * speed) as f32;

        // self.donut_pos.x = radius * angle.cos();
        // self.donut_pos.z = radius * angle.sin();
        // self.donut_pos.y = 1.0;

        // let donut2_r = 0.2;
        // let speed2 = 2.0;
        // let angle2 = (self.elapsed * speed2) as f32;

        // self.donut2_pos.x = self.donut_pos.x + donut2_r * angle2.cos();
        // self.donut2_pos.z = self.donut_pos.z + donut2_r * angle2.sin();
        // self.donut2_pos.y = 1.0; // Same height as Donut 1

        //write_data(self.animation.current_pose.clone(), "current_pose_after_one_update.txt");
        //panic!();





        // =============================================================
        // Render test text
        // =============================================================
        // let shader = self.renderer.shaders.get_mut(&ShaderType::Text).unwrap();

        // shader.activate();
        // unsafe {
        //     // Bind texture
        //     gl_call!(gl::ActiveTexture(gl::TEXTURE0));
        //     gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.glyph_tex));

        //     // Bind VAO and draw quad
        //     gl_call!(gl::BindVertexArray(self.tex_vao));
        //     gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6));

        //     // Cleanup
        //     gl_call!(gl::BindVertexArray(0));
        //     gl_call!(gl::UseProgram(0));
        // }









    // fn handle_imgui_event(&mut self, event: &WindowEvent) {
    //     let io = self.imgui.io_mut();
    //     match *event {
    //         // Mouse Buttons
    //         WindowEvent::MouseButton(btn, action, _) => {
    //             let pressed = action != Action::Release;
    //             match btn {
    //                 MouseButton::Button1 => io.mouse_down[0] = pressed,
    //                 MouseButton::Button2 => io.mouse_down[1] = pressed,
    //                 MouseButton::Button3 => io.mouse_down[2] = pressed,
    //                 _ => {}
    //             }
    //         }
    //         // Mouse Position
    //         WindowEvent::CursorPos(x, y) => {
    //             io.mouse_pos = [x as f32, y as f32];
    //         }
    //         // Scroll Wheel
    //         WindowEvent::Scroll(_x, scroll_y) => {
    //             io.mouse_wheel = scroll_y as f32;
    //         }
    //         // Text input
    //         WindowEvent::Char(ch) => {
    //             io.add_input_character(ch);
    //         }
    //         // Key press/release
    //         WindowEvent::Key(key, _, action, mods) => {
    //             let pressed = action != Action::Release;
    //             // If you want to track ImGui’s internal key map, do something like:
    //             // io.keys_down[imgui_key_index] = pressed;
    //             // or handle advanced shortcuts, etc.
    //         }

    //         _ => {}
    //     }
    // }








            //TEXT STUFF

        // =============================================================
        // text
        // ============================================================
        
        let font_data = include_bytes!("../resources/fonts/JetBrainsMonoNL-Regular.ttf");
        let font = Font::try_from_bytes(font_data).unwrap();

        let scale = Scale::uniform(256.0);
        let v_metrics = font.v_metrics(scale);
        let glyph = font.glyph('B').scaled(scale).positioned(point(0.0, v_metrics.ascent));

        let mut glyph_tex: u32 = 0;
        
        let mut glyph_width = 0.0;
        let mut glyph_height = 0.0;


        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph_width = bb.width() as f32;
            glyph_height = bb.height() as f32;
            let width = bb.width() as usize;
            let height = bb.height() as usize;
            let mut pixel_data = vec![0u8; width * height];

            glyph.draw(|x, y, v| {
                let index = (y as usize * width) + x as usize;
                pixel_data[index] = (v * 255.0) as u8;
            });

            let img = GrayImage::from_vec(width as u32, height as u32, pixel_data.clone())
                .expect("Failed to create image");
            img.save("glyph_debug.png").expect("Failed to save image");

    println!("Saved glyph_debug.png ({}x{})", width, height);

            dbg!(width, height, &pixel_data[..10]); // Debug first few pixel values

            unsafe {
                gl_call!(gl::GenTextures(1, &mut glyph_tex));
                gl_call!(gl::BindTexture(gl::TEXTURE_2D, glyph_tex));

                gl_call!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1));

                gl_call!(gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as i32,
                    width as i32,
                    height as i32,
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    pixel_data.as_ptr() as *const _,
                ));

                gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32));
                gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32));
                 gl_call!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4)); 
            }

        }

        let aspect_ratio = glyph_width / glyph_height;
        let pixel_scale_x = 2.0 / fb_width as f32;
        let pixel_scale_y = 2.0 / fb_height as f32;

        let width_ndc = glyph_width * pixel_scale_x;
        let height_ndc = glyph_height * pixel_scale_y;
        let scale = 1.0;
        let quad_vertices: [f32; 30] = [
            // Positions         // Flipped Texture Coords (Swap Y)
            -scale * aspect_ratio,  scale, 0.0,  0.0, 0.0,  // Top-left (was 1.0)
            -scale * aspect_ratio, -scale, 0.0,  0.0, 1.0,  // Bottom-left (was 0.0)
            scale * aspect_ratio, -scale, 0.0,  1.0, 1.0,  // Bottom-right

            -scale * aspect_ratio,  scale, 0.0,  0.0, 0.0,  // Top-left
            scale * aspect_ratio, -scale, 0.0,  1.0, 1.0,  // Bottom-right
            scale * aspect_ratio,  scale, 0.0,  1.0, 0.0   // Top-right
        ];

        let mut tex_vao = 0;
        let mut tex_vbo = 0;

        unsafe {
            gl_call!(gl::GenVertexArrays(1, &mut tex_vao));
            gl_call!(gl::GenBuffers(1, &mut tex_vbo));

            gl_call!(gl::BindVertexArray(tex_vao));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, tex_vbo));
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (quad_vertices.len() * std::mem::size_of::<f32>()) as isize,
                quad_vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            ));

            // Position attribute
            gl_call!(gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * std::mem::size_of::<f32>()) as i32, std::ptr::null()));
            gl_call!(gl::EnableVertexAttribArray(0));

            // Texture coordinate attribute
            gl_call!(gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * std::mem::size_of::<f32>()) as i32, (3 * std::mem::size_of::<f32>()) as *const _));
            gl::EnableVertexAttribArray(1);

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_call!(gl::BindVertexArray(0));
        }

