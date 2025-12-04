# R8: Yet Another CHIP-8 Interpreter in Rust 😉

R8 is a CHIP-8 emulator written in Rust. This repository is organized as a Cargo workspace with multiple crates, so each piece of functionality can be used independently or together to compose the application.

- `r8-core`: core types & utilities (addresses, memory, opcodes, registers, timers, stack, errors)
- `r8-emulator`: the emulator runtime (uses `r8-core`); suitable to build your own frontends/tools
- `r8-assembly`: assembler & tokenizer (standalone crate, planing to build a disassembler later)
- `r8-gui`: Bevy/egui frontend (uses `r8-emulator`, `r8-core`)
- `r8-tui`: Terminal UI frontend (uses `r8-emulator`, `r8-assembly`, `r8-core`)

![R8 Screenshot](img/Screnshot.png)

---

## Highlights (What changed)

- Project is now a Cargo workspace with multiple crates.
- The previous toggling of the TUI/GUI via features is no longer required. Each frontend is a separate crate + binary:
  - GUI binary: `gui`
  - TUI binary: `tui`
- Each frontend is independent; you can use `r8-emulator` alone to build a new frontend (e.g. SDL, Web, or any UI system).
- Assembly tooling (`r8-assembly`) is a standalone crate you can reuse.

---

## Prerequisites

- Rust toolchain (rustup)
- On Linux:
  - GTK 3 (if using file dialog support with the Bevy GUI)
  - Other dependencies your system might require for Bevy or audio backends
- On Windows:
  - Rust toolchain and any native dependencies for Bevy as needed

---

## Build & Run

From repo root:

- Build everything:
```bash
cd R8
cargo build
```

- Run GUI (Bevy + egui frontend):
```bash
cargo run --release --bin gui
```

- Run TUI (terminal frontend):
```bash
cargo run --release --bin tui -- --rom path/to/rom.rom
# or use assembled text input:
cargo run --release --bin tui -- --asm path/to/asm.8s
```

> On a workspace, binaries can be run with `cargo run --release --bin {gui|tui}`.
> If you prefer to target the specific crate by package, use:
> `cargo run -p r8-gui --release --bin gui` or `cargo run -p r8-tui --release --bin tui`.

---

## Developer Notes

- No more feature flags are required to run GUI or TUI — each is a separate crate and binary.
- If you want to build a new frontend, depend on `r8-emulator` in your crate's `Cargo.toml` and use the `Emulator` type to drive the emulation:

Example (minimal usage of `r8-emulator` in a new frontend):
```rust
use r8_emulator::Emulator;

fn main() {
    let mut emu = Emulator::new();

    // Load ROM from file (or another source)
    let rom_file = std::fs::File::open("examples/pong.rom").unwrap();
    emu.load_rom(rom_file).unwrap();

    // A simple emulation loop
    loop {
        emu.tick().unwrap(); // handles CPU tick / timers
        // Read emu.display() to render the frame, etc.
        // Use emu.press_key(...) / emu.release_key(...) to forward input
    }
}
```

- The assembler can be used from `r8-assembly` by calling `r8_assembly::assemble(...)` from other crates or tooling.

---

## Project structure

- `r8-core/` — Core library
- `r8-emulator/` — Emulator runtime (library)
- `r8-assembly/` — Assembler library
- `r8-gui/` — GUI binary (Bevy)
- `r8-tui/` — TUI binary (crossterm)

---

## CLI Options (TUI)

The TUI CLI supports:

```
USAGE:
    tui [OPTIONS] [--rom <ROM PATH>] [--asm <ASM FILE>]

OPTIONS:
    -d, --debug           Enable debug mode (verbose logging)
    -r, --rom <PATH>      Load a ROM file
    -a, --asm <PATH>      Load an assembly file and assemble it to ROM
```

For the GUI, the recorder uses a file dialog to load ROMs by default (no CLI rom path required), and you can toggle debug logging via environment or the TUI debug flags.

---

## Current state

- All CHIP-8 opcodes implemented
- Emulation (display, keyboard, timers, sound)
- Debugging UI (GUI + helper functions)
- Bevy GUI frontend
- TUI frontend with CLI options
- Assembler available as a library (`r8-assembly`)

---

## Future improvements

- Add a WebAssembly target
- Add disassembler crate or CLI
- Add save/load emulator state
- Improve the debug panel (memory, instruction pipeline view)
- Add more frontends or improve modularity (e.g., headless server mode)

---

## References

- [Wikipedia article on CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
- [CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Mastering CHIP-8](https://github.com/mattmikolay/chip-8/wiki/Mastering-CHIP%E2%80%908)
- Public domain ROMs: https://www.zophar.net/pdroms/chip8.html

---

## License

See `LICENSE` for licensing details.
