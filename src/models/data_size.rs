use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataSize {
    bytes: usize,
}

impl DataSize {
    pub const fn new(bytes: usize) -> Self {
        Self { bytes }
    }

    pub fn get_bytes(&self) -> usize {
        self.bytes
    }

    pub fn to_size(&self) -> String {
        let power = self.get_power();
        format!("{} {}", self.to_value_at(power), UNITS[power])
    }

    // Value rounded to the unit another size prints in, so a used size and a
    // total size can share one unit and read as a fraction.
    pub fn to_value_in(&self, total: &DataSize) -> u64 {
        self.to_value_at(total.get_power())
    }

    fn to_value_at(&self, power: usize) -> u64 {
        (self.bytes as f64 / 1024_f64.powi(power as i32)).round() as u64
    }

    // The unit index this size prints in.
    fn get_power(&self) -> usize {
        let bytes = self.bytes as f64;
        let mut power = (1..UNITS.len())
            .rev()
            .find(|power| bytes / 1024_f64.powi(*power as i32) >= 10_f64)
            .unwrap_or(0);

        // Keep the number to four digits, so 10000 B shows as 10 K.
        if power + 1 < UNITS.len() && bytes / 1024_f64.powi(power as i32) >= 10_000_f64 {
            power += 1;
        }

        power
    }
}

impl FromStr for DataSize {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let error = format!(
            "Cannot parse data size '{value}'. The input should be a number followed by a letter (B, K, M, G or T) for bytes, kilobytes, etc. Example: 1G for one gigabyte."
        );

        if value.is_empty() {
            return Err(error);
        }

        let suffix = value.chars().next_back().unwrap();
        let size = &value[..value.len() - suffix.len_utf8()];
        let power = match suffix {
            'B' => 0,
            'K' => 1,
            'M' => 2,
            'G' => 3,
            'T' => 4,
            _ => return Err(error),
        };

        size.parse::<usize>()
            .ok()
            .and_then(|size| size.checked_mul(1024_usize.pow(power)))
            .map(|bytes| Self { bytes })
            .ok_or(error)
    }
}

impl Serialize for DataSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.bytes as u64)
    }
}

impl<'de> Deserialize<'de> for DataSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            bytes: usize::deserialize(deserializer)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_a_size() {
        assert_eq!(DataSize::new(0).to_size(), "0 B");
        assert_eq!(DataSize::new(1024_usize.pow(2)).to_size(), "1024 K");
        assert_eq!(DataSize::new(10 * 1024_usize.pow(2)).to_size(), "10 M");
        assert_eq!(
            DataSize::new(107 * 1024_usize.pow(3) / 10).to_size(),
            "11 G"
        );
        assert_eq!(DataSize::new(9999).to_size(), "9999 B");
        assert_eq!(DataSize::new(10_000).to_size(), "10 K");
    }

    #[test]
    fn test_scales_the_value_to_another_unit() {
        let total = DataSize::new(100 * 1024_usize.pow(3));
        assert_eq!(DataSize::new(1024_usize.pow(3)).to_value_in(&total), 1);
        assert_eq!(
            DataSize::new(44 * 1024_usize.pow(3)).to_value_in(&total),
            44
        );
    }

    #[test]
    fn test_parse_a_size() {
        assert_eq!(DataSize::from_str("1B").unwrap().get_bytes(), 1);
        assert_eq!(DataSize::from_str("1K").unwrap().get_bytes(), 1024);
        assert_eq!(
            DataSize::from_str("1M").unwrap().get_bytes(),
            1024_usize.pow(2)
        );
        assert_eq!(
            DataSize::from_str("1G").unwrap().get_bytes(),
            1024_usize.pow(3)
        );
        assert_eq!(
            DataSize::from_str("1T").unwrap().get_bytes(),
            1024_usize.pow(4)
        );
    }

    #[test]
    fn test_reject_an_invalid_size() {
        for input in ["10€", "€", "99999999999999999T", "10", ""] {
            assert!(DataSize::from_str(input).is_err(), "input {input}");
        }
    }
}
