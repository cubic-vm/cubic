use regex::Regex;
use std::cmp::Ordering;
use std::sync::LazyLock;

static CHUNK_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("[0-9]+|[^0-9]+").unwrap());

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

/// Compares digit runs as numbers, so 3.9 sorts before 3.22
pub fn compare_natural(a: &str, b: &str) -> Ordering {
    let chunks_a = CHUNK_REGEX.find_iter(a).map(|chunk| chunk.as_str());
    let chunks_b = CHUNK_REGEX.find_iter(b).map(|chunk| chunk.as_str());
    chunks_a
        .zip(chunks_b)
        .map(|(a, b)| match (a.parse::<u64>(), b.parse::<u64>()) {
            (Ok(a), Ok(b)) => a.cmp(&b),
            _ => a.cmp(b),
        })
        .find(|ordering| ordering.is_ne())
        .unwrap_or_else(|| a.cmp(b))
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
