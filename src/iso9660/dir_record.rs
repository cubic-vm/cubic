use crate::iso9660::BinaryWriter;
use std::io::{self, Seek, Write};

#[derive(Clone, Default)]
pub struct DirRecord {
    pub len: u8,
    pub extend_attr_len: u8,
    pub extend_loc: u32,
    pub data_len: u32,
    pub file_flags: u8,
    pub file_unit_size: u8,
    pub interleave_gap_size: u8,
    pub volume_sequence_number: u16,
    pub file_id_len: u8,
    pub file_id: String,
}

impl DirRecord {
    pub fn write<T: Write + Seek>(&self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        writer.write_byte(self.len)?;
        writer.write_byte(self.extend_attr_len)?;
        writer.write_u32_le_be(self.extend_loc)?;
        writer.write_u32_le_be(self.data_len)?;
        writer.skip(7)?; // Recording date and time
        writer.write_byte(self.file_flags)?;
        writer.write_byte(self.file_unit_size)?;
        writer.write_byte(self.interleave_gap_size)?;
        writer.write_u16_le_be(self.volume_sequence_number)?;
        writer.write_byte(self.file_id_len)?;
        writer.write_bytes(self.file_id.as_bytes())?;

        // Padding
        if self.file_id_len.is_multiple_of(2) {
            writer.write_byte(0x00)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_dir_record() {
        let writer = &mut BinaryWriter::new(Cursor::new(Vec::new()));
        let mut dir = DirRecord::default();
        dir.len = 34;
        dir.extend_attr_len = 1;
        dir.extend_loc = 0xABCDEF01;
        dir.data_len = 0x98765432;
        dir.file_flags = 0x12;
        dir.file_unit_size = 0x23;
        dir.interleave_gap_size = 0xAB;
        dir.volume_sequence_number = 0xFEDC;
        dir.file_id_len = 6;
        dir.file_id = "foobar".to_string();
        dir.write(writer).unwrap();

        let result = writer.get_writer().get_ref();
        assert_eq!(result[0], 34);
        assert_eq!(result[1], 1);
        assert_eq!(&result[2..6], &[0x01, 0xEF, 0xCD, 0xAB]);
        assert_eq!(&result[6..10], &[0xAB, 0xCD, 0xEF, 0x01]);
        assert_eq!(&result[10..14], &[0x32, 0x54, 0x76, 0x98]);
        assert_eq!(&result[14..18], &[0x98, 0x76, 0x54, 0x32]);
        assert_eq!(&result[18..25], &[0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(result[25], 0x12);
        assert_eq!(result[26], 0x23);
        assert_eq!(result[27], 0xAB);
        assert_eq!(&result[28..30], &[0xDC, 0xFE]);
        assert_eq!(&result[30..32], &[0xFE, 0xDC]);
        assert_eq!(result[32], 6);
        assert_eq!(&result[33..39], &[102, 111, 111, 98, 97, 114]);
        assert_eq!(result[39], 0);
    }
}
