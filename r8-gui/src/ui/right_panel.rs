use bevy_egui::egui;
use r8_core::constants;

use crate::emulator::Emulator;

pub const RIGHT_PANEL_WIDTH: f32 = 300.0;

/// Memory inspector state
#[derive(Default)]
pub struct MemoryInspectorState {
  /// Address to view (hex string for input)
  pub address_input: String,
  /// Current view address
  pub view_address: u16,
  /// Number of bytes to display
  pub bytes_to_show: usize,
  /// Follow PC automatically
  pub follow_pc: bool,
  /// Follow I register automatically
  pub follow_i: bool,
}

impl MemoryInspectorState {
  pub fn new() -> Self {
    Self {
      address_input: String::from("200"),
      view_address: 0x200,
      bytes_to_show: 128,
      follow_pc: false,
      follow_i: false,
    }
  }
}

/// Renders the right debug panel with CPU state, registers, and memory inspector
pub fn right_panel_system(
  ctx: &egui::Context,
  emulator: &Emulator,
  memory_state: &mut MemoryInspectorState,
) {
  egui::SidePanel::right("r8_debug_panel")
    .exact_width(RIGHT_PANEL_WIDTH)
    .resizable(false)
    .show(ctx, |ui| {
      egui::ScrollArea::vertical().show(ui, |ui| {
        // CPU State Section
        ui.heading("CPU State");
        ui.separator();

        egui::Grid::new("cpu_state_grid")
          .num_columns(2)
          .spacing([20.0, 4.0])
          .show(ui, |ui| {
            ui.label("PC:");
            ui.monospace(format!("0x{:03X}", emulator.0.pc().inner()));
            ui.end_row();

            ui.label("I:");
            ui.monospace(format!("0x{:03X}", emulator.0.i().inner()));
            ui.end_row();

            ui.label("SP:");
            ui.monospace(format!("0x{:X}", emulator.0.stack().len()));
            ui.end_row();

            ui.label("DT:");
            ui.monospace(format!("0x{:02X}", emulator.0.delay_timer()));
            ui.end_row();

            ui.label("ST:");
            ui.monospace(format!("0x{:02X}", emulator.0.sound_timer()));
            ui.end_row();

            ui.label("State:");
            ui.monospace(format!("{:?}", emulator.0.state()));
            ui.end_row();
          });

        ui.add_space(4.0);

        // Current opcode
        ui.horizontal(|ui| {
          ui.label("Opcode:");
          if let Ok(opcode) = emulator.0.fetch_opcode() {
            ui.monospace(format!("{}", opcode));
          } else {
            ui.monospace("---");
          }
        });

        ui.add_space(8.0);
        ui.separator();

        // Registers Section
        ui.heading("V Registers");
        ui.separator();

        egui::Grid::new("registers_grid")
          .num_columns(4)
          .spacing([8.0, 4.0])
          .show(ui, |ui| {
            for i in 0..constants::REGISTER_COUNT {
              let idx = i as u8;
              let value = *emulator.0.v_registers().try_index(idx).unwrap();
              ui.monospace(format!("V{:X}:{:02X}", idx, value));

              if (i + 1) % 4 == 0 {
                ui.end_row();
              }
            }
          });

        ui.add_space(8.0);
        ui.separator();

        // Stack Section
        ui.heading("Stack");
        ui.separator();

        let stack = emulator.0.stack();
        if stack.is_empty() {
          ui.label("(empty)");
        } else {
          egui::Grid::new("stack_grid")
            .num_columns(2)
            .spacing([8.0, 2.0])
            .show(ui, |ui| {
              for (i, addr) in stack.iter().enumerate() {
                ui.monospace(format!("[{}]", i));
                ui.monospace(format!("0x{:03X}", addr.inner()));
                ui.end_row();
              }
            });
        }

        ui.add_space(8.0);
        ui.separator();

        // Memory Inspector Section
        ui.heading("Memory Inspector");
        ui.separator();

        // Update view address if following PC or I
        if memory_state.follow_pc {
          memory_state.view_address = emulator.0.pc().inner();
          memory_state.address_input = format!("{:03X}", memory_state.view_address);
        } else if memory_state.follow_i {
          memory_state.view_address = emulator.0.i().inner();
          memory_state.address_input = format!("{:03X}", memory_state.view_address);
        }

        // Address input and controls
        ui.horizontal(|ui| {
          ui.label("Addr:");
          let response = ui.add(
            egui::TextEdit::singleline(&mut memory_state.address_input)
              .desired_width(50.0)
              .font(egui::TextStyle::Monospace),
          );

          if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Ok(addr) = u16::from_str_radix(&memory_state.address_input, 16) {
              memory_state.view_address = addr.min(0xFFF);
              memory_state.follow_pc = false;
              memory_state.follow_i = false;
            }
          }

          if ui.button("Go").clicked() {
            if let Ok(addr) = u16::from_str_radix(&memory_state.address_input, 16) {
              memory_state.view_address = addr.min(0xFFF);
              memory_state.follow_pc = false;
              memory_state.follow_i = false;
            }
          }
        });

        ui.horizontal(|ui| {
          if ui
            .checkbox(&mut memory_state.follow_pc, "Follow PC")
            .clicked()
          {
            if memory_state.follow_pc {
              memory_state.follow_i = false;
            }
          }
          if ui
            .checkbox(&mut memory_state.follow_i, "Follow I")
            .clicked()
          {
            if memory_state.follow_i {
              memory_state.follow_pc = false;
            }
          }
        });

        ui.horizontal(|ui| {
          ui.label("Bytes:");
          ui.add(egui::Slider::new(&mut memory_state.bytes_to_show, 32..=256).step_by(16.0));
        });

        ui.add_space(4.0);

        // Memory hex dump
        render_memory_dump(ui, emulator, memory_state);
      });
    });
}

/// Renders a hex dump of memory
fn render_memory_dump(ui: &mut egui::Ui, emulator: &Emulator, state: &MemoryInspectorState) {
  let start_addr = state.view_address as usize;
  let bytes_per_row = 8;
  let pc = emulator.0.pc().inner() as usize;
  let i_reg = emulator.0.i().inner() as usize;

  egui::ScrollArea::vertical()
    .max_height(200.0)
    .show(ui, |ui| {
      ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

      for row in 0..(state.bytes_to_show / bytes_per_row) {
        let row_addr = start_addr + row * bytes_per_row;
        if row_addr >= 0x1000 {
          break;
        }

        ui.horizontal(|ui| {
          // Address column
          ui.label(format!("{:03X}:", row_addr));

          // Hex bytes
          let mut hex_str = String::new();
          let mut ascii_str = String::new();

          for col in 0..bytes_per_row {
            let addr = row_addr + col;
            if addr >= 0x1000 {
              hex_str.push_str("   ");
              ascii_str.push(' ');
              continue;
            }

            // Read byte from memory
            let byte = read_memory_byte(emulator, addr as u16);

            // Highlight PC and I positions
            if addr == pc || addr == pc + 1 {
              hex_str.push_str(&format!("[{:02X}]", byte));
            } else if addr == i_reg {
              hex_str.push_str(&format!("<{:02X}>", byte));
            } else {
              hex_str.push_str(&format!(" {:02X} ", byte));
            }

            // ASCII representation
            if byte >= 0x20 && byte <= 0x7E {
              ascii_str.push(byte as char);
            } else {
              ascii_str.push('.');
            }
          }

          ui.label(hex_str);
          ui.separator();
          ui.label(ascii_str);
        });
      }
    });

  // Legend
  ui.add_space(4.0);
  ui.horizontal(|ui| {
    ui.label("[XX] = PC");
    ui.label("<XX> = I");
  });
}

/// Helper to read a byte from emulator memory
fn read_memory_byte(emulator: &Emulator, addr: u16) -> u8 {
  // Use fetch through the memory's Index implementation
  if let Ok(address) = addr.try_into() {
    // Access memory through the emulator's internal state
    // We need to use the debug accessor or expose memory
    // For now, we'll read via the fetch_opcode-like pattern
    let mut buf = [0u8; 1];
    if emulator.0.read_memory(address, &mut buf).is_ok() {
      return buf[0];
    }
  }
  0
}
