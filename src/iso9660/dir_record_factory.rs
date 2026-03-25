use crate::iso9660::DirRecord;

#[derive(Default)]
pub struct DirRecordFactory;

impl DirRecordFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn create_file(&self, name: &str, sector: u32, size: u32) -> DirRecord {
        let mut dir = DirRecord::default();
        dir.file_id = name.to_string();
        dir.file_id_len = dir.file_id.len() as u8;
        dir.len = 33 + dir.file_id.len() as u8;
        if name.len().is_multiple_of(2) {
            dir.len += 1;
        }
        dir.volume_sequence_number = 1;
        dir.extend_loc = sector;
        dir.data_len = size;
        dir
    }

    pub fn create_dir(&self, name: &str, sector: u32, size: u32) -> DirRecord {
        let mut dir = self.create_file(name, sector, size);
        dir.file_flags = 0x02;
        dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dir() {
        let dir = DirRecordFactory::new().create_dir("testdir", 20, 1024);
        assert_eq!(dir.len, 40);
        assert_eq!(dir.extend_attr_len, 0);
        assert_eq!(dir.extend_loc, 20);
        assert_eq!(dir.data_len, 1024);
        assert_eq!(dir.file_flags, 0x02);
        assert_eq!(dir.file_unit_size, 0);
        assert_eq!(dir.interleave_gap_size, 0);
        assert_eq!(dir.volume_sequence_number, 1);
        assert_eq!(dir.file_id_len, 7);
        assert_eq!(dir.file_id, "testdir");
    }

    #[test]
    fn test_create_file() {
        let dir = DirRecordFactory::new().create_file("testfile", 21, 500);
        assert_eq!(dir.len, 42);
        assert_eq!(dir.extend_attr_len, 0);
        assert_eq!(dir.extend_loc, 21);
        assert_eq!(dir.data_len, 500);
        assert_eq!(dir.file_flags, 0);
        assert_eq!(dir.file_unit_size, 0);
        assert_eq!(dir.interleave_gap_size, 0);
        assert_eq!(dir.volume_sequence_number, 1);
        assert_eq!(dir.file_id_len, 8);
        assert_eq!(dir.file_id, "testfile");
    }
}
