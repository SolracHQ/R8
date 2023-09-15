use log::warn;
use sdl2::keyboard::Keycode as K;

const SCALE: u32 = 12;

pub fn run(mut emulator: crate::emulator::Emulator) -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let video_subsystem = sdl_context.video()?;

    const BG_COLOR: egui::Color32 = egui::Color32::from_rgb(30, 30, 80);
    const FG_COLOR: egui::Color32 = egui::Color32::from_rgb(30, 120, 30);

    // Set OpenGL Attributes
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window(
            "R8 - Chip8 Emulator",
            SCALE * crate::emulator::WIDTH as u32 + 50,
            crate::emulator::HEIGHT as u32 * SCALE + 200,
        )
        .position_centered()
        .opengl()
        .build()?;

    let _ctx = window.gl_create_context()?;
    let mut egui_ctx = egui::Context::default();

    match video_subsystem.gl_set_swap_interval(-1) {
        Ok(()) => (),
        Err(err) => {
            warn!("Unable to set Adaptive VSync: {err}");
            video_subsystem.gl_set_swap_interval(1)?
        }
    }

    let shader_ver = egui_sdl2_gl::ShaderVersion::Default;
    let (mut painter, mut egui_state) =
        egui_sdl2_gl::with_sdl2(&window, shader_ver, egui_sdl2_gl::DpiScaling::Custom(1.0));

    let texture_id = painter.new_user_texture(
        (crate::emulator::WIDTH, crate::emulator::HEIGHT),
        &[egui::Color32::WHITE; crate::emulator::WIDTH * crate::emulator::HEIGHT],
        false,
    );

    let start_time = std::time::Instant::now();
    'game_loop: loop {
        std::thread::sleep(std::time::Duration::new(0, 16_666_666)); // 60Hz

        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());

        for event in event_pump.poll_iter() {
            use sdl2::event::Event as E;
            match event {
                E::Quit { .. }
                | E::KeyDown {
                    keycode: Some(K::Escape),
                    ..
                } => break 'game_loop,
                E::KeyDown {
                    keycode: Some(key), ..
                } => {
                    let key = get_key(key);
                    if let Some(key) = key {
                        emulator.keyboard.set(key)
                    }
                }
                E::KeyUp {
                    keycode: Some(key), ..
                } => {
                    let key = get_key(key);
                    if let Some(key) = key {
                        emulator.keyboard.unset(key)
                    }
                }
                E::DropFile { filename, .. } => {
                    let rom = std::fs::File::open(filename)?;
                    emulator.load_rom(rom)?;
                }
                _ => {
                    egui_sdl2_gl::input_to_egui(&window, event, &mut painter, &mut egui_state);
                }
            }
        }
        emulator.tick()?;
        let mut pixels = vec![];
        for y in 0..crate::emulator::HEIGHT {
            for x in 0..crate::emulator::WIDTH {
                pixels.push(match emulator.display.get(x, y) {
                    true => FG_COLOR,
                    false => BG_COLOR,
                })
            }
        }

        painter.update_user_texture_data(texture_id, &pixels);

        egui::Window::new("R8")
            .fixed_pos(egui::pos2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .show(&mut egui_ctx, |ui| {
                ui.add(egui::Image::new(
                    texture_id,
                    egui::vec2(
                        (SCALE * crate::emulator::WIDTH as u32) as _,
                        (crate::emulator::HEIGHT as u32 * SCALE) as _,
                    ),
                ))
            });

        let egui::FullOutput {
            platform_output,
            repaint_after: _,
            textures_delta,
            shapes,
        } = egui_ctx.end_frame();
        // Process output
        egui_state.process_output(&window, &platform_output);

        let paint_jobs = egui_ctx.tessellate(shapes);

        // Note: passing a bg_color to paint_jobs will clear any previously drawn stuff.
        // Use this only if egui is being used for all drawing and you aren't mixing your own Open GL
        // drawing calls with it.
        // Since we are custom drawing an OpenGL Triangle we don't need egui to clear the background.
        painter.paint_jobs(None, textures_delta, paint_jobs);

        window.gl_swap_window();
    }
    Ok(())
}

fn get_key(key: K) -> Option<u8> {
    match key {
        K::Num1 => Some(0x1u8),
        K::Num2 => Some(0x2),
        K::Num3 => Some(0x3),
        K::Num4 => Some(0xc),
        K::Q => Some(0x4),
        K::W => Some(0x5),
        K::E => Some(0x6),
        K::R => Some(0xd),
        K::A => Some(0x7),
        K::S => Some(0x8),
        K::D => Some(0x9),
        K::F => Some(0xe),
        K::Z => Some(0xa),
        K::X => Some(0x0),
        K::C => Some(0xb),
        K::V => Some(0xf),
        _ => None,
    }
}
