use std::path::PathBuf;

use clap::Parser;
use crossterm::{style::Stylize, ExecutableCommand};
use r8::emulator;

// Clap
#[derive(Parser)]
/// R8 - Chip-8 Emulator
pub struct R8 {
    #[clap(short, long)]
    /// Enable debug mode
    debug: bool,
    /// Path to the ROM to load
    #[clap(short, long)]
    rom: Option<PathBuf>,
    /// Path to the assembly file to load
    #[clap(short, long)]
    asm: Option<PathBuf>,
}

macro_rules! log_and_exit {
    ($($arg:tt)*) => {
        log::error!($($arg)*);
        crossterm::terminal::disable_raw_mode().unwrap();
        eprintln!($($arg)*);
        std::process::exit(1);
    };
}

fn main() {
    let args = R8::parse();

    let log_level = if args.debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    if let Err(err) = simple_logging::log_to_file("r8.log", log_level) {
        println!("Failed to initialize logging: {}", err);
        return;
    }

    // Enable raw mode
    crossterm::terminal::enable_raw_mode().unwrap();

    let mut emu = emulator::Emulator::new();

    load_rom(args, &mut emu);

    let mut stdout = std::io::stdout();

    let frame_duration = std::time::Duration::from_secs_f32(1.0 / 60.0);

    loop {
        let frame_start = std::time::Instant::now();

        if let Ok(true) = crossterm::event::poll(frame_duration) {
            match crossterm::event::read() {
                Ok(crossterm::event::Event::Key(key)) => {
                    log::debug!("Key: {:?}", key);
                    match key.code {
                        crossterm::event::KeyCode::Esc => {
                            break;
                        }
                        crossterm::event::KeyCode::Char(key) => {
                            if let Some(key) = map_key(key) {
                                emu.keyboard.set(key);
                            }
                        }
                        _ => {}
                    }
                }
                Err(err) => {
                    log_and_exit!("Failed to read event: {}", err);
                }
                _ => {}
            }
        }

        if let Err(err) = emu.tick() {
            log_and_exit!("Fatal emulator error: {}", err);
        }

        if emu.display.updated {
            if let Err(err) = stdout.execute(crossterm::terminal::Clear(
                crossterm::terminal::ClearType::All,
            )) {
                log_and_exit!("Failed to clear terminal: {}", err);
            }
            for x in 0..r8::constants::WIDTH {
                for y in 0..r8::constants::HEIGHT {
                    if emu.display.get(x, y) {
                        if let Err(err) =
                            stdout.execute(crossterm::cursor::MoveTo(x as u16, y as u16))
                        {
                            log_and_exit!("Failed to move cursor: {}", err);
                        }
                        if let Err(err) = stdout.execute(crossterm::style::Print('█'.blue())) {
                            log_and_exit!("Failed to print pixel: {}", err);
                        }
                    }
                }
            }
        }
        emu.display.updated = false;

        // Due TUI limitations, we can only know if a key is pressed
        // so we clear all keys on every frame
        (0..0xF).for_each(|x| emu.keyboard.unset(x));

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
    crossterm::terminal::disable_raw_mode().unwrap();
}

/// The original implementation of the Chip8 system had a 16-key hexadecimal keypad with the following layout:
///
/// | 1 | 2 | 3 | C |
/// |---|---|---|---|
/// | 4 | 5 | 6 | D |
/// | 7 | 8 | 9 | E |
/// | A | 0 | B | F |
///
/// The keys are mapped to the following indexes:
///
/// | 1 | 2 | 3 | 4 |
/// |---|---|---|---|
/// | Q | W | E | R |
/// | A | S | D | F |
/// | Z | X | C | V |
fn map_key(key: char) -> Option<u8> {
    match key {
        '1' => Some(0x1u8),
        '2' => Some(0x2),
        '3' => Some(0x3),
        '4' => Some(0xc),
        'Q' | 'q' => Some(0x4),
        'W' | 'w' => Some(0x5),
        'E' | 'e' => Some(0x6),
        'R' | 'r' => Some(0xd),
        'A' | 'a' => Some(0x7),
        'S' | 's' => Some(0x8),
        'D' | 'd' => Some(0x9),
        'F' | 'f' => Some(0xe),
        'Z' | 'z' => Some(0xa),
        'X' | 'x' => Some(0x0),
        'C' | 'c' => Some(0xb),
        'V' | 'v' => Some(0xf),
        _ => None,
    }
}

/// Loads the ROM or the assembly file.
fn load_rom(args: R8, emu: &mut emulator::Emulator) {
    match (args.rom, args.asm) {
        (Some(rom), None) => {
            let rom = match std::fs::File::open(rom) {
                Ok(file) => file,
                Err(err) => {
                    log_and_exit!("Failed to open ROM: {}", err);
                }
            };
            if let Err(err) = emu.load_rom(rom) {
                log_and_exit!("Failed to load ROM: {}", err);
            }
        }
        (None, Some(asm)) => {
            let mut asm = match std::fs::File::open(asm) {
                Ok(file) => file,
                Err(err) => {
                    log_and_exit!("Failed to open assembly file: {}", err);
                }
            };
            let mut rom = vec![];
            if let Err(err) = r8::assembler::assemble(&mut asm, &mut rom) {
                log_and_exit!("Failed to assemble: {}", err);
            }
            if let Err(err) = emu.load_rom(std::io::Cursor::new(rom)) {
                log_and_exit!("Failed to load ROM: {}", err);
            }
        }
        _ => {
            log_and_exit!("Please specify either a ROM or an assembly file");
        }
    }
}
