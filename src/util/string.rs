use regex::Regex;
use std::cmp::Ordering;

pub fn find_and_extract(regex: &str, input: &str) -> Vec<String> {
    Regex::new(regex)
        .unwrap()
        .captures_iter(input)
        .map(|content| content.extract::<1>())
        .map(|(_, values)| values[0].to_string())
        .collect()
}

/// A `*` matches any characters within one file name
pub fn convert_glob_to_regex(glob: &str) -> String {
    regex::escape(glob).replace("\\*", "[^\"/]*")
}

/// Finds the newest file that a directory listing links to
pub fn find_newest_file(pattern: &str, listing: &str) -> Option<String> {
    find_and_extract(&format!("href=\"\\.?/?({pattern})\""), listing)
        .into_iter()
        .max_by(|a, b| compare_natural(a, b))
}

/// Compares digit runs as numbers, so 3.9 sorts before 3.22
pub fn compare_natural(a: &str, b: &str) -> Ordering {
    let is_same_kind = |a: &u8, b: &u8| a.is_ascii_digit() == b.is_ascii_digit();
    let parse = |chunk: &[u8]| str::from_utf8(chunk).ok()?.parse::<u64>().ok();
    a.as_bytes()
        .chunk_by(is_same_kind)
        .zip(b.as_bytes().chunk_by(is_same_kind))
        .map(|(a, b)| match (parse(a), parse(b)) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => a.cmp(b),
        })
        .find(|ordering| ordering.is_ne())
        .unwrap_or_else(|| a.cmp(b))
}

pub fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

pub fn to_yes_no(condition: bool) -> &'static str {
    if condition { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_extract() {
        assert_eq!(
            &find_and_extract(
                ">([a-z]+)/<",
                "<p>buster/</p>\n<p>bookworm/</p>\n<p>trixie/</p>\n"
            ),
            &["buster", "bookworm", "trixie"]
        )
    }

    #[test]
    fn test_find_extract_without_match() {
        assert!(find_and_extract(">([a-z]+)/<", "no listing here").is_empty());
    }

    #[test]
    fn test_convert_glob_to_regex_matches_one_file_name() {
        let regex =
            Regex::new(&format!("^{}$", convert_glob_to_regex("di-amd64-*.qcow2"))).unwrap();

        assert!(regex.is_match("di-amd64-20260920T170055Z.qcow2"));
        assert!(!regex.is_match("di-amd64-20260920T170055Z.qcow2.asc"));
        assert!(!regex.is_match("di-amd64-20260920T170055Z_qcow2"));
        assert!(!regex.is_match("di-amd64-dir/image.qcow2"));
    }

    #[test]
    fn test_find_newest_file_picks_the_newest_match() {
        let listing = r#"
<a href="image-20260630.2.qcow2">
<a href="image-20260630.10.qcow2">
<a href="image-20260630.10.qcow2.sha256">
<a href="image-20260629.12.qcow2">"#;

        assert_eq!(
            find_newest_file(&convert_glob_to_regex("image-*.qcow2"), listing),
            Some("image-20260630.10.qcow2".to_string())
        );
        assert_eq!(
            find_newest_file(&convert_glob_to_regex("debian-*.qcow2"), listing),
            None
        );
    }

    #[test]
    fn test_compare_natural_compares_digit_runs_as_numbers() {
        assert_eq!(compare_natural("3.9", "3.22"), Ordering::Less);
        assert_eq!(compare_natural("3.22.10", "3.22.9"), Ordering::Greater);
        assert_eq!(compare_natural("3.22", "3.22.1"), Ordering::Less);
    }

    #[test]
    fn test_to_yes_no() {
        assert_eq!(to_yes_no(true), "yes");
        assert_eq!(to_yes_no(false), "no");
    }
}
