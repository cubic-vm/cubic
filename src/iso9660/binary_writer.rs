use std::cmp::min;
use std::io::{self, Seek, SeekFrom, Write};

pub const SECTOR_SIZE: usize = 2048;

pub struct BinaryWriter<T: Write + Seek> {
    out: T,
}

impl<T: Write + Seek> BinaryWriter<T> {
    pub fn new(out: T) -> Self {
        Self { out }
    }

    pub fn write_byte(&mut self, byte: u8) -> io::Result<u64> {
        self.out.write_all(&[byte])?;
        self.out.stream_position()
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<u64> {
        self.out.write_all(bytes)?;
        self.out.stream_position()
    }

    pub fn write_padded_string(&mut self, string: &str, len: usize) -> io::Result<u64> {
        let string_len = min(string.len(), len);
        self.out
            .write_all(format!("{:<len$}", &string[..string_len]).as_bytes())?;
        self.out.stream_position()
    }

    pub fn write_u32_le(&mut self, value: u32) -> io::Result<u64> {
        self.out.write_all(&value.to_le_bytes())?;
        self.out.stream_position()
    }

    pub fn write_u32_be(&mut self, value: u32) -> io::Result<u64> {
        self.out.write_all(&value.to_be_bytes())?;
        self.out.stream_position()
    }

    pub fn write_u32_le_be(&mut self, value: u32) -> io::Result<u64> {
        self.write_u32_le(value)?;
        self.write_u32_be(value)?;
        self.out.stream_position()
    }

    pub fn write_u16_le_be(&mut self, value: u16) -> io::Result<u64> {
        self.out.write_all(&value.to_le_bytes())?;
        self.out.write_all(&value.to_be_bytes())?;
        self.out.stream_position()
    }

    pub fn skip(&mut self, bytes: u32) -> io::Result<u64> {
        self.out.seek(SeekFrom::Current(bytes as i64))?;
        self.out.stream_position()
    }

    pub fn pad_to_sector_end(&mut self) -> io::Result<u64> {
        let padding = SECTOR_SIZE as u64 - (self.out.stream_position()? % SECTOR_SIZE as u64);

        if padding > 0 {
            if padding > 1 {
                self.skip(padding as u32 - 1)?;
            }
            self.write_byte(0x00)?;
        }
        self.out.stream_position()
    }

    #[cfg(test)]
    pub fn get_writer(&mut self) -> &mut T {
        &mut self.out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_one_byte() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_byte(1).unwrap(), 1);
        assert_eq!(writer.get_writer().get_ref(), &[1]);
    }

    #[test]
    fn test_write_two_bytes() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_byte(1).unwrap(), 1);
        assert_eq!(writer.write_byte(2).unwrap(), 2);
        assert_eq!(writer.get_writer().get_ref(), &[1, 2]);
    }

    #[test]
    fn test_write_empty_chunk() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_bytes(&[]).unwrap(), 0);
        assert_eq!(writer.get_writer().get_ref(), &[0u8; 0]);
    }

    #[test]
    fn test_write_one_chunk() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_bytes(&[1, 2, 3, 4, 5]).unwrap(), 5);
        assert_eq!(writer.get_writer().get_ref(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_write_two_chunks() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_bytes(&[1, 2, 3, 4, 5]).unwrap(), 5);
        assert_eq!(writer.write_bytes(&[5, 4, 3, 2, 1]).unwrap(), 10);
        assert_eq!(
            writer.get_writer().get_ref(),
            &[1, 2, 3, 4, 5, 5, 4, 3, 2, 1]
        );
    }

    #[test]
    fn test_write_empty_padded_string() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_padded_string("", 0).unwrap(), 0);
        assert_eq!(writer.get_writer().get_ref(), &[0u8; 0]);
    }

    #[test]
    fn test_write_one_padded_string() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_padded_string("foo", 3).unwrap(), 3);
        assert_eq!(writer.get_writer().get_ref(), &[102, 111, 111]);
    }

    #[test]
    fn test_write_two_padded_strings() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_padded_string("foo", 3).unwrap(), 3);
        assert_eq!(writer.write_padded_string("bar", 3).unwrap(), 6);
        assert_eq!(writer.get_writer().get_ref(), &[102, 111, 111, 98, 97, 114]);
    }

    #[test]
    fn test_write_three_letters_padded_to_ten() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_padded_string("foo", 10).unwrap(), 10);
        assert_eq!(
            writer.get_writer().get_ref(),
            &[102, 111, 111, 32, 32, 32, 32, 32, 32, 32]
        );
    }

    #[test]
    fn test_write_six_letters_padded_to_three() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_padded_string("foobar", 3).unwrap(), 3);
        assert_eq!(writer.get_writer().get_ref(), &[102, 111, 111]);
    }

    #[test]
    fn test_write_u32_le() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_u32_le(0x12345678).unwrap(), 4);
        assert_eq!(writer.get_writer().get_ref(), &[0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_write_u32_be() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_u32_be(0x12345678).unwrap(), 4);
        assert_eq!(writer.get_writer().get_ref(), &[0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_write_u32_le_be() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_u32_le_be(0x12345678).unwrap(), 8);
        assert_eq!(
            writer.get_writer().get_ref(),
            &[0x78, 0x56, 0x34, 0x12, 0x12, 0x34, 0x56, 0x78]
        );
    }

    #[test]
    fn test_write_u16_le_be() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_u16_le_be(0x1234).unwrap(), 4);
        assert_eq!(writer.get_writer().get_ref(), &[0x34, 0x12, 0x12, 0x34]);
    }

    #[test]
    fn test_skip() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.skip(1).unwrap(), 1);
        assert_eq!(writer.skip(1).unwrap(), 2);
        assert_eq!(writer.skip(0).unwrap(), 2);
        assert_eq!(writer.skip(500).unwrap(), 502);
    }

    #[test]
    fn test_write_one_byte_and_pad_sector() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.write_byte(1).unwrap(), 1);
        assert_eq!(writer.pad_to_sector_end().unwrap(), 2048);
    }

    #[test]
    fn test_skip_3000_bytes_and_pad_sector() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        assert_eq!(writer.skip(3000).unwrap(), 3000);
        assert_eq!(writer.pad_to_sector_end().unwrap(), 4096);
    }
}
