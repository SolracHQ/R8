use std::{collections::HashMap, io::Write};

use super::error;

#[derive(Debug)]
pub enum MemorySlices<'src> {
    Opcode(u16),
    Pending(u8, &'src str, usize),
    Byte(u8),
    Word(u16),
    Empty,
}

impl MemorySlices<'_> {
    /// Write the memory slice to the writer.
    /// 
    /// # Arguments
    /// 
    /// * `labels` - The labels to use for the pending memory slices.
    /// * `writer` - The writer to write to.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the write was successful.
    /// * `Err(Error)` - If there was an error writing to the writer or if a label was undefined.
    pub fn write<W: Write>(
        self,
        labels: &HashMap<&str, u16>,
        writer: &mut W,
    ) -> Result<(), error::Error> {
        match self {
            MemorySlices::Opcode(data) | MemorySlices::Word(data) => {
                writer.write_all(&data.to_be_bytes())?;
                Ok(())
            }
            MemorySlices::Pending(first_nibble, label, line) => {
                if let Some(&addr) = labels.get(label) {
                    let addr = addr & 0x0FFF; // Mask out upper bits
                    let opcode = (first_nibble as u16) << 12 | addr; // Combine first nibble and address
                    writer.write_all(&opcode.to_be_bytes())?;
                    Ok(())
                } else {
                    Err(error::Error::UndefinedLabel(label.to_string(), line))
                }
            }
            MemorySlices::Byte(data) => {
                writer.write_all(&[data])?;
                Ok(())
            }
            MemorySlices::Empty => Ok(()),
        }
    }
}
