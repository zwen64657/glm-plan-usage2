# GLM Plan Usage 完整修改报告

> 本报告合并了四份修改报告：环境变量与时间显示修改、周限量支持、模型判断功能、调用次数显示（含时间窗口同步修复）

---

## 目录

1. [环境变量与时间显示修改](#1-环境变量与时间显示修改)
2. [周限量支持](#2-周限量支持)
3. [模型判断功能](#3-模型判断功能)
4. [调用次数显示](#4-调用次数显示)

---

## 1. 环境变量与时间显示修改

### 修改时间
2026-03-06 16:06:43

### 修改内容

#### 1.1 环境变量 (src/api/client.rs)

| 修改前 | 修改后 |
|--------|--------|
| ANTHROPIC_AUTH_TOKEN | GLM_AUTH_TOKEN |
| ANTHROPIC_BASE_URL | GLM_BASE_URL |

#### 1.2 图标和时间显示 (src/core/segments/glm_usage.rs)

| 项目 | 修改前 | 修改后 |
|------|--------|--------|
| 图标 | ⌛️ (沙漏) | ⏰ (闹钟) |
| 时间格式 | 倒计时 (如 1:44) | 绝对时间 (如 18:17) |
| 函数名 | format_countdown() | format_reset_time() |

#### 1.3 代码变更

**src/api/client.rs (第17-20行)**
```
修改前:
let token = std::env::var("ANTHROPIC_AUTH_TOKEN")
let base_url = std::env::var("ANTHROPIC_BASE_URL")

修改后:
let token = std::env::var("GLM_AUTH_TOKEN")
let base_url = std::env::var("GLM_BASE_URL")
```

**src/core/segments/glm_usage.rs**
```
修改前 (倒计时):
fn format_countdown(reset_at: i64) -> Option<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
    let remaining = reset_at.saturating_sub(now);
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    Some(format!("{}:{:02}", hours, minutes))
}

修改后 (绝对时间):
fn format_reset_time(reset_at: i64) -> Option<String> {
    use chrono::{DateTime, Local, TimeZone, Timelike};
    let dt: DateTime<Local> = Local.timestamp_opt(reset_at, 0).single()?;
    Some(format!("{}:{:02}", dt.hour(), dt.minute()))
}
```

---

## 2. 周限量支持

### 修改时间
2026-03-06

### 修改目的
添加周限量（Weekly Usage）显示支持，自动适配不同套餐用户。

### 最终效果

| 套餐类型 | 状态栏显示 |
|---------|-----------|
| 无周限量 | `🪙 4% (⏰ 18:17) · 🌐 20/100` |
| 有周限量 | `🪙 4% (⏰ 18:17) · 📅 25% · 🌐 20/100` |

### 代码变更

#### 2.1 src/api/types.rs

**添加 unit 字段**（用于识别配额类型）：
```rust
#[derive(Debug, Deserialize, Clone)]
pub struct QuotaLimitItem {
    #[serde(rename = "type", default)]
    pub quota_type: String,
    #[serde(default)]
    pub unit: i32, // 3=5h, 5=MCP, 6=weekly
    // ... 其他字段
}
```

**添加周限量到 UsageStats**：
```rust
pub struct UsageStats {
    pub token_usage: Option<QuotaUsage>,
    pub mcp_usage: Option<QuotaUsage>,
    pub weekly_usage: Option<QuotaUsage>,  // 新增
}
```

#### 2.2 src/api/client.rs

**解析周限量数据**（unit=6）：
```rust
// Extract weekly usage (unit=6)
let weekly_usage = quota_response
    .data
    .limits
    .iter()
    .find(|item| item.unit == 6)
    .map(|item| QuotaUsage {
        used: item.current_value,
        limit: item.usage,
        percentage: item.percentage.clamp(0, 100) as u8,
        time_window: "7d".to_string(),
        reset_at: item.next_reset_time.map(|ms| ms / 1000),
    });

Ok(UsageStats {
    token_usage,
    mcp_usage,
    weekly_usage,  // 新增
})
```

#### 2.3 src/core/segments/glm_usage.rs

**显示周限量**：
```rust
// Weekly usage
if let Some(weekly) = &stats.weekly_usage {
    parts.push(format!("📅 {}%", weekly.percentage));
}
```

**颜色计算包含周限量**：
```rust
let max_pct = stats
    .token_usage
    .as_ref()
    .map(|u| u.percentage)
    .unwrap_or(0)
    .max(stats.mcp_usage.as_ref().map(|u| u.percentage).unwrap_or(0))
    .max(stats.weekly_usage.as_ref().map(|u| u.percentage).unwrap_or(0));  // 新增
```

### API 配额类型说明

| unit | 类型 | 图标 | 说明 |
|------|------|------|------|
| 3 | 5小时配额 | 🪙 ⏰ | 5小时滚动窗口 |
| 5 | MCP配额 | 🌐 | 30天工具调用限制 |
| 6 | 周限量 | 📅 | 7天滚动窗口（部分套餐有） |

### 自动适配逻辑

- 程序从 `/api/monitor/usage/quota/limit` 获取配额数据
- 根据返回的 `unit` 字段动态判断用户有哪些配额
- 只有 API 返回对应 unit 的数据时才显示
- 无需硬编码用户套餐类型

---

## 3. 模型判断功能

### 修改时间
2026-03-08 20:32

### 修改目的
使用非GLM模型（如 deepseek）时，状态栏不显示GLM用量信息。

### 修改逻辑
通过检测当前使用的模型名称，判断是否显示用量：
- 模型名包含 `glm` 或 `chatglm` → 显示用量
- 其他模型 → 隐藏用量

### 最终效果

| 模型 | 状态栏显示 |
|------|-----------|
| glm-4-plus | `🪙 4% (⏰ 18:17) · 🌐 20/4000` |
| chatglm-xxx | `🪙 4% (⏰ 18:17) · 🌐 20/4000` |
| deepseek-xxx | (不显示) |
| claude-xxx | (不显示) |

### 代码变更

#### 3.1 src/core/segments/glm_usage.rs

**修改 collect 函数**（添加模型判断）：

```rust
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
```

**修改前**：
```rust
fn collect(&self, _input: &InputData, config: &Config) -> Option<SegmentData> {
    let stats = self.get_usage_stats(config)?;
    // ...
}
```

**修改后**：
```rust
fn collect(&self, input: &InputData, config: &Config) -> Option<SegmentData> {
    // Only show for GLM models
    if let Some(model) = &input.model {
        let model_id = model.id.to_lowercase();
        if !model_id.contains("glm") && !model_id.contains("chatglm") {
            return None;
        }
    }

    let stats = self.get_usage_stats(config)?;
    // ...
}
```

### 相关数据结构

InputData 来自 Claude Code 传入的 stdin JSON：

```json
{
  "model": {
    "id": "glm-4-plus",
    "display_name": "GLM-4 Plus"
  }
}
```

---

## 4. 调用次数显示

### 修改时间
2026-03-25 20:54（初始）· 2026-03-26 16:00（时间窗口同步修复）

### 修改目的
在状态栏显示5小时调用次数，格式为 `📊 93/9000`（当前调用次数/套餐上限）。

### 配额表（×15 换算：1 prompt = 15 次调用）

#### 老套餐（无周限量）

| Level | Prompts | 调用上限 |
|-------|---------|---------|
| Lite | 120 | 1,800 |
| Pro | 600 | 9,000 |
| Max | 2400 | 36,000 |

#### 新套餐（有周限量）

| Level | 5小时 Prompts | 5小时调用 | 周限额 Prompts | 周限额调用 |
|-------|--------------|----------|---------------|-----------|
| Lite | 80 | 1,200 | 400 | 6,000 |
| Pro | 400 | 6,000 | 2000 | 30,000 |
| Max | 1600 | 24,000 | 8000 | 120,000 |

### 判断逻辑
- **有 `weekly_usage` 数据** → 新套餐 → 显示 5小时 + 周限额
- **无 `weekly_usage` 数据** → 老套餐 → 只显示 5小时

### 最终效果

**老套餐（Pro）：**
```
🪙 5% (⏰ 23:00) · 📊 93/9000 · 🌐 0/1000
```

**新套餐（Pro）：**
```
🪙 5% (⏰ 23:00) · 📊 93/6000 · 📅 300/30000 · 🌐 0/1000
```

### 代码变更

#### 4.1 Cargo.toml

**添加 urlencoding 依赖**：
```toml
urlencoding = "2.1"
```

#### 4.2 src/api/mod.rs

**导出 PlanLevel**：
```rust
pub use types::{PlanLevel, UsageStats};
```

#### 4.3 src/api/types.rs

**添加 ModelUsageApiResponse**（修复字段命名）：
```rust
#[derive(Debug, Deserialize)]
pub struct ModelUsageApiResponse {
    pub code: Option<i32>,
    pub msg: Option<String>,
    pub data: Option<ModelUsageApiData>,
    #[serde(default)]
    pub success: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ModelUsageApiData {
    #[serde(default)]
    pub totalUsage: Option<ModelTotalUsage>,
    #[serde(default)]
    pub total_usage: Option<ModelTotalUsage>,
}

impl ModelUsageApiData {
    pub fn get_total_usage(&self) -> Option<&ModelTotalUsage> {
        self.totalUsage.as_ref().or(self.total_usage.as_ref())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelTotalUsage {
    #[serde(rename = "totalModelCallCount")]
    pub total_model_call_count: i64,
    #[serde(rename = "totalTokensUsage", default)]
    pub total_tokens_usage: i64,
}
```

**添加 PlanLevel 枚举**：
```rust
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
```

**更新 UsageStats**：
```rust
pub struct UsageStats {
    pub token_usage: Option<QuotaUsage>,
    pub mcp_usage: Option<QuotaUsage>,
    pub weekly_usage: Option<QuotaUsage>,
    pub call_count: Option<i64>,      // 新增
    pub level: Option<PlanLevel>,     // 新增
}
```

#### 4.4 src/api/client.rs

**添加 QuotaLimitResponseWithLevel**：
```rust
#[derive(Debug, serde::Deserialize)]
struct QuotaLimitResponseWithLevel {
    code: i32,
    msg: String,
    data: QuotaLimitData,
    success: bool,
    level: Option<String>,
}
```

**解析 level**：
```rust
let level = quota_response
    .level
    .as_ref()
    .and_then(|l| PlanLevel::from_str(l));
```

**添加 fetch_call_count 方法**（2026-03-26 更新：时间窗口同步）：
```rust
fn fetch_call_count(&self, reset_time_ms: Option<i64>) -> Result<Option<i64>> {
    let url = format!("{}/monitor/usage/model-usage", self.base_url);

    // 使用 nextResetTime 计算时间窗口，与配额窗口同步
    let now = chrono::Utc::now();
    let (start_time, end_time) = if let Some(reset_ms) = reset_time_ms {
        // 从 (reset - 5h) 到 reset，与配额窗口完全同步
        let reset_time = chrono::DateTime::from_timestamp_millis(reset_ms)
            .unwrap_or(now);
        let start = reset_time - chrono::Duration::hours(5);
        (start, reset_time)
    } else {
        // 无 reset time 时回退到简单 5h 窗口
        let start = now - chrono::Duration::hours(5);
        (start, now)
    };

    let start_str = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_str = end_time.format("%Y-%m-%d %H:%M:%S").to_string();

    let url_with_params = format!(
        "{}?startTime={}&endTime={}",
        url,
        urlencoding::encode(&start_str),
        urlencoding::encode(&end_str)
    );

    let response = self.authenticated_request(&url_with_params)
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
        .map(|u| u.total_model_call_count);

    Ok(call_count)
}
```

**调用处更新**（传入 reset_time）：
```rust
// 获取 reset time 用于调用次数查询（与配额窗口同步）
let reset_time_ms = token_usage
    .as_ref()
    .and_then(|t| t.reset_at)
    .map(|s| s * 1000);

// 调用次数查询使用配额窗口
let call_count = self.fetch_call_count(reset_time_ms).ok().flatten();
```

#### 4.5 src/core/segments/glm_usage.rs

**添加配额常量**：
```rust
const OLD_PLAN_LIMITS: [(PlanLevel, i64); 3] = [
    (PlanLevel::Lite, 1800),
    (PlanLevel::Pro, 9000),
    (PlanLevel::Max, 36000),
];

const NEW_PLAN_5H_LIMITS: [(PlanLevel, i64); 3] = [
    (PlanLevel::Lite, 1200),
    (PlanLevel::Pro, 6000),
    (PlanLevel::Max, 24000),
];

const NEW_PLAN_WEEKLY_LIMITS: [(PlanLevel, i64); 3] = [
    (PlanLevel::Lite, 6000),
    (PlanLevel::Pro, 30000),
    (PlanLevel::Max, 120000),
];

fn get_limit(limits: &[(PlanLevel, i64); 3], level: PlanLevel) -> i64 {
    limits.iter().find(|(l, _)| *l == level).map(|(_, v)| *v).unwrap_or(9000)
}
```

**更新 format_stats**：
```rust
fn format_stats(stats: &UsageStats) -> String {
    let mut parts = Vec::new();
    let is_new_plan = stats.weekly_usage.is_some();
    let level = stats.level.unwrap_or(PlanLevel::Pro);

    // Token usage with reset time
    if let Some(token) = &stats.token_usage {
        let reset_time = token.reset_at.and_then(format_reset_time)
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

    parts.join(" · ")
}
```

### 时间窗口同步修复（2026-03-26）

#### 问题描述
调用次数 `📊 108/9000` 和配额使用率 `🪙 1%` 的时间窗口不同步：
- 配额使用率来自 `/quota/limit` API，使用 API 内部的 5 小时滚动窗口（基于 `nextResetTime`）
- 调用次数来自 `/model-usage` API，之前用 `now - 5h` 到 `now` 查询

导致第二天打开电脑时，配额已重置显示 1%，但调用次数仍显示昨天的累计值（如 108）。

#### 解决方案
修改 `fetch_call_count` 使用 `nextResetTime` 计算查询时间窗口：
- **之前**：`now - 5h` 到 `now`（可能与配额窗口不同步）
- **现在**：`reset_time - 5h` 到 `reset_time`（与配额窗口完全同步）

调用次数现在会在配额重置时同步归零。

### 时区修复（2026-03-27）

#### 问题描述
2026-03-27 当天实际有 72 次模型调用、约 338 万 Token 消耗，但状态栏 `📊 0/9000` 始终显示调用次数为 0。

#### 根本原因
`fetch_call_count` 方法使用 `chrono::Utc::now()` 计算时间窗口，格式化为 `"2026-03-27 06:00:00"` 这样的字符串发送给智谱 `/monitor/usage/model-usage` API。但智谱服务器将 `startTime` / `endTime` 参数解释为**北京时间（UTC+8）**，而代码发送的是 UTC 时间。

这导致查询窗口偏移 8 小时，实际调用时间完全不在查询范围内，API 返回 0。

**示例：**
- 当前北京时间 15:00 = UTC 07:00
- `nextResetTime` 对应北京时间 19:00 = UTC 11:00
- 代码计算窗口：UTC 06:00 ~ UTC 11:00
- 格式化发送：`"2026-03-27 06:00:00"` ~ `"2026-03-27 11:00:00"`
- API 解释为北京时间：BJT 06:00 ~ BJT 11:00 = UTC 22:00(26日) ~ UTC 03:00(27日)
- 实际调用时间 BJT 10:00~15:00 = UTC 02:00~07:00 → **不在窗口内，返回 0**

#### 解决方案
修改 `fetch_call_count` 根据**平台自动选择时区**进行时间计算和格式化：

- **智谱（bigmodel.cn）**：使用北京时间（UTC+8），因为智谱服务器期望 CST 时间字符串
- **ZAI（api.z.ai）**：使用 UTC（UTC+0），因为国外服务器期望 UTC 时间字符串

```rust
let tz = match self.platform {
    Platform::Zhipu => chrono::FixedOffset::east_opt(8 * 3600).unwrap(),
    Platform::Zai => chrono::FixedOffset::east_opt(0).unwrap(),
};
let now = chrono::Utc::now().with_timezone(&tz);

let (start_time, end_time) = if let Some(reset_ms) = reset_time_ms {
    let reset_time = chrono::DateTime::from_timestamp_millis(reset_ms)
        .unwrap_or_else(|| chrono::Utc::now())
        .with_timezone(&tz);
    let start = reset_time - chrono::Duration::hours(5);
    (start, reset_time)
} else {
    let start = now - chrono::Duration::hours(5);
    (start, now)
};
```

#### 部署说明
由于 `glm-plan-usage.exe` 在 Droid 运行期间被锁定，需要：
1. 退出 Droid
2. 执行替换命令：
   ```
   del C:\Users\18773\.claude\glm-plan-usage\glm-plan-usage.exe
   ren C:\Users\18773\.claude\glm-plan-usage\glm-plan-usage.exe.new glm-plan-usage.exe
   ```
3. 重新启动 Droid

### Token 消耗显示（2026-03-27）

#### 问题描述
用户希望在状态栏中显示 5 小时窗口内的 Token 消耗量，放在 MCP 配额（🌐）后面。

#### 实现方案
从已有的 `/monitor/usage/model-usage` API 中提取 `totalTokensUsage` 字段（该字段之前已有反序列化但未使用）。

**修改内容：**

1. **`src/api/types.rs`** — `UsageStats` 新增 `tokens_used: Option<i64>` 字段

2. **`src/api/client.rs`** — `fetch_call_count` 重命名为 `fetch_model_usage`，返回值从 `Option<i64>` 改为 `Option<(i64, i64)>`（调用次数 + Token 消耗），调用处解构为 `call_count` 和 `tokens_used`

3. **`src/core/segments/glm_usage.rs`** — `format_stats` 中在 `🌐` 后追加 Token 消耗显示，使用已有的 `format_tokens` 函数智能格式化

**显示格式：**
```
🪙 4% (⏰ 07:00) · 📊 72/9000 · 🌐 0/1000 · ⚡ 3.38M
```

- `< 1万`：显示原始数字，如 `⚡ 8542`
- `1万~100万`：显示 K 单位，如 `⚡ 310.9K`
- `>= 100万`：显示 M 单位，如 `⚡ 3.38M`
- 即使消耗 8 亿（800M）也能正常显示

### 缓存 TTL 调整（2026-03-27）

#### 修改内容
将缓存过期时间从 300 秒（5 分钟）调整为 120 秒（2 分钟），使 `📊` 调用次数和 `⚡` Token 消耗的刷新更及时。

**影响：**
- API 请求频率从 12 次/小时增加到 30 次/小时，对性能无显著影响
- 本地计算量不变

---

## 5. 纯 Node.js 实现

### 修改时间
2026-03-27

### 修改目的
解决 Rust 二进制文件在某些平台上因 TLS 证书问题导致 HTTPS 请求失败的问题。

### 问题背景
原有的 Rust 原生二进制文件使用 rustls TLS 库，通过 webpki-roots 内置的根证书列表验证 HTTPS 连接。但在某些网络环境下（特别是中国大陆），服务器使用 TrustAsia 等本地 CA 签发的证书，不在 webpki-roots 信任列表中，导致请求静默失败。

### 解决方案

#### 方案 A：纯 Node.js 实现（推荐）

添加 `glm-plan-usage-pure.js`，使用 Node.js 内置 `https` 模块：
- 自动使用操作系统证书存储（Windows 用 SChannel，macOS 用 Keychain，Linux 用 OpenSSL）
- 无需编译，只要有 Node.js 环境即可运行
- 避免所有 TLS 兼容性问题

#### 方案 B：Rust 切换到 native-tls

修改 `Cargo.toml`，将 ureq 从 rustls 切换到 native-tls：
```toml
# 修改前
ureq = { version = "2.10", features = ["json"] }

# 修改后
ureq = { version = "2.10", features = ["json", "native-tls"], default-features = false }
```

native-tls 使用操作系统原生 TLS 库，自动信任系统证书。

### 文件变更

**新增文件：**
- `npm/main/bin/glm-plan-usage-pure.js` — 纯 Node.js 实现

**修改文件：**
- `Cargo.toml` — ureq 切换到 native-tls
- `Cargo.lock` — 依赖更新
- `README.md` — 添加 Node.js 安装方式
- `README_en.md` — 添加 Node.js 安装方式

### 环境变量支持

Node.js 版本支持两种环境变量格式：

```javascript
const token = getEnv("GLM_AUTH_TOKEN") || getEnv("ANTHROPIC_AUTH_TOKEN");
const baseUrl = getEnv("GLM_BASE_URL") || getEnv("ANTHROPIC_BASE_URL") || "https://open.bigmodel.cn/api/anthropic";
```

- `GLM_*` 优先级高于 `ANTHROPIC_*`
- 兼容不同版本的 Claude Code 配置

### 使用方式

**Node.js 版本配置：**
```json
{
  "statusLine": {
    "type": "command",
    "command": "node C:/Users/用户名/.claude/glm-plan-usage/glm-plan-usage-pure.js",
    "padding": 0
  }
}
```

**Rust 二进制配置：**
```json
{
  "statusLine": {
    "type": "command",
    "command": "%USERPROFILE%\\.claude\\glm-plan-usage\\glm-plan-usage.exe",
    "padding": 0
  }
}
```

### 两种方案对比

| 特性 | Node.js 版本 | Rust 版本 (native-tls) |
|------|-------------|----------------------|
| 依赖 | 需要 Node.js | 无依赖 |
| 编译 | 无需编译 | 需要 Rust 工具链 |
| TLS | 系统证书 | 系统证书 |
| 启动速度 | 稍慢 | 更快 |
| 分发 | 单文件 | 单文件 |
| 兼容性 | 最佳 | 最佳 |

---

## 安装与配置

### 安装位置

| 项目 | 路径 |
|------|------|
| 程序 | `C:\Users\18773\.claude\glm-plan-usage\glm-plan-usage.exe` |
| 备份 | `C:\Users\18773\.claude\glm-plan-usage\glm-plan-usage.exe.old` |
| 启动脚本 | `C:\Users\18773\.claude\glm-plan-usage\glm-plan-usage.bat` |
| 源码 | `C:\Users\18773\Desktop\glm-plan-usage-main` |
| 配置 | `C:\Users\18773\.factory\settings.json` |

### 编译命令

```powershell
cd "C:\Users\18773\Desktop\glm-plan-usage-main"
cargo build --release
```

### 环境变量设置

```powershell
$env:GLM_AUTH_TOKEN="你的token"
$env:GLM_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

### 使用说明

重启 Factory Droid 后生效。

---

## 状态栏显示汇总

**老套餐：**
```
🪙 5% (⏰ 23:00) · 📊 93/9000 · 🌐 0/1000 · ⚡ 3.38M
│    │         │         │          │
│    │         │          │          └── 5小时Token消耗
│    │         │          └── MCP配额（30天工具调用）
│    │         └── 5小时调用次数/上限
│    │         └── 重置时间（绝对时间）
│    └── 5小时配额使用率
└── Token 图标
```

**新套餐：**
```
🪙 5% (⏰ 23:00) · 📊 93/6000 · 📅 300/30000 · 🌐 0/1000 · ⚡ 3.38M
│    │         │         │            │            │          │
│    │         │         │            │            │          └── 5小时Token消耗
│    │         │         │            │            └── MCP配额（30天工具调用）
│    │         │         │            └── 周限量调用次数/上限
│    │         │         └── 5小时调用次数/上限
│    │         │         └── 重置时间（绝对时间）
│    └── 5小时配额使用率
└── Token 图标
```
