const SECTOR_SIZE: usize = 2048;
const ROOT_SECTOR: u32 = 18;
const SYSTEM_ID: &str = "LINUX";
const APPLICATION_ID: &str = "Cubic";

pub struct IsoImage;

impl IsoImage {
    // Layout: 16 empty sectors, primary descriptor, terminator, root directory, file data
    pub fn build(volume_id: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
        let root_size = SECTOR_SIZE as u32;
        let parent = Self::build_dir_record(b"\x01", ROOT_SECTOR, root_size, true);
        let mut records = Self::build_dir_record(b"\0", ROOT_SECTOR, root_size, true);
        records.extend(parent);

        let mut sector = ROOT_SECTOR + 1;
        for (name, data) in files {
            let id = format!("{name};1");
            let size = data.len() as u32;
            records.extend(Self::build_dir_record(id.as_bytes(), sector, size, false));
            sector += Self::get_sector_count(data);
        }

        let mut image = vec![0; 16 * SECTOR_SIZE];
        image.extend(Self::build_primary_desc(volume_id, sector));
        image.extend(Self::build_desc_header(0xFF));
        Self::push_sectors(&mut image, &records);
        for (_, data) in files {
            Self::push_sectors(&mut image, data);
        }
        image
    }

    fn build_primary_desc(volume_id: &str, volume_size: u32) -> [u8; SECTOR_SIZE] {
        let root = Self::build_dir_record(b"\0", ROOT_SECTOR, SECTOR_SIZE as u32, true);
        let mut desc = Self::build_desc_header(0x01);
        Self::put_str(&mut desc[8..40], SYSTEM_ID);
        Self::put_str(&mut desc[40..72], volume_id);
        Self::put_u32_both(&mut desc[80..88], volume_size);
        Self::put_u16_both(&mut desc[120..124], 1); // Volume set size
        Self::put_u16_both(&mut desc[124..128], 1); // Volume sequence number
        Self::put_u16_both(&mut desc[128..132], SECTOR_SIZE as u16);
        desc[156..190].copy_from_slice(&root);
        desc[190..813].fill(b' '); // Unused identifiers
        Self::put_str(&mut desc[574..702], APPLICATION_ID);
        desc[881] = 1; // File structure version
        desc
    }

    fn build_desc_header(kind: u8) -> [u8; SECTOR_SIZE] {
        let mut desc = [0; SECTOR_SIZE];
        desc[0] = kind;
        desc[1..7].copy_from_slice(b"CD001\x01");
        desc
    }

    fn build_dir_record(id: &[u8], sector: u32, size: u32, is_dir: bool) -> Vec<u8> {
        let padding = (id.len() + 1) % 2;
        let mut record = vec![0; 33 + id.len() + padding];
        record[0] = record.len() as u8;
        Self::put_u32_both(&mut record[2..10], sector);
        Self::put_u32_both(&mut record[10..18], size);
        record[25] = if is_dir { 0x02 } else { 0x00 };
        Self::put_u16_both(&mut record[28..32], 1); // Volume sequence number
        record[32] = id.len() as u8;
        record[33..33 + id.len()].copy_from_slice(id);
        record
    }

    fn get_sector_count(data: &[u8]) -> u32 {
        data.len().div_ceil(SECTOR_SIZE).max(1) as u32
    }

    fn push_sectors(image: &mut Vec<u8>, data: &[u8]) {
        let end = image.len() + Self::get_sector_count(data) as usize * SECTOR_SIZE;
        image.extend_from_slice(data);
        image.resize(end, 0);
    }

    fn put_str(field: &mut [u8], text: &str) {
        let len = text.len().min(field.len());
        field.fill(b' ');
        field[..len].copy_from_slice(&text.as_bytes()[..len]);
    }

    fn put_u16_both(field: &mut [u8], value: u16) {
        field[..2].copy_from_slice(&value.to_le_bytes());
        field[2..].copy_from_slice(&value.to_be_bytes());
    }

    fn put_u32_both(field: &mut [u8], value: u32) {
        field[..4].copy_from_slice(&value.to_le_bytes());
        field[4..].copy_from_slice(&value.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_u32(image: &[u8], pos: usize) -> usize {
        u32::from_le_bytes(image[pos..pos + 4].try_into().unwrap()) as usize
    }

    fn get_file_contents(image: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut files = Vec::new();
        let mut pos = ROOT_SECTOR as usize * SECTOR_SIZE;
        while image[pos] != 0 {
            let id_len = image[pos + 32] as usize;
            let id = image[pos + 33..pos + 33 + id_len].to_vec();
            let start = get_u32(image, pos + 2) * SECTOR_SIZE;
            let end = start + get_u32(image, pos + 10);
            if image[pos + 25] == 0 {
                files.push((id, image[start..end].to_vec()));
            }
            pos += image[pos] as usize;
        }
        files
    }

    #[test]
    fn test_build_writes_the_volume_descriptors() {
        let image = IsoImage::build("cidata", &[("meta-data", b"id")]);

        let primary = &image[16 * SECTOR_SIZE..17 * SECTOR_SIZE];
        assert_eq!(&primary[..7], b"\x01CD001\x01");
        assert_eq!(&primary[40..72], format!("{:<32}", "cidata").as_bytes());
        assert_eq!(get_u32(primary, 80), image.len() / SECTOR_SIZE);
        assert_eq!(&image[17 * SECTOR_SIZE..][..7], b"\xFFCD001\x01");
    }

    #[test]
    fn test_build_places_each_file_at_its_recorded_sector() {
        let large = vec![b'A'; 3000];
        let image = IsoImage::build("cidata", &[("large", &large), ("small", b"BBBB")]);

        let expected = vec![
            (b"large;1".to_vec(), large),
            (b"small;1".to_vec(), b"BBBB".to_vec()),
        ];
        assert_eq!(get_file_contents(&image), expected);
    }

    #[test]
    fn test_build_gives_a_full_sector_file_one_sector() {
        let data = vec![b'A'; SECTOR_SIZE];
        let image = IsoImage::build("cidata", &[("full", &data)]);

        assert_eq!(image.len(), 20 * SECTOR_SIZE);
    }
}
