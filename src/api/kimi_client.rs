use super::kimi_types::*;
use crate::config::{get_api_key, get_base_url};
use anyhow::Result;
use std::time::Duration;
use ureq::{Agent, Request};

/// Kimi API client
pub struct KimiApiClient {
    agent: Agent,
    base_url: String,
    token: String,
}

impl KimiApiClient {
    /// Create client from environment variables or Claude Code config.
    /// Requires `ANTHROPIC_BASE_URL` containing `kimi.com`.
    /// Credential lookup: env var → Claude Code settings.json.
    pub fn from_env() -> Result<Self> {
        let token = get_api_key()
            .map_err(|_| anyhow::anyhow!("Missing API key (env or Claude config)"))?;

        let base_url = get_base_url("");
        if base_url.is_empty() {
            return Err(anyhow::anyhow!("Missing base URL (env or Claude config)"));
        }

        // Verify it's a Kimi URL
        if !base_url.contains("kimi.com") {
            return Err(anyhow::anyhow!("Not a Kimi base URL"));
        }

        // Extract domain for monitor API (same domain)
        let monitor_base = extract_domain(&base_url);

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(5))
            .build();

        Ok(Self {
            agent,
            base_url: monitor_base,
            token,
        })
    }

    /// Fetch usage stats
    pub fn fetch_usage_stats(&self) -> Result<KimiUsageStats> {
        let mut last_error = None;

        for attempt in 0..=2 {
            match self.try_fetch() {
                Ok(stats) => return Ok(stats),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < 2 {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    fn try_fetch(&self) -> Result<KimiUsageStats> {
        let url = format!("{}/coding/v1/usages", self.base_url);

        let response = self
            .authenticated_request(&url)
            .call()
            .map_err(|e| anyhow::anyhow!("HTTP error: {}", e))?;

        if response.status() != 200 {
            return Err(anyhow::anyhow!("HTTP status {}", response.status()));
        }

        let body: KimiUsagesResponse = response
            .into_json()
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

        // Find 5-hour window: duration=300, time_unit=TIME_UNIT_MINUTE
        let five_hour = body
            .limits
            .iter()
            .find(|l| l.window.duration == 300 && l.window.time_unit == "TIME_UNIT_MINUTE");

        // Find weekly window: duration=10080
        let weekly = body
            .limits
            .iter()
            .find(|l| l.window.duration == 10080);

        let (five_hour_pct, five_hour_reset) = match five_hour {
            Some(l) => {
                let pct = if l.detail.limit > 0 {
                    let used = l.detail.limit - l.detail.remaining;
                    ((used as f64 / l.detail.limit as f64) * 100.0).round() as u8
                } else {
                    0
                };
                (pct, l.detail.reset_time.clone())
            }
            None => return Err(anyhow::anyhow!("No 5-hour window found in Kimi response")),
        };

        let (weekly_pct, weekly_reset) = match weekly {
            Some(l) => {
                let pct = if l.detail.limit > 0 {
                    let used = l.detail.limit - l.detail.remaining;
                    ((used as f64 / l.detail.limit as f64) * 100.0).round() as u8
                } else {
                    0
                };
                (pct, l.detail.reset_time.clone())
            }
            None => return Err(anyhow::anyhow!("No weekly window found in Kimi response")),
        };

        Ok(KimiUsageStats {
            five_hour_pct,
            five_hour_reset,
            weekly_pct,
            weekly_reset,
        })
    }

    fn authenticated_request(&self, url: &str) -> Request {
        self.agent
            .get(url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Content-Type", "application/json")
    }
}

/// Extract scheme + domain from a URL like "https://api.kimi.com/coding/"
fn extract_domain(url: &str) -> String {
    let scheme_end = url.find("://").unwrap_or(0);
    let scheme = if scheme_end > 0 { &url[..scheme_end + 3] } else { "" };

    let rest = &url[scheme_end + 3..];
    let domain_end = rest.find('/').unwrap_or(rest.len());
    let domain = &rest[..domain_end];

    format!("{}{}", scheme, domain)
}
