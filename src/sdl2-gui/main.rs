mod gui;
mod sound;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(file_err) = simple_logging::log_to_file("r8.log", log::LevelFilter::Debug) {
        // Can't log to file, tell to user and try to log to stdout
        eprintln!("Failed to log to file: {}", file_err);
        simple_logging::log_to(std::io::stdout(), log::LevelFilter::Debug);
    }
    let emu = r8::emulator::Emulator::new();
    gui::run(emu)?;
    Ok(())
}
