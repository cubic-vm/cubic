pub fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode_foobar() {
        assert_eq!(hex_encode("foobar".as_bytes()).as_str(), "666f6f626172");
    }

    #[test]
    fn test_hex_encode_pads_small_bytes_with_zero() {
        assert_eq!(hex_encode(&[0x00, 0x0f, 0xa0]).as_str(), "000fa0");
    }

    #[test]
    fn test_hex_encode_empty_input() {
        assert_eq!(hex_encode(&[]).as_str(), "");
    }
}
