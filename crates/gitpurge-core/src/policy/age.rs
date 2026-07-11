//! Age threshold parsing.

use crate::model::AgeThreshold;
use std::time::Duration;

/// Parse a human-readable age threshold string (e.g. "1 year ago", "90d") into a `Duration`.
pub fn parse_age_threshold(s: &str) -> Result<Duration, String> {
    let clean = s.trim().to_lowercase().replace(" ago", "");
    let parts: Vec<&str> = clean.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty age threshold".to_string());
    }

    let (num_str, suffix) = if parts.len() == 1 {
        let first = parts[0];
        let num_end = first.chars().take_while(|c| c.is_ascii_digit()).count();
        if num_end == 0 || num_end == first.len() {
            return Err(format!("Invalid age threshold format: {}", first));
        }
        (&first[..num_end], &first[num_end..])
    } else {
        (parts[0], parts[1])
    };

    let count: u64 = num_str
        .parse()
        .map_err(|e| format!("Invalid number: {}. Error: {}", num_str, e))?;

    let duration = match suffix {
        "d" | "day" | "days" => Duration::from_secs(count * 24 * 3600),
        "w" | "week" | "weeks" => Duration::from_secs(count * 7 * 24 * 3600),
        "m" | "month" | "months" => Duration::from_secs(count * 30 * 24 * 3600),
        "y" | "year" | "years" => Duration::from_secs(count * 365 * 24 * 3600),
        _ => return Err(format!("Unknown duration suffix: {}", suffix)),
    };

    Ok(duration)
}

impl AgeThreshold {
    /// Try to construct a new `AgeThreshold` from a raw string.
    pub fn parse(raw: String) -> Result<Self, String> {
        let duration = parse_age_threshold(&raw)?;
        Ok(Self { raw, duration })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_age_threshold() {
        assert_eq!(
            parse_age_threshold("1 year ago").unwrap(),
            Duration::from_secs(365 * 24 * 3600)
        );
        assert_eq!(
            parse_age_threshold("90d").unwrap(),
            Duration::from_secs(90 * 24 * 3600)
        );
        assert_eq!(
            parse_age_threshold("3 weeks").unwrap(),
            Duration::from_secs(3 * 7 * 24 * 3600)
        );
        assert_eq!(
            parse_age_threshold("6 months ago").unwrap(),
            Duration::from_secs(6 * 30 * 24 * 3600)
        );
    }
}
