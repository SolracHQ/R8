mod emulator;
mod gui;
mod sound;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().format_timestamp(None).init();

    let rom = std::fs::File::open("roms/PONG2")?;
    let mut emu = emulator::Emulator::new();
    emu.load_rom(rom)?;

    gui::run(emu)?;
    Ok(())
}
