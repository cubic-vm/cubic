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
}
