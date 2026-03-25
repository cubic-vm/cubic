use std::collections::HashMap;

use crate::iso9660::{
    BinaryWriter, DirRecordFactory, PrimaryVolumeDesc, SECTOR_SIZE, TermVolumeDesc,
};
use std::fs::File;
use std::io::{self, BufWriter};

pub struct IsoWriter {
    pub pvd: PrimaryVolumeDesc,
    pub files: HashMap<String, Vec<u8>>,
}

impl IsoWriter {
    pub fn new() -> Self {
        Self {
            pvd: PrimaryVolumeDesc::new(),
            files: HashMap::new(),
        }
    }

    pub fn create_iso(&self, output_path: &str) -> io::Result<()> {
        let file = File::create(output_path)?;
        let writer = &mut BinaryWriter::new(BufWriter::new(file));

        // Sectors 0 - 15: Boot sector
        writer.skip(16u32 * SECTOR_SIZE as u32)?;

        // Sector 16: Primary Volume Descriptor
        let mut pvd = self.pvd.clone();
        pvd.volume_space_size = 21;
        pvd.volume_set_size = 1;
        pvd.volume_sequence_number = 1;
        pvd.root_dir.len = 34;
        pvd.root_dir.file_flags = 0x02;
        pvd.root_dir.extend_loc = 18;
        pvd.root_dir.data_len = SECTOR_SIZE as u32;
        pvd.root_dir.volume_sequence_number = 1;
        pvd.write(writer)?;

        // Sector 17: Volume Descriptor Set Terminator
        TermVolumeDesc::new().write(writer)?;

        // Sector 18: Root Directory Entries
        let fac = DirRecordFactory::new();

        fac.create_dir("\x00", 18, SECTOR_SIZE as u32)
            .write(writer)?;
        fac.create_dir("\x01", 18, SECTOR_SIZE as u32)
            .write(writer)?;

        for (index, (name, content)) in self.files.iter().enumerate() {
            fac.create_file(
                &format!("{name};1"),
                19 + index as u32,
                content.len() as u32,
            )
            .write(writer)?;
        }

        writer.pad_to_sector_end()?;

        // Sector 19+: File Data
        for content in self.files.values() {
            writer.write_bytes(content)?;
            writer.pad_to_sector_end()?;
        }

        Ok(())
    }
}
