use std::fmt;
use std::time::{Duration, Instant};

pub struct TimeDuration {
    start: Instant,
}

impl TimeDuration {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    fn format_duration_string(duration: Duration) -> String {
        let minutes = duration.as_secs() / 60;
        let secondes = duration.as_secs() % 60;
        let millis = duration.as_millis() % 1000 / 100;

        let output_secs = format!("{secondes:2}.{millis}s");

        if minutes > 0 {
            format!("{minutes}m {output_secs}")
        } else {
            output_secs
        }
    }
}

impl fmt::Display for TimeDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Self::format_duration_string(self.start.elapsed()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seconds() {
        assert_eq!(
            TimeDuration::format_duration_string(Duration::from_millis(3_123)),
            " 3.1s"
        )
    }

    #[test]
    fn test_half_minute() {
        assert_eq!(
            TimeDuration::format_duration_string(Duration::from_secs(30)),
            "30.0s"
        )
    }

    #[test]
    fn test_minutes() {
        assert_eq!(
            TimeDuration::format_duration_string(Duration::from_millis(127_512)),
            "2m  7.5s"
        )
    }
}
