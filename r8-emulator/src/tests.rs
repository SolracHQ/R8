use super::emulator::Emulator;
use r8_core::{Address, RegisterIndex};

fn initialize_empty_emulator() -> Emulator {
  let mut emulator = Emulator::new();
  assert!(emulator.load_rom(&[0u8] as &[u8]).is_ok());
  emulator
}

#[test]
fn memory_load_and_access() {
  let data = [0xFF, 0xEE, 0xDD, 0xCC];
  let mut buffer = vec![0; 4];
  let mut emulator = initialize_empty_emulator();

  // Read Program EntryPoint to check it doesn't contain anything (initialized as zeros)
  emulator
    .memory
    .write_range(Address::ENTRY_POINT, &mut buffer)
    .unwrap();
  assert_eq!(buffer, [0; 4]);

  // Write some data into memory, then read it back to verify
  emulator
    .memory
    .read_range(Address::ENTRY_POINT, &data)
    .unwrap();
  emulator
    .memory
    .write_range(Address::ENTRY_POINT, &mut buffer)
    .unwrap();

  assert_eq!(buffer, data);
}

#[test]
/// Test 1NNN, 2NNN and 00EE chip-8 instructions
fn test_jump_instructions() {
  // Initialize the emulator
  let mut emulator = initialize_empty_emulator();

  let address: Address = Address::new(0x344);
  // Place instruction JP 0x344 at entry point -> 0x1344
  let jump_opcode = [0x13, 0x44];
  emulator
    .memory
    .read_range(Address::ENTRY_POINT, &jump_opcode)
    .unwrap();
  // Execute one tick: fetch/execute the JP instruction
  assert!(matches!(emulator.tick(), Ok(())));
  // Program counter must be at 0x344
  assert_eq!(address, emulator.pc);

  let program = [
    0x23, 0x46, // CALL 0x346
    0x00, 0xEE, // RET
  ];
  // Write program into memory at the computed address
  emulator.memory.read_range(address, &program).unwrap();
  // Execute call instruction (the CPU should call and set PC accordingly)
  assert!(matches!(emulator.tick(), Ok(())));
  // Call instruction increments PC to address + 2 -> 0x346
  assert_eq!(address.inner() + 2, emulator.pc.inner());
  // Execute next instruction (which should be RET) and ensure PC returns appropriately
  assert!(matches!(emulator.tick(), Ok(())));
  assert_eq!(address.inner() + 2, emulator.pc.inner());
}

#[test]
/// Test 3XKK, 4XKK and 5XY0 chip-8 instructions (skip conditions)
fn test_skip() {
  // Initialize the emulator
  let mut emulator = initialize_empty_emulator();

  // Define a program with various skip instructions and padding
  let program = [
    0x30, 0x00, // SE V0, 0 -> Skip next if V0 == 0
    0x00, 0x00, // Padding
    0x30, 0x01, // SE V0, 1 -> Skip next if V0 == 1
    0x42, 0x03, // SNE V2, 3 -> Skip next if V2 != 3
    0x00, 0x00, // Padding
    0x42, 0x04, // SNE V2, 4 -> Skip next if V2 != 4
    0x51, 0x20, // SE V1, V2
    0x00, 0x00, // Padding
    0x51, 0x20, // SE V1, V2
  ];

  // Load the program into the emulator's memory at the entry point
  emulator
    .memory
    .read_range(Address::ENTRY_POINT, &program)
    .unwrap();

  // Set V0 to a specific value for testing the first skip instruction
  emulator.registers[RegisterIndex::new(0)] = 0;

  // Tick the emulator and assert that the program counter has skipped the padding instruction
  assert!(matches!(emulator.tick(), Ok(())));
  assert_eq!(emulator.pc.inner(), Address::ENTRY_POINT.inner() + 4);

  // Change V0 to test the second skip instruction
  emulator.registers[RegisterIndex::new(0)] = 2;
  assert!(matches!(emulator.tick(), Ok(())));
  assert_eq!(emulator.pc.inner(), Address::ENTRY_POINT.inner() + 6);

  // Set V2 to a specific value for testing the third and fourth skip instructions
  emulator.registers[RegisterIndex::new(2)] = 4;
  assert!(matches!(emulator.tick(), Ok(())));
  assert_eq!(emulator.pc.inner(), Address::ENTRY_POINT.inner() + 10);
  assert!(matches!(emulator.tick(), Ok(())));
  assert_eq!(emulator.pc.inner(), Address::ENTRY_POINT.inner() + 12);

  // Set V1 and V2 to the same value for testing the fifth skip instruction
  emulator.registers[RegisterIndex::new(1)..=RegisterIndex::new(2)].copy_from_slice(&[6, 6]);

  assert!(matches!(emulator.tick(), Ok(())));
  assert_eq!(emulator.pc.inner(), Address::ENTRY_POINT.inner() + 16);

  // Change V1 to test the sixth skip instruction
  emulator.registers[RegisterIndex::new(1)] = 0;

  assert!(matches!(emulator.tick(), Ok(())));
  assert_eq!(emulator.pc.inner(), Address::ENTRY_POINT.inner() + 18);
}
