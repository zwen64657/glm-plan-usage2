use serde::Deserialize;
use std::fmt;
use thiserror::Error;

/// Platform detection from base URL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Zai,
    Zhipu,
}

impl Platform {
    pub fn detect_from_url(base_url: &str) -> Option<Self> {
        if base_url.contains("api.z.ai") {
            Some(Platform::Zai)
        } else if base_url.contains("bigmodel.cn") || base_url.contains("zhipu") {
            Some(Platform::Zhipu)
        } else {
            None
        }
    }
}

/// API error types
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Credential not found: {0}")]
    CredentialNotFound(String),

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("API returned error: {0}")]
    ApiResponse(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Platform detection failed")]
    PlatformDetectionFailed,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QuotaLimitItem {
    #[serde(rename = "type", default)]
    pub quota_type: String,
    #[serde(default)]
    pub unit: i32, // 3=5h, 5=MCP, 6=weekly
    #[serde(default)]
    pub usage: i64,
    #[serde(rename = "currentValue", default)]
    pub current_value: i64,
    pub percentage: i32,
    #[serde(rename = "nextResetTime", default)]
    pub next_reset_time: Option<i64>, // Millisecond timestamp
}

/// Model usage response
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ModelUsageResponse {
    pub code: Option<i32>,
    pub msg: Option<String>,
    pub data: Option<ModelUsageData>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ModelUsageData {
    pub total: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub prompt_tokens: Option<i64>,
}

/// Model usage API response (for 5-hour call count)
#[derive(Debug, Deserialize)]
pub struct ModelUsageApiResponse {
    #[allow(dead_code)]
    pub code: Option<i32>,
    #[allow(dead_code)]
    pub msg: Option<String>,
    pub data: Option<ModelUsageApiData>,
}

#[derive(Debug, Deserialize)]
pub struct ModelUsageApiData {
    #[serde(rename = "totalUsage", default)]
    pub total_usage: Option<ModelTotalUsage>,
}

impl ModelUsageApiData {
    pub fn get_total_usage(&self) -> Option<&ModelTotalUsage> {
        self.total_usage.as_ref()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelTotalUsage {
    #[serde(rename = "totalModelCallCount")]
    pub total_model_call_count: i64,
    #[serde(rename = "totalTokensUsage", default)]
    pub total_tokens_usage: i64,
}

/// Tool usage response
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ToolUsageResponse {
    pub code: Option<i32>,
    pub msg: Option<String>,
    pub data: Option<ToolUsageData>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ToolUsageData {
    pub total: Option<i64>,
}

/// Plan level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanLevel {
    Lite,
    Pro,
    Max,
}

impl PlanLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "lite" => Some(PlanLevel::Lite),
            "pro" => Some(PlanLevel::Pro),
            "max" => Some(PlanLevel::Max),
            _ => None,
        }
    }
}

/// Combined usage statistics
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub token_usage: Option<QuotaUsage>,
    pub mcp_usage: Option<QuotaUsage>,
    pub weekly_usage: Option<QuotaUsage>,
    pub call_count: Option<i64>,
    pub tokens_used: Option<i64>,
    #[allow(dead_code)]
    pub level: Option<PlanLevel>,
}

/// Individual quota usage
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QuotaUsage {
    pub used: i64,
    pub limit: i64,
    pub percentage: u8,
    pub time_window: String,
    pub reset_at: Option<i64>, // Second-level timestamp (converted from ms)
}

impl fmt::Display for QuotaUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.percentage)
    }
}
