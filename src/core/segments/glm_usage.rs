use super::Segment;
use crate::api::{GlmApiClient, PlanLevel, UsageStats};
use crate::config::{Config, InputData};
use crate::core::segments::{SegmentData, SegmentStyle};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Call count limits for different plans (prompts * 15)
const OLD_PLAN_LIMITS: [(PlanLevel, i64); 3] = [
    (PlanLevel::Lite, 1800),   // 120 * 15
    (PlanLevel::Pro, 9000),    // 600 * 15
    (PlanLevel::Max, 36000),   // 2400 * 15
];

const NEW_PLAN_5H_LIMITS: [(PlanLevel, i64); 3] = [
    (PlanLevel::Lite, 1200),   // 80 * 15
    (PlanLevel::Pro, 6000),    // 400 * 15
    (PlanLevel::Max, 24000),   // 1600 * 15
];

const NEW_PLAN_WEEKLY_LIMITS: [(PlanLevel, i64); 3] = [
    (PlanLevel::Lite, 6000),    // 400 * 15
    (PlanLevel::Pro, 30000),    // 2000 * 15
    (PlanLevel::Max, 120000),   // 8000 * 15
];

fn get_limit(limits: &[(PlanLevel, i64); 3], level: PlanLevel) -> i64 {
    limits.iter().find(|(l, _)| *l == level).map(|(_, v)| *v).unwrap_or(9000)
}

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
}

struct CacheEntry {
    stats: UsageStats,
    timestamp: Instant,
}

impl GlmUsageSegment {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
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

    fn format_stats(stats: &UsageStats) -> String {
        let mut parts = Vec::new();
        let is_new_plan = stats.weekly_usage.is_some();
        let level = stats.level.unwrap_or(PlanLevel::Pro);

        // Token usage with reset time
        if let Some(token) = &stats.token_usage {
            let reset_time = token
                .reset_at
                .and_then(format_reset_time)
                .unwrap_or_else(|| "--:--".to_string());

            parts.push(format!("🪙 {}% (⏰ {})", token.percentage, reset_time));
        }

        // Call count with limit (5-hour)
        if let Some(call_count) = stats.call_count {
            let limit = if is_new_plan {
                get_limit(&NEW_PLAN_5H_LIMITS, level)
            } else {
                get_limit(&OLD_PLAN_LIMITS, level)
            };
            parts.push(format!("📊 {}/{}", call_count, limit));
        }

        // Weekly usage (new plan only)
        if let Some(weekly) = &stats.weekly_usage {
            let weekly_limit = get_limit(&NEW_PLAN_WEEKLY_LIMITS, level);
            let weekly_used = (weekly.percentage as i64) * weekly_limit / 100;
            parts.push(format!("📅 {}/{}", weekly_used, weekly_limit));
        }

        // MCP raw count
        if let Some(mcp) = &stats.mcp_usage {
            parts.push(format!("🌐 {}/{}", mcp.used, mcp.limit));
        }

        // Token consumption (5-hour window)
        if let Some(tokens) = stats.tokens_used {
            parts.push(format!("⚡ {}", format_tokens(tokens)));
        }

        if parts.is_empty() {
            String::new()
        } else {
            parts.join(" · ")
        }
    }

    fn get_color(stats: &UsageStats) -> SegmentStyle {
        // Get maximum usage percentage
        let max_pct = stats
            .token_usage
            .as_ref()
            .map(|u| u.percentage)
            .unwrap_or(0)
            .max(stats.mcp_usage.as_ref().map(|u| u.percentage).unwrap_or(0))
            .max(stats.weekly_usage.as_ref().map(|u| u.percentage).unwrap_or(0));

        let color_256 = match max_pct {
            0..=79 => Some(109),   // Green
            80..=94 => Some(226),  // Yellow
            95..=100 => Some(196), // Red
            _ => Some(109),
        };

        SegmentStyle {
            color: None,
            color_256,
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
        // Only show for GLM models
        if let Some(model) = &input.model {
            let model_id = model.id.to_lowercase();
            if !model_id.contains("glm") && !model_id.contains("chatglm") {
                return None;
            }
        }

        let stats = self.get_usage_stats(config)?;

        let text = Self::format_stats(&stats);

        if text.is_empty() {
            return None;
        }

        let style = Self::get_color(&stats);

        Some(SegmentData { text, style })
    }
}
