use regex::Regex;

pub fn find_and_extract(regex: &str, input: &str) -> Vec<String> {
    Regex::new(regex)
        .unwrap()
        .captures_iter(input)
        .map(|content| content.extract::<1>())
        .map(|(_, values)| values[0].to_string())
        .collect()
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
    fn test_to_yes_no() {
        assert_eq!(to_yes_no(true), "yes");
        assert_eq!(to_yes_no(false), "no");
    }
}
