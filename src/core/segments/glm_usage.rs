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

/// Speed cache for TPS calculation
struct SpeedCache {
    total_input: i64,
    total_output: i64,
    timestamp: Instant,
    output_tps: f64,
    input_tps: f64,
}

/// GLM usage segment with caching
pub struct GlmUsageSegment {
    cache: Arc<Mutex<Option<CacheEntry>>>,
    speed_cache: Arc<Mutex<Option<SpeedCache>>>,
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
            speed_cache: Arc::new(Mutex::new(None)),
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

    fn format_stats(stats: &UsageStats, char_mode: CharMode, output_tps: f64, input_tps: f64) -> String {
        let mut parts = Vec::new();

        // Character mapping based on mode
        let (token_icon, clock_icon, chart_icon, calendar_icon, globe_icon, lightning_icon, rocket_icon, inbox_icon) = match char_mode {
            CharMode::Emoji => ("🔋", "⏰", "📊", "📅", "🌐", "⚡", "🚀", "📥"),
            CharMode::Ascii => ("$", "T", "#", "%", "M", "k", "v", "^"),
        };

        // Token usage with reset time
        if let Some(token) = &stats.token_usage {
            let reset_time = token
                .reset_at
                .and_then(format_reset_time)
                .unwrap_or_else(|| "--:--".to_string());

            parts.push(format!("{} {}% · {} {}", token_icon, token.percentage, clock_icon, reset_time));
        }

        // Output speed (model generation)
        if output_tps > 0.0 {
            parts.push(format!("↓{} {:.1}", rocket_icon, output_tps));
        }

        // Input speed (context + prompt + file reading)
        if input_tps > 0.0 {
            parts.push(format!("↑{} {:.1}", inbox_icon, input_tps));
        }

        // Call count (raw number only)
        if let Some(call_count) = stats.call_count {
            parts.push(format!("{} {}", chart_icon, call_count));
        }

        // Weekly usage (new plan only, percentage)
        if let Some(weekly) = &stats.weekly_usage {
            parts.push(format!("{} {}%", calendar_icon, weekly.percentage));
        }

        // MCP raw count
        if let Some(mcp) = &stats.mcp_usage {
            parts.push(format!("{} {}/{}", globe_icon, mcp.used, mcp.limit));
        }

        // Token consumption (5-hour window)
        if let Some(tokens) = stats.tokens_used {
            parts.push(format!("{} {}", lightning_icon, format_tokens(tokens)));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("GLM {}", parts.join(" · "))
        }
    }

    fn get_color(_stats: &UsageStats) -> SegmentStyle {
        SegmentStyle {
            color: None,
            color_256: Some(109),
            bold: true,
        }
    }

    /// Calculate input/output TPS using EMA smoothing
    fn calc_tps(&self, total_input: i64, total_output: i64) -> (f64, f64) {
        let now = Instant::now();
        let mut speed = self.speed_cache.lock().unwrap();

        let result = match speed.as_ref() {
            Some(prev) => {
                let delta_ms = prev.timestamp.elapsed().as_millis() as f64;
                if delta_ms > 100.0 {
                    let delta_sec = delta_ms / 1000.0;
                    let delta_out = total_output - prev.total_output;
                    let delta_in = total_input - prev.total_input;
                    let instant_out = if delta_out > 0 { delta_out as f64 / delta_sec } else { 0.0 };
                    let instant_in = if delta_in > 0 { delta_in as f64 / delta_sec } else { 0.0 };
                    // EMA: interval > 30s use instant value directly
                    let alpha = if delta_ms < 30000.0 { 0.5 } else { 1.0 };
                    let out_tps = alpha * instant_out + (1.0 - alpha) * prev.output_tps;
                    let in_tps = alpha * instant_in + (1.0 - alpha) * prev.input_tps;
                    (out_tps, in_tps)
                } else {
                    // Too short interval, use previous smoothed value
                    (prev.output_tps, prev.input_tps)
                }
            }
            None => (0.0, 0.0),
        };

        *speed = Some(SpeedCache {
            total_input,
            total_output,
            timestamp: now,
            output_tps: result.0,
            input_tps: result.1,
        });

        result
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
        // Only show for GLM models
        if let Some(model) = &input.model {
            let model_id = model.id.to_lowercase();
            if !model_id.contains("glm") && !model_id.contains("chatglm") {
                return None;
            }
        }

        // Calculate TPS from context_window
        let total_input = input.context_window.as_ref().and_then(|cw| cw.total_input_tokens).unwrap_or(0);
        let total_output = input.context_window.as_ref().and_then(|cw| cw.total_output_tokens).unwrap_or(0);
        let (output_tps, input_tps) = self.calc_tps(total_input, total_output);

        let stats = self.get_usage_stats(config);

        let (text, style) = match &stats {
            Some(s) => {
                (Self::format_stats(s, self.char_mode, output_tps, input_tps), Self::get_color(s))
            }
            None => {
                // Placeholder format when no data
                let (token_icon, clock_icon, chart_icon, _calendar_icon, globe_icon, lightning_icon, rocket_icon, inbox_icon) = match self.char_mode {
                    CharMode::Emoji => ("🔋", "⏰", "📊", "📅", "🌐", "⚡", "🚀", "📥"),
                    CharMode::Ascii => ("$", "T", "#", "%", "M", "k", "v", "^"),
                };
                let parts = vec![
                    format!("{} % · {} --:--", token_icon, clock_icon),
                    format!("↓{} --", rocket_icon),
                    format!("↑{} --", inbox_icon),
                    format!("{} 0", chart_icon),
                    format!("{} /", globe_icon),
                    format!("{}", lightning_icon),
                ];
                let text = format!("GLM {}", parts.join(" · "));
                let style = SegmentStyle { color_256: Some(109), bold: true, color: None };
                (text, style)
            }
        };

        if text.is_empty() {
            None
        } else {
            Some(SegmentData { text, style })
        }
    }
}
