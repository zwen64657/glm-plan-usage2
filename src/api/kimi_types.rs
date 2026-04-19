use serde::Deserialize;

/// Kimi usages response
#[derive(Debug, Deserialize)]
pub struct KimiUsagesResponse {
    #[allow(dead_code)]
    pub usage: Option<KimiUsage>,
    pub limits: Vec<KimiLimit>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct KimiUsage {
    pub remaining: Option<i64>,
    pub limit: Option<i64>,
    #[serde(default, rename = "resetTime")]
    pub reset_time: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KimiLimit {
    pub window: KimiWindow,
    pub detail: KimiDetail,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KimiWindow {
    pub duration: i64,
    #[serde(default, rename = "timeUnit")]
    pub time_unit: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KimiDetail {
    pub limit: i64,
    pub remaining: i64,
    #[serde(default, rename = "resetTime")]
    pub reset_time: Option<String>,
}

/// Processed Kimi usage stats
#[derive(Debug, Clone)]
pub struct KimiUsageStats {
    pub five_hour_pct: u8,
    pub five_hour_reset: Option<String>, // ISO 8601 string
    pub weekly_pct: u8,
    #[allow(dead_code)]
    pub weekly_reset: Option<String>,    // ISO 8601 string
}
