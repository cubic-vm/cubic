use crate::iso9660::{BinaryWriter, DirRecord, SECTOR_SIZE, VolumeDesc};
use std::io::{self, Seek, Write};

#[derive(Clone, Default)]
pub struct PrimaryVolumeDesc {
    pub vd: VolumeDesc,
    pub system_id: String,
    pub volume_id: String,
    pub volume_space_size: u32,
    pub volume_set_size: u16,
    pub volume_sequence_number: u16,
    pub logical_block_size: u16,
    pub path_table_size: u32,
    pub lpath_table_loc: u32,
    pub optional_lpath_table_loc: u32,
    pub mpath_table_loc: u32,
    pub optional_mpath_table_loc: u32,
    pub root_dir: DirRecord,
    pub volume_set_id: String,
    pub publisher_id: String,
    pub data_prepare_id: String,
    pub application_id: String,
    pub copyright_file_id: String,
    pub abstract_file_id: String,
    pub bibliographic_file_id: String,
    pub file_structure_version: u8,
}

impl PrimaryVolumeDesc {
    pub fn new() -> Self {
        Self {
            vd: VolumeDesc::new(0x01),
            logical_block_size: SECTOR_SIZE as u16,
            file_structure_version: 1,
            ..Default::default()
        }
    }

    pub fn write<T: Write + Seek>(&self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        self.vd.write(writer)?;
        writer.skip(1)?; // Unused
        writer.write_padded_string(&self.system_id, 32)?;
        writer.write_padded_string(&self.volume_id, 32)?;
        writer.skip(8)?; // Unused
        writer.write_u32_le_be(self.volume_space_size)?;
        writer.skip(32)?; // Unused
        writer.write_u16_le_be(self.volume_set_size)?;
        writer.write_u16_le_be(self.volume_sequence_number)?;
        writer.write_u16_le_be(self.logical_block_size)?;
        writer.write_u32_le_be(self.path_table_size)?;
        writer.write_u32_le(self.lpath_table_loc)?;
        writer.write_u32_le(self.optional_lpath_table_loc)?;
        writer.write_u32_be(self.mpath_table_loc)?;
        writer.write_u32_be(self.optional_mpath_table_loc)?;
        self.root_dir.write(writer)?;
        writer.write_padded_string(&self.volume_set_id, 128)?;
        writer.write_padded_string(&self.publisher_id, 128)?;
        writer.write_padded_string(&self.data_prepare_id, 128)?;
        writer.write_padded_string(&self.application_id, 128)?;
        writer.write_padded_string(&self.copyright_file_id, 37)?;
        writer.write_padded_string(&self.abstract_file_id, 37)?;
        writer.write_padded_string(&self.bibliographic_file_id, 37)?;
        writer.skip(17)?; // Volume Creation Date and Time
        writer.skip(17)?; // Volume Modification Date and Time
        writer.skip(17)?; // Volume Expiration Date and Time
        writer.skip(17)?; // Volume Effective Date and Time
        writer.write_byte(self.file_structure_version)?;
        writer.skip(1)?; // Unused
        writer.skip(512)?; // Application Unused
        writer.skip(652)?; // Reserved
        writer.write_byte(0)?; // Reserved
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write() {
        let mut pvd = PrimaryVolumeDesc::default();
        pvd.vd = VolumeDesc::new(0x01);
        pvd.system_id = "system id".to_string();
        pvd.volume_id = "volume id".to_string();
        pvd.volume_space_size = 0x12345678;
        pvd.volume_set_size = 0x2345;
        pvd.volume_sequence_number = 0x3456;
        pvd.logical_block_size = 0x4567;
        pvd.path_table_size = 0x56789ABC;
        pvd.lpath_table_loc = 0x6789ABCD;
        pvd.optional_lpath_table_loc = 0x789ABCDE;
        pvd.mpath_table_loc = 0x89ABCDEF;
        pvd.optional_mpath_table_loc = 0x9ABCDEF0;
        pvd.root_dir = DirRecord::default();
        pvd.volume_set_id = "volume set id".to_string();
        pvd.publisher_id = "publisher id".to_string();
        pvd.data_prepare_id = "data prepare id".to_string();
        pvd.application_id = "application id".to_string();
        pvd.copyright_file_id = "copyright file id".to_string();
        pvd.abstract_file_id = "abstract file id".to_string();
        pvd.bibliographic_file_id = "bibliographic file id".to_string();
        pvd.file_structure_version = 0xAB;

        let writer = &mut BinaryWriter::new(Cursor::new(Vec::new()));
        pvd.write(writer).unwrap();

        let result = writer.get_writer().get_ref();
        assert_eq!(result[0], 1);
        assert_eq!(&result[1..6], "CD001".as_bytes());
        assert_eq!(result[6], 1);
        assert_eq!(result[7], 0);
        assert_eq!(
            &result[8..40],
            "system id                       ".as_bytes()
        );
        assert_eq!(
            &result[40..72],
            "volume id                       ".as_bytes()
        );
        assert_eq!(&result[72..80], &[0u8; 8]);
        assert_eq!(&result[80..84], &[0x78, 0x56, 0x34, 0x12]);
        assert_eq!(&result[84..88], &[0x12, 0x34, 0x56, 0x78]);
        assert_eq!(&result[88..120], &[0u8; 32]);
        assert_eq!(&result[120..122], &[0x45, 0x23]);
        assert_eq!(&result[122..124], &[0x23, 0x45]);
        assert_eq!(&result[124..126], &[0x56, 0x34]);
        assert_eq!(&result[126..128], &[0x34, 0x56]);
        assert_eq!(&result[128..130], &[0x67, 0x45]);
        assert_eq!(&result[130..132], &[0x45, 0x67]);
        assert_eq!(&result[132..136], &[0xBC, 0x9A, 0x78, 0x56]);
        assert_eq!(&result[136..140], &[0x56, 0x78, 0x9A, 0xBC]);
        assert_eq!(&result[140..144], &[0xCD, 0xAB, 0x89, 0x67]);
        assert_eq!(&result[144..148], &[0xDE, 0xBC, 0x9A, 0x78]);
        assert_eq!(&result[148..152], &[0x89, 0xAB, 0xCD, 0xEF]);
        assert_eq!(&result[152..156], &[0x9A, 0xBC, 0xDE, 0xF0]);
        //assert_eq!(&result[156..190], ...);
        assert_eq!(&result[190..318], "volume set id                                                                                                                   ".as_bytes());
        assert_eq!(&result[318..446], "publisher id                                                                                                                    ".as_bytes());
        assert_eq!(&result[446..574], "data prepare id                                                                                                                 ".as_bytes());
        assert_eq!(&result[574..702], "application id                                                                                                                  ".as_bytes());
        assert_eq!(
            &result[702..739],
            "copyright file id                    ".as_bytes()
        );
        assert_eq!(
            &result[739..776],
            "abstract file id                     ".as_bytes()
        );
        assert_eq!(
            &result[776..813],
            "bibliographic file id                ".as_bytes()
        );
        assert_eq!(&result[813..830], &[0u8; 17]);
        assert_eq!(&result[830..847], &[0u8; 17]);
        assert_eq!(&result[847..864], &[0u8; 17]);
        assert_eq!(&result[864..881], &[0u8; 17]);
        assert_eq!(result[881], 0xAB);
        assert_eq!(result[882], 0);
        assert_eq!(&result[883..1395], &[0u8; 512]);
        assert_eq!(&result[1395..2048], &[0u8; 653]);
    }
}
