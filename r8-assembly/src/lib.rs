use std::io::{Read, Write};

use std::collections::HashMap;

use self::memory_slices::MemorySlices;
use self::tokenizer::Line;

pub mod error;
mod memory_slices;
mod tokenizer;

/// Assembles a CHIP-8 program.
///
/// # Arguments
///
/// * `input` - The input stream to read from.
/// * `output` - The output stream to write to.
///
/// # Returns
///
/// * `Ok(())` - If the program was successfully assembled.
/// * `Err(Error)` - If the program was not successfully assembled.
///
/// # Examples
///
/// ```ignore
/// use std::fs::File;
/// use std::io::BufReader;
///
/// let mut input = BufReader::new(File::open("examples/pong.8s").unwrap());
/// let mut output = File::create("examples/pong.8o").unwrap();
/// match r8::assembler::assemble(&mut input, &mut output) {
///    Ok(_) => {},
///    Err(err) => println!("{}", err),
/// }
/// ```
pub fn assemble<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<(), error::Error> {
  let mut src = String::new();
  input.read_to_string(&mut src)?;
  let tokenizer = tokenizer::Tokenizer::new(&src);
  let mut address = 0x200;
  let mut labels: HashMap<&str, u16> = HashMap::new();
  let slices = tokenizer
    .map(|src_line| match src_line {
      Ok(src_line) => cast_line(src_line, &mut address, &mut labels),
      Err(err) => Err(err),
    })
    .collect::<Vec<Result<MemorySlices, error::Error>>>();
  for result in slices {
    let slice = result?;
    slice.write(&labels, output)?;
  }

  Ok(())
}

/// Converts a line of Tokens into a Raw opcode
///
/// # Arguments
///
/// * `line` - The line to convert
/// * `address` - The current address of the line
/// * `labels` - The labels defined in the program
///
/// # Returns
///
/// * `Ok(MemorySlices)` - The memory slices that make up the line
/// * `Err(Error)` - If there was an error converting the line
#[allow(clippy::match_ref_pats)]
fn cast_line<'src>(
  line: Line<'src>,
  address: &mut u16,
  labels: &mut HashMap<&'src str, u16>,
) -> Result<MemorySlices<'src>, error::Error> {
  // Match macro helpers
  macro_rules! id {
    ($id:ident) => {
      tokenizer::Token::Identifier($id)
    };
    ($id:expr) => {
      tokenizer::Token::Identifier($id)
    };
  }
  macro_rules! register {
    ($reg:ident) => {
      tokenizer::Token::Register($reg)
    };
    (0) => {
      tokenizer::Token::Register(0)
    };
  }
  macro_rules! num {
    ($num:ident) => {
      tokenizer::Token::Number($num)
    };
  }
  macro_rules! label {
    ($label:ident) => {
      tokenizer::Token::Label($label)
    };
  }
  macro_rules! comma {
    () => {
      tokenizer::Token::Comma
    };
  }

  // Num checkers
  macro_rules! addr {
    ($num:expr) => {{
      if $num > 0x0FFF {
        return Err(error::Error::InvalidAddress($num, line.line));
      }
      $num
    }};
  }
  macro_rules! byte {
    ($num:expr) => {{
      if $num > 0xFF {
        return Err(error::Error::InvalidByte($num as u16, line.line));
      }
      $num as u8
    }};
  }
  macro_rules! nibble {
    ($num:expr) => {{
      if $num > 0xF {
        return Err(error::Error::InvalidNibble($num as u16, line.line));
      }
      $num as u8
    }};
  }

  // Opcode helpers
  macro_rules! op_sxyn {
    ($s:expr, $x:expr, $y:expr, $n:expr) => {{
      *address += 2;
      let (s, x, y, n) = (nibble!($s), nibble!($x), nibble!($y), nibble!($n));
      Ok(MemorySlices::Opcode(
        ((s as u16) << 12) | ((x as u16) << 8) | ((y as u16) << 4) | n as u16,
      ))
    }};
  }

  macro_rules! op_snnn {
    ($s:expr, $nnn:expr) => {{
      *address += 2;
      let (s, nnn) = (nibble!($s), addr!($nnn));
      Ok(MemorySlices::Opcode(((s as u16) << 12) | nnn))
    }};
  }

  macro_rules! op_sxkk {
    ($s:expr, $x:expr, $kk:expr) => {{
      *address += 2;
      let (s, x, kk) = ($s, $x, byte!($kk));
      Ok(MemorySlices::Opcode(
        ((s as u16) << 12) | ((x as u16) << 8) | kk as u16,
      ))
    }};
  }

  macro_rules! op_slabel {
    ($s:expr, $label:ident) => {{
      *address += 2;
      if labels.contains_key($label) {
        let addr = labels[$label];
        Ok(MemorySlices::Opcode((($s as u16) << 12) | addr))
      } else {
        Ok(MemorySlices::Pending($s, $label, line.line))
      }
    }};
  }

  match line.tokens.as_slice() {
    &[] => Ok(MemorySlices::Empty),
    &[label!(label)] => {
      if labels.contains_key(label) {
        return Err(error::Error::DuplicateLabel(label.to_string(), line.line));
      }
      labels.insert(label, *address);
      Ok(MemorySlices::Empty)
    }
    // 00E0 - CLS
    &[id!("CLS")] => op_sxyn!(0x0, 0x0, 0xE, 0x0),
    // 00EE - RET
    &[id!("RET")] => op_sxyn!(0x0, 0x0, 0xE, 0xE),
    // 0NNN - SYS NNN
    &[id!("SYS"), num!(addr)] => op_snnn!(0x00, addr),
    // 0NNN - SYS :label
    &[id!("SYS"), id!(lb)] => op_slabel!(0x0, lb),
    // 1NNN - JP NNN
    &[id!("JP"), num!(addr)] => op_snnn!(0x1, addr),
    // 1NNN - JP :label
    &[id!("JP"), id!(lb)] => op_slabel!(0x1, lb),
    // 2NNN - CALL NNN
    &[id!("CALL"), num!(addr)] => op_snnn!(0x2, addr),
    // 2NNN - CALL :label
    &[id!("CALL"), id!(lb)] => op_slabel!(0x2, lb),
    // 3XKK - SE VX, KK
    &[id!("SE"), register!(x), comma!(), num!(kk)] => op_sxkk!(0x3, x, kk),
    // 4XKK - SNE VX, KK
    &[id!("SNE"), register!(x), comma!(), num!(kk)] => op_sxkk!(0x4, x, kk),
    // 5XY0 - SE VX, VY
    &[id!("SE"), register!(x), comma!(), register!(y)] => op_sxyn!(0x5, x, y, 0x0),
    // 6XKK - LD VX, KK
    &[id!("LD"), register!(x), comma!(), num!(kk)] => op_sxkk!(0x6, x, kk),
    // 7XKK - ADD VX, KK
    &[id!("ADD"), register!(x), comma!(), num!(kk)] => op_sxkk!(0x7, x, kk),
    // 8XY0 - LD VX, VY
    &[id!("LD"), register!(x), comma!(), register!(y)] => op_sxyn!(0x8, x, y, 0x00),
    // 8XY1 - OR VX, VY
    &[id!("OR"), register!(x), comma!(), register!(y)] => op_sxyn!(0x8, x, y, 0x01),
    // 8XY2 - AND VX, VY
    &[id!("AND"), register!(x), comma!(), register!(y)] => op_sxyn!(0x8, x, y, 0x02),
    // 8XY3 - XOR VX, VY
    &[id!("XOR"), register!(x), comma!(), register!(y)] => op_sxyn!(0x8, x, y, 0x03),
    // 8XY4 - ADD VX, VY
    &[id!("ADD"), register!(x), comma!(), register!(y)] => op_sxyn!(0x8, x, y, 0x04),
    // 8XY5 - SUB VX, VY
    &[id!("SUB"), register!(x), comma!(), register!(y)] => op_sxyn!(0x8, x, y, 0x05),
    // 8XY6 - SHR VX, VY
    &[id!("SHR"), register!(x)] => op_sxyn!(0x8, x, 0x1, 0x06),
    // 8XY7 - SUBN VX, VY
    &[id!("SUBN"), register!(x), comma!(), register!(y)] => {
      op_sxyn!(0x8, x, y, 0x07)
    }
    // 8XYE - SHL VX, VY
    &[id!("SHL"), register!(x)] => op_sxyn!(0x8, x, 0x0, 0x0E),
    // 9XY0 - SNE VX, VY
    &[id!("SNE"), register!(x), comma!(), register!(y)] => op_sxyn!(0x9, x, y, 0x00),
    // ANNN - LD I, NNN
    &[id!("LD"), id!("I"), comma!(), num!(addr)] => op_snnn!(0xA, addr),
    // ANNN - LD I, :label
    &[id!("LD"), id!("I"), comma!(), id!(lb)] => op_slabel!(0xA, lb),
    // BNNN - JP V0, NNN
    &[id!("JP"), register!(0), comma!(), num!(addr)] => op_snnn!(0xB, addr),
    // BNNN - JP V0, :label
    &[id!("JP"), register!(0), comma!(), id!(lb)] => op_slabel!(0xB, lb),
    // CXKK - RND VX, KK
    &[id!("RND"), register!(x), comma!(), num!(kk)] => op_sxkk!(0xC, x, kk),
    // DXYN - DRW VX, VY, N
    &[id!("DRW"), register!(x), comma!(), register!(y), comma!(), num!(n)] => {
      op_sxyn!(0xD, x, y, n)
    }
    // EX9E - SKP VX
    &[id!("SKP"), register!(x)] => op_sxyn!(0xE, x, 0x9, 0xE),
    // EXA1 - SKNP VX
    &[id!("SKNP"), register!(x)] => op_sxyn!(0xE, x, 0xA, 0x1),
    // FX07 - LD VX, DT
    &[id!("LD"), register!(x), comma!(), id!("DT")] => op_sxyn!(0xF, x, 0x0, 0x7),
    // FX0A - LD VX, K
    &[id!("LD"), register!(x), comma!(), id!("K")] => op_sxyn!(0xF, x, 0x0, 0xA),
    // FX15 - LD DT, VX
    &[id!("LD"), id!("DT"), comma!(), register!(x)] => op_sxyn!(0xF, x, 0x1, 0x5),
    // FX18 - LD ST, VX
    &[id!("LD"), id!("ST"), comma!(), register!(x)] => op_sxyn!(0xF, x, 0x1, 0x8),
    // FX1E - ADD I, VX
    &[id!("ADD"), id!("I"), comma!(), register!(x)] => op_sxyn!(0xF, x, 0x1, 0xE),
    // FX29 - LD F, VX
    &[id!("LD"), id!("F"), comma!(), register!(x)] => op_sxyn!(0xF, x, 0x2, 0x9),
    // FX33 - LD B, VX
    &[id!("LD"), id!("B"), comma!(), register!(x)] => op_sxyn!(0xF, x, 0x3, 0x3),
    // FX55 - LD [I], VX
    &[id!("LD"), id!("[I]"), comma!(), register!(x)] => op_sxyn!(0xF, x, 0x5, 0x5),
    // FX65 - LD VX, [I]
    &[id!("LD"), register!(x), comma!(), id!("[I]")] => op_sxyn!(0xF, x, 0x6, 0x5),
    &[id!("DB"), num!(n)] => {
      *address += 1;
      Ok(MemorySlices::Byte(byte!(n)))
    }
    &[id!("DW"), num!(n)] => {
      *address += 2;
      Ok(MemorySlices::Word(n))
    }
    _ => Err(error::Error::InvalidLine((&line).into())),
  }
}
