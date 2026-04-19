use serde::Deserialize;

/// MiniMax coding plan remains response
#[derive(Debug, Deserialize)]
pub struct MiniMaxRemainsResponse {
    #[allow(dead_code)]
    pub base_resp: Option<MiniMaxBaseResp>,
    pub model_remains: Vec<MiniMaxModelRemains>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MiniMaxBaseResp {
    pub status_code: i32,
    pub status_msg: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MiniMaxModelRemains {
    pub model_name: String,
    #[serde(default)]
    pub current_interval_total_count: i64,
    #[serde(default)]
    pub current_interval_usage_count: i64,
    #[serde(default)]
    pub end_time: Option<i64>,
    #[serde(default)]
    pub current_weekly_total_count: i64,
    #[serde(default)]
    pub current_weekly_usage_count: i64,
    #[serde(default)]
    pub weekly_end_time: Option<i64>,
}

/// Processed MiniMax usage stats
#[derive(Debug, Clone)]
pub struct MiniMaxUsageStats {
    pub interval_used: i64,
    pub interval_total: i64,
    pub interval_pct: u8,
    pub reset_time: Option<i64>,
    #[allow(dead_code)]
    pub weekly_used: Option<i64>,
    #[allow(dead_code)]
    pub weekly_total: Option<i64>,
    pub weekly_pct: Option<u8>,
    #[allow(dead_code)]
    pub weekly_reset_time: Option<i64>,
}
