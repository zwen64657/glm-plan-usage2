pub mod glm_usage;

use crate::config::{Config, InputData};

/// Segment data for rendering
#[derive(Debug, Clone)]
pub struct SegmentData {
    pub text: String,
    pub style: SegmentStyle,
}

#[derive(Debug, Clone, Default)]
pub struct SegmentStyle {
    pub color: Option<(u8, u8, u8)>,
    pub color_256: Option<u8>,
    pub bold: bool,
}

/// Segment trait
pub trait Segment: Send + Sync {
    /// Get segment identifier
    fn id(&self) -> &str;

    /// Check if segment is enabled
    fn is_enabled(&self, config: &Config) -> bool {
        config
            .segments
            .iter()
            .find(|s| s.id == self.id())
            .map(|s| s.enabled)
            .unwrap_or(false)
    }

    /// Collect segment data
    fn collect(&self, input: &InputData, config: &Config) -> Option<SegmentData>;
}
