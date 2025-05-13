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




		{
			"entity_type": "MooseMan",
			"position": [0.0, 0.0, 4.0],
			"scale": [0.013, 0.013, 0.013],
			"__note__": "use Quat::from_rotation_x()",
			"rotation": "-FRAC_PI_2",
			"faction": "Enemy",
			"mesh_path": "resources/models/animated/001_moose/moose_model_FINAL.txt", 
			"bone_path": "resources/models/animated/001_moose/moose_bones_FINAL.txt",
			"animation_properties" : [
				{
					"name": "Idle",
					"one_shots": { },
					"continuous_sounds": [
						"moose3D"
					]
				}
			]
		},



		{
			"entity_type": "TreeFoliage",
			"position": [-4.2, 0.0, -3.1],
			"scale": [1.0, 1.0, 1.0],
			"__note__": "use Quat::from_rotation_x()",
			"rotation": "",
			"faction": "Static",
			"mesh_path": "resources/models/static/trees/001_tree_foliage_model.txt", 
			"bone_path":"",
			"animation_properties" : []
		},
		{
			"entity_type": "TreeTrunk",
			"position": [-4.2, 0.0, -3.1],
			"scale": [1.0, 1.0, 1.0],
			"__note__": "use Quat::from_rotation_x()",
			"rotation": "",
			"faction": "Static",
			"mesh_path": "resources/models/static/trees/001_tree_trunk_model.txt", 
			"bone_path":"",
			"animation_properties" : []
		},
		{
			"entity_type": "TreeFoliage",
			"position": [3.0, 0.0, 2.8],
			"scale": [1.0, 1.0, 1.0],
			"__note__": "use Quat::from_rotation_x()",
			"rotation": "",
			"faction": "Static",
			"mesh_path": "resources/models/static/trees/001_tree_foliage_model.txt", 
			"bone_path":"",
			"animation_properties" : []
		},
		{
			"entity_type": "TreeTrunk",
			"position": [3.0, 0.0, 2.8],
			"scale": [1.0, 1.0, 1.0],
			"__note__": "use Quat::from_rotation_x()",
			"rotation": "",
			"faction": "Static",
			"mesh_path": "resources/models/static/trees/001_tree_trunk_model.txt", 
			"bone_path":"",
			"animation_properties" : []
		},


		{
			"entity_type": "TreeFoliage",
			"position": [5.0, 0.0, -2.1],
			"scale": [1.0, 1.0, 1.0],
			"__note__": "use Quat::from_rotation_x()",
			"rotation": "",
			"faction": "Static",
			"mesh_path": "resources/models/static/trees/001_tree_foliage_model.txt", 
			"bone_path":"",
			"animation_properties" : []
		},
		{
			"entity_type": "TreeTrunk",
			"position": [5.0, 0.0, -2.1],
			"scale": [1.0, 1.0, 1.0],
			"__note__": "use Quat::from_rotation_x()",
			"rotation": "",
			"faction": "Static",
			"mesh_path": "resources/models/static/trees/001_tree_trunk_model.txt", 
			"bone_path":"",
			"animation_properties" : []
		}







            // Donut revolution stuff
            if let Some(donut) = self.entity_types.iter().find(|e| e.value() == &EntityType::Donut) {
                let donut_key = donut.key();

                let player_position = self.transforms.get(player_key).map(|t| t.position);

                if let Some(donut_transform) = self.transforms.get_mut(donut_key) {
                    if let Some(player_position) = player_position {
                        revolve_around_something(
                            &mut donut_transform.position,
                            &player_position,
                            elapsed_time,
                            2.0,
                            5.0
                        );
                    }
                }
            }





        /// RNG FLAT MAP STUFF
    pub fn populate_floor_tiles(&mut self, grid: &Grid, model_path: &str) {
        for cell in grid.cells.iter() {
            let pos = cell.position;
            self.create_static_entity(EntityType::BlockGrass, Faction::World, pos, vec3(1.0, 1.0, 1.0), Quat::IDENTITY, model_path);
        }
    }

    pub fn populate_cell_rng(&mut self, grid: &Grid) {
        for cell in grid.cells.iter() {

            let (entity_data, subtile_size, entity_type) = match cell.cell_type {
                CellType::Tree => (TREES, 3.0, EntityType::Tree),
                CellType::Grass => (GRASSES, 3.0, EntityType::Grass),
                _=> continue,
            };

            let within = grid.cell_size / subtile_size;

            let cell_pos = cell.position;
            for x in -1..=1 {
                for z in -1..=1 {
                    let num = self.rng.random_range(0..entity_data.len() + 1);
                    let scale = match entity_type {
                        EntityType::Grass => self.rng.random_range(20..=45) as f32 / 100.0,
                        EntityType::Tree => self.rng.random_range(90..=110) as f32 / 100.0,
                        _=> 1.0,
                };
                    let smoff = self.rng.random_range(-0.1..=0.1);

                    let offset_x = x as f32 * within;
                    let offset_z = z as f32 * within;

                    if num < entity_data.len() {
                        self.create_static_entity(
                            entity_type.clone(),
                            Faction::World,
                            vec3(cell_pos.x + offset_x + smoff, 0.0, cell_pos.z + offset_z + smoff),
                            Vec3::splat(scale),
                            Quat::IDENTITY,
                            entity_data[num],
                        );
                    }
                }
            }
        }
    }

