use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub struct TimezoneName(String);

impl Default for TimezoneName {
    fn default() -> Self {
        Self("UTC".to_string())
    }
}

impl FromStr for TimezoneName {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        if !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "/_+-".contains(c))
        {
            Ok(Self(name.to_string()))
        } else {
            Err(format!("Invalid timezone name '{name}'"))
        }
    }
}

impl fmt::Display for TimezoneName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert_eq!(TimezoneName::default().to_string(), "UTC");
        assert_eq!(
            TimezoneName::from_str("Region/Some_City")
                .unwrap()
                .to_string(),
            "Region/Some_City"
        );
        assert!(TimezoneName::from_str("Etc/GMT+2").is_ok());
        assert!(TimezoneName::from_str("Etc/GMT-14").is_ok());
    }

    #[test]
    fn test_reject_invalid_names() {
        assert!(TimezoneName::from_str("").is_err());
        assert!(TimezoneName::from_str("Region/Some City").is_err());
        assert!(TimezoneName::from_str("Region/City # note").is_err());
        assert!(TimezoneName::from_str("Region/City\nruncmd: [reboot]").is_err());
    }
}
