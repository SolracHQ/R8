mod gui;
mod sound;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().format_timestamp(None).init();
    let emu = r8::emulator::Emulator::new();
    gui::run(emu)?;
    Ok(())
}
