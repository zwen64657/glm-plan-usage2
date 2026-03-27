use super::types::*;
use anyhow::Result;
use std::time::Duration;
use ureq::{Agent, Request};

/// GLM API client
pub struct GlmApiClient {
    agent: Agent,
    base_url: String,
    token: String,
    platform: Platform,
}

/// Quota limit response with level field (level is inside data, not at root)
#[derive(Debug, serde::Deserialize)]
struct QuotaLimitResponseWithLevel {
    #[allow(dead_code)]
    code: i32,
    msg: String,
    data: QuotaLimitDataWithLevel,
    success: bool,
}

#[derive(Debug, serde::Deserialize)]
struct QuotaLimitDataWithLevel {
    #[serde(default)]
    level: Option<String>,
    limits: Vec<QuotaLimitItem>,
}

impl GlmApiClient {
    /// Create client from environment variables
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("GLM_AUTH_TOKEN")
            .or_else(|_| std::env::var("ANTHROPIC_AUTH_TOKEN"))
            .map_err(|_| ApiError::MissingEnvVar("GLM_AUTH_TOKEN or ANTHROPIC_AUTH_TOKEN".to_string()))?;

        let base_url = std::env::var("GLM_BASE_URL")
            .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
            .unwrap_or_else(|_| "https://open.bigmodel.cn/api/anthropic".to_string());

        let platform =
            Platform::detect_from_url(&base_url).ok_or(ApiError::PlatformDetectionFailed)?;

        // Fix base URL for monitor API (matching JS: always apply replacement)
        let base_url = base_url
            .replace("/api/anthropic", "/api")
            .replace("/anthropic", "");

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(5))
            .build();

        Ok(Self {
            agent,
            base_url,
            token,
            platform,
        })
    }

    /// Fetch complete usage statistics (simplified - all data from quota/limit endpoint)
    pub fn fetch_usage_stats(&self) -> Result<UsageStats> {
        // Retry logic
        let mut last_error = None;

        for attempt in 0..=2 {
            match self.try_fetch_usage_stats() {
                Ok(stats) => return Ok(stats),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < 2 {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    fn try_fetch_usage_stats(&self) -> Result<UsageStats> {
        // Fetch quota limits (contains all the data we need)
        let url = format!("{}/monitor/usage/quota/limit", self.base_url);

        let response = self
            .authenticated_request(&url)
            .call()
            .map_err(|e| ApiError::HttpError(e.to_string()))?;

        if response.status() != 200 {
            return Err(ApiError::ApiResponse(format!(
                "Status {}: {}",
                response.status(),
                response.status_text()
            ))
            .into());
        }

        let quota_response: QuotaLimitResponseWithLevel = response
            .into_json()
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        if !quota_response.success {
            return Err(ApiError::ApiResponse(quota_response.msg).into());
        }

        // Extract level from data.level (matching JS: quota.data?.level)
        let level = quota_response
            .data
            .level
            .as_ref()
            .and_then(|l| PlanLevel::from_str(l));

        // Extract token usage (TOKENS_LIMIT with unit=3, matching JS: l.type === "TOKENS_LIMIT" && l.unit === 3)
        let token_usage = quota_response
            .data
            .limits
            .iter()
            .find(|item| item.quota_type == "TOKENS_LIMIT" && item.unit == 3)
            .map(|item| QuotaUsage {
                used: item.current_value,
                limit: item.usage,
                percentage: item.percentage.clamp(0, 100) as u8,
                time_window: "5h".to_string(),
                reset_at: item.next_reset_time.map(|ms| ms / 1000),
            });

        // Get reset time for call count query (sync with quota window)
        let reset_time_ms = token_usage
            .as_ref()
            .and_then(|t| t.reset_at)
            .map(|s| s * 1000);

        // Extract tool usage (TIME_LIMIT)
        let mcp_usage = quota_response
            .data
            .limits
            .iter()
            .find(|item| item.quota_type == "TIME_LIMIT")
            .map(|item| QuotaUsage {
                used: item.current_value,
                limit: item.usage,
                percentage: item.percentage.clamp(0, 100) as u8,
                time_window: "30d".to_string(),
                reset_at: item.next_reset_time.map(|ms| ms / 1000),
            });

        // Extract weekly usage (TOKENS_LIMIT with unit=6, matching JS: l.type === "TOKENS_LIMIT" && l.unit === 6)
        let weekly_usage = quota_response
            .data
            .limits
            .iter()
            .find(|item| item.quota_type == "TOKENS_LIMIT" && item.unit == 6)
            .map(|item| QuotaUsage {
                used: item.current_value,
                limit: item.usage,
                percentage: item.percentage.clamp(0, 100) as u8,
                time_window: "7d".to_string(),
                reset_at: item.next_reset_time.map(|ms| ms / 1000),
            });

        // Fetch model usage for call count and tokens (synced with quota window)
        let model_usage = self.fetch_model_usage(reset_time_ms).ok().flatten();
        let (call_count, tokens_used) = match model_usage {
            Some((cc, tk)) => (Some(cc), Some(tk)),
            None => (None, None),
        };

        Ok(UsageStats {
            token_usage,
            mcp_usage,
            weekly_usage,
            call_count,
            tokens_used,
            level,
        })
    }

    /// Fetch 5-hour call count and token usage from model-usage endpoint
    /// Uses reset_time to sync with quota window instead of simple now-5h
    fn fetch_model_usage(&self, reset_time_ms: Option<i64>) -> Result<Option<(i64, i64)>> {
        // Without nextResetTime, a rolling window would include pre-reset stale data
        let reset_ms = match reset_time_ms {
            Some(ms) => ms,
            None => return Ok(None),
        };

        let url = format!("{}/monitor/usage/model-usage", self.base_url);

        // Use platform-appropriate timezone for API queries
        // Zhipu server expects Beijing time (UTC+8), ZAI server expects UTC
        let tz = match self.platform {
            Platform::Zhipu => chrono::FixedOffset::east_opt(8 * 3600).unwrap(),
            Platform::Zai => chrono::FixedOffset::east_opt(0).unwrap(),
        };

        // Use reset time to calculate window: from (reset - 5h) to reset
        let reset_time = chrono::DateTime::from_timestamp_millis(reset_ms)
            .unwrap_or_else(|| chrono::Utc::now())
            .with_timezone(&tz);
        let start_time = reset_time - chrono::Duration::hours(5);

        let start_str = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let end_str = reset_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let url_with_params = format!(
            "{}?startTime={}&endTime={}",
            url,
            urlencoding::encode(&start_str),
            urlencoding::encode(&end_str)
        );

        let response = self
            .authenticated_request(&url_with_params)
            .call()
            .map_err(|e| ApiError::HttpError(e.to_string()))?;

        if response.status() != 200 {
            return Ok(None);
        }

        let usage_response: ModelUsageApiResponse = response
            .into_json()
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        let call_count = usage_response
            .data
            .as_ref()
            .and_then(|d| d.get_total_usage())
            .map(|u| (u.total_model_call_count, u.total_tokens_usage));

        Ok(call_count)
    }

    fn authenticated_request(&self, url: &str) -> Request {
        self.agent
            .get(url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Content-Type", "application/json")
    }
}
