use crate::config::Config;
use crate::core::segments::{Segment, SegmentData};

/// Status line generator
pub struct StatusLineGenerator {
    segments: Vec<Box<dyn Segment>>,
}

impl StatusLineGenerator {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn add_segment(mut self, segment: Box<dyn Segment>) -> Self {
        self.segments.push(segment);
        self
    }

    pub fn generate(&self, input: &crate::config::InputData, config: &Config) -> String {
        let mut parts = Vec::new();

        for segment in &self.segments {
            if !segment.is_enabled(config) {
                continue;
            }

            if let Some(data) = segment.collect(input, config) {
                let formatted = Self::format_segment(&data);
                parts.push(formatted);
            }
        }

        if parts.is_empty() {
            return String::new();
        }

        let separator = &config.style.separator;
        parts.join(separator)
    }

    fn format_segment(data: &SegmentData) -> String {
        let mut output = String::new();

        // Apply color
        if let Some(color_256) = data.style.color_256 {
            output.push_str(&format!("\x1b[38;5;{}m", color_256));
        } else if let Some((r, g, b)) = data.style.color {
            output.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
        }

        // Apply bold
        if data.style.bold {
            output.push_str("\x1b[1m");
        }

        // Add text
        output.push_str(&data.text);

        // Reset
        output.push_str("\x1b[0m");

        output
    }
}

impl Default for StatusLineGenerator {
    fn default() -> Self {
        Self::new()
    }
}
