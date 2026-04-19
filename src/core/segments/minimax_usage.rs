use super::Segment;
use crate::api::minimax_client::MiniMaxApiClient;
use crate::api::minimax_types::MiniMaxUsageStats;
use crate::config::{Config, InputData};
use crate::core::segments::{SegmentData, SegmentStyle};
use crate::terminal::{CharMode, TerminalDetector};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Format reset time as absolute time (HH:MM)
/// Input is a millisecond timestamp from MiniMax API.
fn format_reset_time(reset_at_ms: i64) -> Option<String> {
    use chrono::{DateTime, Local, TimeZone, Timelike};
    let secs = reset_at_ms / 1000;
    let dt: DateTime<Local> = Local.timestamp_opt(secs, 0).single()?;
    Some(format!("{}:{:02}", dt.hour(), dt.minute()))
}

/// MiniMax usage segment with caching
pub struct MiniMaxUsageSegment {
    cache: Arc<Mutex<Option<CacheEntry>>>,
    char_mode: CharMode,
}

struct CacheEntry {
    stats: MiniMaxUsageStats,
    timestamp: Instant,
}

impl MiniMaxUsageSegment {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
            char_mode: TerminalDetector::detect(),
        }
    }

    fn get_usage_stats(&self, config: &Config) -> Option<MiniMaxUsageStats> {
        if config.cache.enabled {
            if let Some(entry) = self.cache.lock().unwrap().as_ref() {
                if entry.timestamp.elapsed() < Duration::from_secs(config.cache.ttl_seconds) {
                    return Some(entry.stats.clone());
                }
            }
        }

        let result: Option<MiniMaxUsageStats> = match MiniMaxApiClient::from_env() {
            Ok(client) => match client.fetch_usage_stats().ok() {
                Some(stats) => {
                    if config.cache.enabled {
                        *self.cache.lock().unwrap() = Some(CacheEntry {
                            stats: stats.clone(),
                            timestamp: Instant::now(),
                        });
                    }
                    Some(stats)
                }
                None => self.cache.lock().unwrap().as_ref().map(|e| e.stats.clone()),
            },
            Err(_) => None,
        };
        result
    }

    fn format_stats(stats: &MiniMaxUsageStats, char_mode: CharMode, prefix: &str) -> String {
        let minimal = std::env::var("USAGE_MINIMAL").is_ok();
        let (token_icon, clock_icon, chart_icon, calendar_icon) = if minimal {
            ("", "", "", "")
        } else {
            match char_mode {
                CharMode::Emoji => ("🔋", "⏰", "📊", "📅"),
                CharMode::Ascii => ("$", "T", "#", "%"),
            }
        };
        let sep = if minimal { "" } else { " " };

        let mut parts = Vec::new();

        // 5h interval percentage with reset time
        let reset_time = stats
            .reset_time
            .and_then(format_reset_time)
            .unwrap_or_else(|| "--:--".to_string());
        parts.push(format!(
            "{}{}{}% · {}{}{}",
            token_icon, sep, stats.interval_pct, clock_icon, sep, reset_time
        ));

        // Call count (used/total)
        parts.push(format!(
            "{}{}{}/{}",
            chart_icon, sep, stats.interval_used, stats.interval_total
        ));

        // Weekly percentage (only if weekly limit exists)
        if let Some(pct) = stats.weekly_pct {
            parts.push(format!("{}{}{}%", calendar_icon, sep, pct));
        }

        format!("{} {}", prefix, parts.join(" · "))
    }

}

impl Default for MiniMaxUsageSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl Segment for MiniMaxUsageSegment {
    fn id(&self) -> &str {
        "minimax_usage"
    }

    fn collect(&self, input: &InputData, config: &Config) -> Option<SegmentData> {
        // No model filtering - show MiniMax usage only if API is configured
        let stats = self.get_usage_stats(config)?;

        let model_name = input.model.as_ref().map(|m| m.id.as_str()).unwrap_or("MiniMax");
        let text = Self::format_stats(&stats, self.char_mode, model_name);
        if text.is_empty() {
            None
        } else {
            Some(SegmentData {
                text,
                style: SegmentStyle { color_256: Some(208), bold: true, color: None },
            })
        }
    }
}
