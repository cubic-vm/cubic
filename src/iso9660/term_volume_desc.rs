use crate::iso9660::{BinaryWriter, VolumeDesc};
use std::io::{self, Seek, Write};

#[derive(Default)]
pub struct TermVolumeDesc {
    pub vd: VolumeDesc,
}

impl TermVolumeDesc {
    pub fn new() -> Self {
        Self {
            vd: VolumeDesc::new(0xFF),
        }
    }

    pub fn write<T: Write + Seek>(&self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        self.vd.write(writer)?;
        writer.pad_to_sector_end()?; // Unused
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write() {
        let writer = &mut BinaryWriter::new(Cursor::new(Vec::new()));
        TermVolumeDesc::new().write(writer).unwrap();

        let result = writer.get_writer().get_ref();
        assert_eq!(result[0], 255);
        assert_eq!(&result[1..6], "CD001".as_bytes());
        assert_eq!(result[6], 1);
        assert_eq!(&result[7..2048], &[0u8; 2041]);
    }
}
