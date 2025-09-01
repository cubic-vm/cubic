use std::str::FromStr;

#[derive(Clone)]
pub struct DataSize {
    bytes: usize,
}

impl DataSize {
    pub fn new(bytes: usize) -> Self {
        Self { bytes }
    }

    pub fn get_bytes(&self) -> usize {
        self.bytes
    }

    pub fn to_size(&self) -> String {
        match self.bytes.checked_ilog(1024) {
            Some(1) => format!("{:.1} KiB", self.bytes as f64 / 1024_f64.powf(1_f64)),
            Some(2) => format!("{:.1} MiB", self.bytes as f64 / 1024_f64.powf(2_f64)),
            Some(3) => format!("{:.1} GiB", self.bytes as f64 / 1024_f64.powf(3_f64)),
            Some(4) => format!("{:.1} TiB", self.bytes as f64 / 1024_f64.powf(4_f64)),
            _ => format!("{}   B", self.bytes as f64),
        }
    }
}

impl FromStr for DataSize {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let error = format!("Cannot parse data size '{value}'. The input should be a number followed by a letter (B, K, M, G or T) for bytes, kilobytes, etc. Example: 1G for one gigabyte.");

        if value.is_empty() {
            return Err(error);
        }

        let suffix: char = value.bytes().last().unwrap() as char;
        let size = &value[..value.len() - 1];
        let power = match suffix {
            'B' => 0,
            'K' => 1,
            'M' => 2,
            'G' => 3,
            'T' => 4,
            _ => return Err(error),
        };

        size.parse()
            .map(|size: usize| Self {
                bytes: size * 1024_usize.pow(power),
            })
            .map_err(|_| error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_to_size() {
        assert_eq!(&DataSize::new(1).to_size(), "1   B")
    }

    #[test]
    fn test_kilobyte_to_size() {
        assert_eq!(&DataSize::new(1024).to_size(), "1.0 KiB")
    }

    #[test]
    fn test_megabyte_to_size() {
        assert_eq!(&DataSize::new(1024_usize.pow(2)).to_size(), "1.0 MiB")
    }

    #[test]
    fn test_gigabyte_to_size() {
        assert_eq!(&DataSize::new(1024_usize.pow(3)).to_size(), "1.0 GiB")
    }

    #[test]
    fn test_terrabyte_to_size() {
        assert_eq!(&DataSize::new(1024_usize.pow(4)).to_size(), "1.0 TiB")
    }

    #[test]
    fn test_from_byte() {
        assert_eq!(DataSize::from_str("1B").unwrap().get_bytes(), 1)
    }

    #[test]
    fn test_from_kilobyte() {
        assert_eq!(DataSize::from_str("1K").unwrap().get_bytes(), 1024)
    }

    #[test]
    fn test_from_megabyte() {
        assert_eq!(
            DataSize::from_str("1M").unwrap().get_bytes(),
            1024_usize.pow(2)
        )
    }

    #[test]
    fn test_from_gigabyte() {
        assert_eq!(
            DataSize::from_str("1G").unwrap().get_bytes(),
            1024_usize.pow(3)
        )
    }

    #[test]
    fn test_from_terrabyte() {
        assert_eq!(
            DataSize::from_str("1T").unwrap().get_bytes(),
            1024_usize.pow(4)
        )
    }
}
