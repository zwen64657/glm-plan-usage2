use super::minimax_types::*;
use crate::config::{get_api_key, get_base_url};
use anyhow::Result;
use std::time::Duration;
use ureq::{Agent, Request};

/// MiniMax API client
pub struct MiniMaxApiClient {
    agent: Agent,
    base_url: String,
    token: String,
    cookie: Option<String>,
}

impl MiniMaxApiClient {
    /// Create client from environment variables or Claude Code config.
    /// Requires `ANTHROPIC_BASE_URL` containing `minimaxi.com` or `minimax.io`.
    /// Credential lookup: env var → Claude Code settings.json.
    pub fn from_env() -> Result<Self> {
        let token = get_api_key()
            .map_err(|_| anyhow::anyhow!("Missing API key (env or Claude config)"))?;

        let base_url = get_base_url("");
        if base_url.is_empty() {
            return Err(anyhow::anyhow!("Missing base URL (env or Claude config)"));
        }

        // Verify it's a MiniMax URL
        if !base_url.contains("minimaxi.com") && !base_url.contains("minimax.io") {
            return Err(anyhow::anyhow!("Not a MiniMax base URL"));
        }

        // Extract domain for monitor API (same domain)
        let monitor_base = extract_domain(&base_url);

        // Read cookie: prefer MINIMAX_COOKIE, fall back to HERTZ_SESSION
        let cookie = std::env::var("USAGE_MINIMAX_COOKIE")
            .or_else(|_| {
                std::env::var("USAGE_HERTZ_SESSION")
                    .map(|v| format!("HERTZ-SESSION={}", v))
            })
            .ok();

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(5))
            .build();

        Ok(Self {
            agent,
            base_url: monitor_base,
            token,
            cookie,
        })
    }

    /// Fetch coding model usage stats
    pub fn fetch_usage_stats(&self) -> Result<MiniMaxUsageStats> {
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

    fn try_fetch(&self) -> Result<MiniMaxUsageStats> {
        let url = format!(
            "{}/v1/api/openplatform/coding_plan/remains",
            self.base_url
        );

        let response = self
            .authenticated_request(&url)
            .call()
            .map_err(|e| anyhow::anyhow!("HTTP error: {}", e))?;

        if response.status() != 200 {
            return Err(anyhow::anyhow!("HTTP status {}", response.status()));
        }

        let body: MiniMaxRemainsResponse = response
            .into_json()
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

        // Filter for coding model: model_name starts with "MiniMax-M"
        let coding_model = body
            .model_remains
            .iter()
            .find(|m| m.model_name.starts_with("MiniMax-M"));

        let model = match coding_model {
            Some(m) => m,
            None => return Err(anyhow::anyhow!("No coding model found in response")),
        };

        // API returns "remains" values, so usage_count = remaining, not used
        let interval_remaining = model.current_interval_usage_count;
        let interval_total = model.current_interval_total_count;
        let interval_used = interval_total - interval_remaining;
        let interval_pct = if interval_total > 0 {
            ((interval_used as f64 / interval_total as f64) * 100.0).round() as u8
        } else {
            0
        };

        // Weekly: only show if weekly_total > 0 (old plans have weekly_total_count=0)
        let (weekly_used, weekly_total, weekly_pct, weekly_reset) =
            if model.current_weekly_total_count > 0 {
                let weekly_remaining = model.current_weekly_usage_count;
                let w_total = model.current_weekly_total_count;
                let w_used = w_total - weekly_remaining;
                let pct = if w_total > 0 {
                    ((w_used as f64 / w_total as f64) * 100.0).round() as u8
                } else {
                    0
                };
                (
                    Some(w_used),
                    Some(w_total),
                    Some(pct),
                    model.weekly_end_time,
                )
            } else {
                (None, None, None, None)
            };

        Ok(MiniMaxUsageStats {
            interval_used,
            interval_total,
            interval_pct,
            reset_time: model.end_time,
            weekly_used,
            weekly_total,
            weekly_pct,
            weekly_reset_time: weekly_reset,
        })
    }

    fn authenticated_request(&self, url: &str) -> Request {
        let mut req = self
            .agent
            .get(url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Content-Type", "application/json");

        if let Some(ref cookie) = self.cookie {
            req = req.set("Cookie", cookie);
        }

        req
    }
}

/// Extract scheme + domain from a URL like "https://api.minimaxi.com/anthropic"
fn extract_domain(url: &str) -> String {
    // Find the scheme
    let scheme_end = url.find("://").unwrap_or(0);
    let scheme = if scheme_end > 0 { &url[..scheme_end + 3] } else { "" };

    // Find the end of domain (next /)
    let rest = &url[scheme_end + 3..];
    let domain_end = rest.find('/').unwrap_or(rest.len());
    let domain = &rest[..domain_end];

    format!("{}{}", scheme, domain)
}
