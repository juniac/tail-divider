use colored::{Color, Colorize};
use regex::Regex;
use std::sync::LazyLock;

static TIMESTAMP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?").unwrap());
static URL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://[^\s.,;:!?)]+").unwrap());
static ANSI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]").unwrap());

fn format_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub struct LineProcessor {
    timestamp_color: Color,
    url_color: Color,
    idle_color: Color,
}

impl LineProcessor {
    pub fn new(idle_color: Color) -> Self {
        Self {
            timestamp_color: Color::TrueColor { r: 128, g: 128, b: 128 },
            url_color: Color::TrueColor { r: 84, g: 117, b: 238 },
            idle_color,
        }
    }

    pub fn highlight(&self, line: &str) -> String {
        let mut result = line.to_string();
        for cap in TIMESTAMP_RE.find_iter(line) {
            let matched = cap.as_str();
            result = result.replace(matched, &matched.color(self.timestamp_color).to_string());
        }
        for cap in URL_RE.find_iter(line) {
            let matched = cap.as_str();
            result = result.replace(matched, &matched.color(self.url_color).to_string());
        }
        result
    }

    pub fn format_idle_line(&self, idle_count: u64) -> String {
        let message = format!("[{}]-[{}]----- idle -------", idle_count, format_timestamp());
        message.color(self.idle_color).to_string()
    }

    pub fn is_activity(&self, line: &str) -> bool {
        let without_ansi = ANSI_RE.replace_all(line, "");
        let without_controls: String =
            without_ansi.chars().filter(|ch| !ch.is_control()).collect();
        !without_controls.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn processor() -> LineProcessor {
        LineProcessor::new(Color::White)
    }

    #[test]
    fn is_activity_returns_false_for_ansi_only_line() {
        assert!(!processor().is_activity("\x1B[32m\x1B[0m"));
    }

    #[test]
    fn is_activity_returns_true_for_ansi_decorated_text() {
        assert!(processor().is_activity("\x1B[32mhello\x1B[0m"));
    }

    #[test]
    fn is_activity_returns_false_for_whitespace_only() {
        assert!(!processor().is_activity("   \t  "));
    }

    #[test]
    fn highlight_preserves_timestamp_and_surrounding_text() {
        let result = processor().highlight("2024-01-15T12:34:56Z event happened");
        assert!(result.contains("2024-01-15T12:34:56Z"));
        assert!(result.contains("event happened"));
    }

    #[test]
    fn highlight_does_not_capture_trailing_punctuation_in_url() {
        let result = processor().highlight("see https://example.com. done");
        assert!(result.contains(". done"));
    }
}
