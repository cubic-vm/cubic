use crate::iso9660::BinaryWriter;
use std::io::{self, Seek, Write};

#[derive(Clone, Default)]
pub struct VolumeDesc {
    pub typ: u8,
    pub id: String,
    pub version: u8,
}

impl VolumeDesc {
    pub fn new(typ: u8) -> Self {
        Self {
            typ,
            id: "CD001".to_string(),
            version: 1,
        }
    }

    pub fn write<T: Write + Seek>(&self, writer: &mut BinaryWriter<T>) -> io::Result<u64> {
        writer.write_byte(self.typ)?;
        writer.write_padded_string(&self.id, 5)?;
        writer.write_byte(self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_volume_desc_write() {
        let writer = &mut BinaryWriter::new(Cursor::new(Vec::new()));
        VolumeDesc::new(2).write(writer).unwrap();

        let result = writer.get_writer().get_ref();
        assert_eq!(result[0], 2);
        assert_eq!(&result[1..6], &[67, 68, 48, 48, 49]);
        assert_eq!(result[6], 1);
    }
}
