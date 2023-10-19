use crate::register::RegisterIndex;

use super::emulator::Emulator;

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

    // Read Program EntryPoint to check it don't load anithing
    emulator
        .memory
        .write_range(super::memory::Address::ENTRY_POINT, &mut buffer)
        .unwrap();
    assert_eq!(buffer, [0; 4]);

    // Test if write and read functions works as expected writing some data
    emulator
        .memory
        .read_range(super::memory::Address::ENTRY_POINT, &data)
        .unwrap();
    emulator
        .memory
        .write_range(super::memory::Address::ENTRY_POINT, &mut buffer)
        .unwrap();

    assert_eq!(buffer, data)
}

#[test]
/// Test 1NNN, 2NNN and 00EE chip-8 instructions
fn test_jump_instructions() {
    // Initialize the emulator
    let mut emulator = initialize_empty_emulator();

    let address: super::memory::Address = super::memory::Address::new(0x344);
    // Set instruction on 0x345 to JMP 0x200 -> 1200
    let jump_opcode = [0x13, 0x44];
    emulator
        .memory
        .read_range(super::memory::Address::ENTRY_POINT, &jump_opcode)
        .unwrap();
    // Tick Emulator
    assert!(matches!(emulator.tick(), Ok(())));
    // Program counter must be in the address 0x344
    assert_eq!(address, emulator.pc);
    let program = [
        0x23, 0x46, // Call 0x346
        0x00, 0xEE, // Ret
    ];
    // Write program on current address
    emulator.memory.read_range(address, &program).unwrap();
    // Tick Emulator
    assert!(matches!(emulator.tick(), Ok(())));
    // Call instruction is called and Program counter must be Address + 2 -> 0x346
    assert_eq!(address.inner() + 2, emulator.pc.inner());
    // Tick Emulator
    assert!(matches!(emulator.tick(), Ok(())));
    // Ret mus set Program counter to last saved PC + 2 -> 0x346
    assert_eq!(address.inner() + 2, emulator.pc.inner());
}

#[test]
/// Test 3xKK, 4xKK and 5xy0 chip-8 instructions
fn test_skip() {
    // Initialize the emulator
    let mut emulator = initialize_empty_emulator();

    // Define a program with various skip instructions and padding
    let program = [
        0x30, 0x00, // Skip next instruction if V0 equals 0
        0x00, 0x00, // Padding
        0x30, 0x01, // Skip next instruction if V0 equals 1
        0x42, 0x03, // Skip next instruction if V2 does not equal 3
        0x00, 0x00, // Padding
        0x42, 0x04, // Skip next instruction if V2 does not equal 4
        0x51, 0x20, // Skip next instruction if V1 equals V2
        0x00, 0x00, // Padding
        0x51, 0x20, // Skip next instruction if V1 equals V2
    ];

    // Load the program into the emulator's memory at the entry point
    emulator
        .memory
        .read_range(super::memory::Address::ENTRY_POINT, &program)
        .unwrap();

    // Set V0 to a specific value for testing the first skip instruction
    emulator.registers[RegisterIndex::new(0)] = 0;

    // Tick the emulator and assert that the program counter has skipped the padding instruction
    assert!(matches!(emulator.tick(), Ok(())));
    assert_eq!(
        emulator.pc.inner(),
        super::memory::Address::ENTRY_POINT.inner() + 4
    );

    // Change V0 to test the second skip instruction
    emulator.registers[RegisterIndex::new(0)] = 2;
    assert!(matches!(emulator.tick(), Ok(())));
    assert_eq!(
        emulator.pc.inner(),
        super::memory::Address::ENTRY_POINT.inner() + 6
    );

    // Set V2 to a specific value for testing the third and fourth skip instructions
    emulator.registers[RegisterIndex::new(2)] = 4;
    assert!(matches!(emulator.tick(), Ok(())));
    assert_eq!(
        emulator.pc.inner(),
        super::memory::Address::ENTRY_POINT.inner() + 10
    );
    assert!(matches!(emulator.tick(), Ok(())));
    assert_eq!(
        emulator.pc.inner(),
        super::memory::Address::ENTRY_POINT.inner() + 12
    );

    // Set V1 and V2 to the same value for testing the fifth skip instruction
    emulator.registers[RegisterIndex::new(1)..=RegisterIndex::new(2)].copy_from_slice(&[6, 6]);

    assert!(matches!(emulator.tick(), Ok(())));
    assert_eq!(
        emulator.pc.inner(),
        super::memory::Address::ENTRY_POINT.inner() + 16
    );

    // Change V1 to test the sixth skip instruction
    emulator.registers[RegisterIndex::new(1)] = 0;

    assert!(matches!(emulator.tick(), Ok(())));
    assert_eq!(
        emulator.pc.inner(),
        super::memory::Address::ENTRY_POINT.inner() + 18
    );
}
