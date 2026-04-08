use super::Segment;
use crate::api::{GlmApiClient, UsageStats};
use crate::config::{Config, InputData};
use crate::core::segments::{SegmentData, SegmentStyle};
use crate::terminal::{CharMode, TerminalDetector};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Format token count with appropriate units (M/K/raw)
#[allow(dead_code)]
fn format_tokens(count: i64) -> String {
    if count < 0 {
        return "N/A".to_string();
    }
    if count >= 1_000_000 {
        format!("{:.2}M", count as f64 / 1_000_000.0)
    } else if count >= 10_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        format!("{}", count)
    }
}

/// Format reset time as absolute time (HH:MM)
fn format_reset_time(reset_at: i64) -> Option<String> {
    use chrono::{DateTime, Local, TimeZone, Timelike};
    let dt: DateTime<Local> = Local.timestamp_opt(reset_at, 0).single()?;
    Some(format!("{}:{:02}", dt.hour(), dt.minute()))
}

/// GLM usage segment with caching
pub struct GlmUsageSegment {
    cache: Arc<Mutex<Option<CacheEntry>>>,
    char_mode: CharMode,
}

struct CacheEntry {
    stats: UsageStats,
    timestamp: Instant,
}

impl GlmUsageSegment {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
            char_mode: TerminalDetector::detect(),
        }
    }

    fn get_usage_stats(&self, config: &Config) -> Option<UsageStats> {
        // Check cache first
        if config.cache.enabled {
            if let Some(entry) = self.cache.lock().unwrap().as_ref() {
                if entry.timestamp.elapsed() < Duration::from_secs(config.cache.ttl_seconds) {
                    return Some(entry.stats.clone());
                }
            }
        }

        // Fetch from API
        match GlmApiClient::from_env() {
            Ok(client) => {
                match client.fetch_usage_stats() {
                    Ok(stats) => {
                        // Update cache
                        if config.cache.enabled {
                            let entry = CacheEntry {
                                stats: stats.clone(),
                                timestamp: Instant::now(),
                            };
                            *self.cache.lock().unwrap() = Some(entry);
                        }
                        Some(stats)
                    }
                    Err(_) => {
                        // Return cached data if available
                        self.cache.lock().unwrap().as_ref().map(|e| e.stats.clone())
                    }
                }
            }
            Err(_) => None,
        }
    }

    fn format_stats(stats: &UsageStats, char_mode: CharMode, prefix: &str) -> String {
        let mut parts = Vec::new();
        let minimal = std::env::var("USAGE_MINIMAL").is_ok();

        // Character mapping: minimal mode strips all icons
        let (token_icon, clock_icon, chart_icon, calendar_icon, globe_icon, lightning_icon) = if minimal {
            ("", "", "", "", "", "")
        } else {
            match char_mode {
                CharMode::Emoji => ("🔋", "⏰", "📊", "📅", "🌐", "⚡"),
                CharMode::Ascii => ("$", "T", "#", "%", "M", "k"),
            }
        };

        let sep = if minimal { "" } else { " " };

        // Token usage with reset time
        if let Some(token) = &stats.token_usage {
            let reset_time = token
                .reset_at
                .and_then(format_reset_time)
                .unwrap_or_else(|| "--:--".to_string());

            parts.push(format!("{}{}{}% · {}{}{}", token_icon, sep, token.percentage, clock_icon, sep, reset_time));
        }

        // Call count (raw number only)
        if let Some(call_count) = stats.call_count {
            parts.push(format!("{}{}{}", chart_icon, sep, call_count));
        }

        // Weekly usage (new plan only, percentage)
        if let Some(weekly) = &stats.weekly_usage {
            parts.push(format!("{}{}{}%", calendar_icon, sep, weekly.percentage));
        }

        // MCP raw count
        if let Some(mcp) = &stats.mcp_usage {
            parts.push(format!("{}{}{}/{}", globe_icon, sep, mcp.used, mcp.limit));
        }

        // Token consumption (5-hour window)
        if let Some(tokens) = stats.tokens_used {
            parts.push(format!("{}{}{}", lightning_icon, sep, format_tokens(tokens)));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("{} {}", prefix, parts.join(" · "))
        }
    }

    fn get_color(_stats: &UsageStats) -> SegmentStyle {
        SegmentStyle {
            color: None,
            color_256: Some(109),
            bold: true,
        }
    }
}

impl Default for GlmUsageSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl Segment for GlmUsageSegment {
    fn id(&self) -> &str {
        "glm_usage"
    }

    fn collect(&self, input: &InputData, config: &Config) -> Option<SegmentData> {
        // No model filtering - show GLM usage if API is configured
        let stats = self.get_usage_stats(config)?;

        let model_name = input.model.as_ref().map(|m| m.id.as_str()).unwrap_or("GLM");
        let text = Self::format_stats(&stats, self.char_mode, model_name);
        if text.is_empty() {
            None
        } else {
            Some(SegmentData { text, style: Self::get_color(&stats) })
        }
    }
}
