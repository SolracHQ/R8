use std::path::PathBuf;

use clap::Parser;
mod display;
mod input;
use crate::display::TerminalDisplay;
use crate::input::{process_event, release_all_keys};
use r8_emulator::Emulator;

/// CLI wrapper for the TUI binary
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
  // Clock speed in hz (default: 60)
  #[clap(short = 'c', long, default_value_t = 60.0)]
  clock: f64,
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

  let mut emu = Emulator::new();

  load_rom(&args, &mut emu);

  let mut td = match TerminalDisplay::new() {
    Ok(display) => display,
    Err(err) => {
      log_and_exit!("Failed to initialize terminal display: {}", err);
    }
  };

  let frame_duration = std::time::Duration::from_secs_f64(1.0 / args.clock);

  loop {
    let frame_start = std::time::Instant::now();

    if let Ok(true) = crossterm::event::poll(frame_duration) {
      match crossterm::event::read() {
        Ok(event) => {
          log::debug!("Event: {:?}", event);
          if process_event(event, &mut emu) {
            // input instructs to exit (e.g. Esc)
            break;
          }
        }
        Err(err) => {
          log_and_exit!("Failed to read event: {}", err);
        }
      }
    }

    if let Err(err) = emu.tick() {
      log_and_exit!("Fatal emulator error: {}", err);
    }

    if emu.display().updated {
      let vram = emu.display().get_vram();
      if let Err(err) = td.render(vram) {
        log_and_exit!("Failed to render display: {}", err);
      }
    } else {
      // When there's no update, still check terminal size so we can show the resize message
      if let Err(err) = td.check_size() {
        log_and_exit!("Failed to check terminal size: {}", err);
      }
    }

    // Due TUI limitations, we can only know if a key is pressed,
    // so we clear all keys on every frame.
    release_all_keys(&mut emu);

    let elapsed = frame_start.elapsed();
    if elapsed < frame_duration {
      std::thread::sleep(frame_duration - elapsed);
    }
  }
  crossterm::terminal::disable_raw_mode().unwrap();
}

/// Key mapping and event processing are handled inside the `input` module.
/// See `r8-tui/src/input.rs` for details.

/// Loads the ROM or the assembly file.
fn load_rom(args: &R8, emu: &mut Emulator) {
  match (args.rom.clone(), args.asm.clone()) {
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
      if let Err(err) = r8_assembly::assemble(&mut asm, &mut rom) {
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
