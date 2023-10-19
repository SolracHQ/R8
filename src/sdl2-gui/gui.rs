use log::warn;
use r8::{emulator::Emulator, HEIGHT, REGISTER_COUNT, WIDTH};
use sdl2::keyboard::Keycode as K;

const SCALE: u32 = 12;

pub fn run(mut emulator: Emulator) -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;
    let mut speaker = crate::sound::Speaker::new(&audio_subsystem)?;

    const BG_COLOR: egui::Color32 = egui::Color32::from_rgb(30, 30, 80);
    const FG_COLOR: egui::Color32 = egui::Color32::from_rgb(30, 120, 30);

    // Set OpenGL Attributes
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window(
            "R8 - Chip8 Emulator",
            WIDTH as u32 * SCALE + 20,
            HEIGHT as u32 * SCALE + 150,
        )
        .position_centered()
        .opengl()
        .build()?;

    let _ctx = window.gl_create_context()?;
    let egui_ctx = egui::Context::default();

    // Set Adaptive VSync if available
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

    // Create a texture for the emulator display
    let texture_id = painter.new_user_texture(
        (WIDTH, HEIGHT),
        &[egui::Color32::WHITE; WIDTH * HEIGHT],
        false,
    );

    let start_time = std::time::Instant::now();
    let mut init_instant = std::time::Instant::now();

    let mut paused: bool = false;
    let mut step: bool = false;

    loop {
        let delta = init_instant.elapsed().as_secs_f64();
        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());

        manage_events(
            &mut event_pump,
            &mut emulator,
            &window,
            &mut painter,
            &mut egui_state,
        )?;

        if !paused || step {
            if step {
                step = false;
            }
            emulator.tick()?;
        }

        if emulator.display.updated {
            let pixels = emulator
                .display
                .grid()
                .map(|pixel| if pixel { FG_COLOR } else { BG_COLOR })
                .collect::<Vec<_>>();
            emulator.display.updated = false;
            painter.update_user_texture_data(texture_id, &pixels);
        }

        match emulator.sound_timer() {
            0 => speaker.stop(),
            _ => speaker.play(),
        }

        egui::Window::new("R8")
            .fixed_pos(egui::pos2(0.0, 0.0))
            .auto_sized()
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .show(&egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.image(
                            texture_id,
                            egui::vec2((SCALE * WIDTH as u32) as _, (HEIGHT as u32 * SCALE) as _),
                        )
                    })
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(format!("PC: 0x{:03X}", emulator.pc().inner()));
                    ui.label(format!("I: 0x{:03X}", emulator.i().inner()));
                    ui.label(format!("SP: 0x{:03X}", emulator.stack().len()));
                    ui.label(format!("DT: 0x{:02X}", emulator.delay_timer()));
                    ui.label(format!("ST: 0x{:02X}", emulator.sound_timer()));
                    ui.label(format!("State: {:?}", emulator.state()));
                    ui.label(format!("{}", emulator.fetch_opcode().unwrap()));
                });

                // Print V registers on colums of 4
                for i in (0..REGISTER_COUNT as u8).step_by(4) {
                    ui.horizontal(|ui| {
                        for j in i..i + 4 {
                            ui.label(format!("V{:X}: 0x{:02X}", j, emulator.v_registers().try_index(j).unwrap()));
                        }
                    });
                }

                // Buttons to pause and step and continue
                ui.horizontal(|ui| {
                    if ui.button("Load ROM").clicked() {
                        let rom = rfd::FileDialog::new()
                            .add_filter("CHIP-8 ROM", &["ch8", "rom", "bin"])
                            .pick_file();
                        if let Some(rom) = rom {
                            let rom = match std::fs::File::open(rom) {
                                Ok(rom) => rom,
                                Err(err) => {
                                    warn!("Unable to open file: {}", err);
                                    return;
                                }
                            };
                            emulator.load_rom(rom).unwrap();
                        }
                    }
                    if ui.button("Load Source Code").clicked() {
                        let src = rfd::FileDialog::new()
                            .add_filter("CHIP-8 Source Code", &["asm", "txt", "8s"])
                            .pick_file();
                        if let Some(src) = src {
                            let mut src = match std::fs::File::open(src) {
                                Ok(src) => src,
                                Err(err) => {
                                    warn!("Unable to open file: {}", err);
                                    return;
                                }
                            };
                            let mut game = vec![];
                            match r8::assembler::assemble(&mut src, &mut game) {
                                Ok(_) => {
                                    emulator.load_rom(game.as_slice()).unwrap();
                                }
                                Err(err) => {
                                    warn!("Unable to assemble source code: {}", err);
                                    return;
                                }
                            }
                        }
                    }
                    if ui.button("Pause").clicked() {
                        paused = true;
                    }
                    if ui.button("Step").clicked() {
                        step = true;
                    }
                    if ui.button("Continue").clicked() {
                        paused = false;
                    }
                });
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

        painter.paint_jobs(None, textures_delta, paint_jobs);

        window.gl_swap_window();

        // Take new snapshot of time
        init_instant = std::time::Instant::now();

        // If delta is less than 1/60s, wait for the remaining time
        if delta < 1.0 / 60.0 {
            std::thread::sleep(std::time::Duration::new(
                0,
                ((1.0 / 60.0 - delta) * 1_000_000_000.0) as u32,
            ));
        }
    }
}

fn manage_events(
    event_pump: &mut sdl2::EventPump,
    emulator: &mut Emulator,
    window: &sdl2::video::Window,
    painter: &mut egui_sdl2_gl::painter::Painter,
    egui_state: &mut egui_sdl2_gl::EguiStateHandler,
) -> Result<(), Box<dyn std::error::Error>> {
    for event in event_pump.poll_iter() {
        use sdl2::event::Event as E;
        match event {
            E::Quit { .. }
            | E::KeyDown {
                keycode: Some(K::Escape),
                ..
            } => std::process::exit(0),
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
                egui_sdl2_gl::input_to_egui(window, event, painter, egui_state);
            }
        }
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
