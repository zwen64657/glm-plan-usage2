# GLM API 接口调研记录

## 接口信息
- **端点**: `/monitor/usage/quota/limit`
- **基础URL**: `https://open.bigmodel.cn/api`
- **认证方式**: Bearer Token (ANTHROPIC_AUTH_TOKEN)

## 实际 API 响应示例

> **最后更新**: 2026-02-14

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "limits": [
      {
        "type": "TIME_LIMIT",
        "unit": 5,
        "number": 1,
        "usage": 100,
        "currentValue": 28,
        "remaining": 72,
        "percentage": 28,
        "nextResetTime": 1772615765983,
        "usageDetails": [
          {
            "modelCode": "search-prime",
            "usage": 67
          },
          {
            "modelCode": "web-reader",
            "usage": 33
          },
          {
            "modelCode": "zread",
            "usage": 0
          }
        ]
      },
      {
        "type": "TOKENS_LIMIT",
        "unit": 3,
        "number": 5,
        "percentage": 1,
        "nextResetTime": 1771073738808
      }
    ],
    "level": "lite"
  },
  "success": true
}
```

## 字段说明

### 通用字段
- `type`: 限额类型
  - `TIME_LIMIT`: MCP/工具调用次数限制
  - `TOKENS_LIMIT`: Token 使用量限制
- `percentage`: 使用百分比 (0-100)

### TIME_LIMIT 特有字段
- `unit`: 单位代码 (5)
- `number`: 数量 (1)
- `usage`: 总量限制 (次数)
- `currentValue`: 当前已使用量
- `remaining`: 剩余可用量
- `nextResetTime`: 下次重置时间戳（毫秒）
- `usageDetails`: 各个 MCP 工具的使用详情
  - `modelCode`: 工具代码
  - `usage`: 该工具的使用次数

### TOKENS_LIMIT 字段
- `unit`: 单位代码 (3) - 可能表示时间单位
- `number`: 数量 (5) - 可能表示时间窗口大小
- `percentage`: 使用百分比 (0-100)
- `nextResetTime`: 下次重置时间戳（毫秒）

> ⚠️ **注意**: TOKENS_LIMIT **不再返回** `usage`、`currentValue`、`remaining` 字段，仅有 `percentage` 表示使用百分比。

### 顶层字段
- `level`: 账户等级 (如 "lite")

## 关键发现

### 1. TOKENS_LIMIT 字段简化
- **不再返回** `usage`、`currentValue`、`remaining` 字段
- 仅返回 `percentage` 表示使用百分比
- 显示格式建议: 仅显示百分比和倒计时 (如 "🪙 1% (⌛️ 1:23)")

### 2. TIME_LIMIT 现在包含重置时间
- `nextResetTime` 字段现在也会在 TIME_LIMIT 中返回
- 数据类型: 毫秒级时间戳 (i64)
- 需要除以 1000 转换为秒

### 3. 新增账户等级字段
- `level`: 账户等级 (如 "lite")
- 可能影响配额限制

### 4. MCP 计数
- `usage`: 总次数限制
- `currentValue`: 已使用次数
- `remaining`: 剩余次数
- 显示格式: 原始计数 (如 "🌐 28/100")
- `usageDetails`: 各工具详细使用情况

### 5. 时间窗口推断
- TOKENS_LIMIT: `unit=3, number=5` 可能表示 5 小时滚动窗口
- TIME_LIMIT: `unit=5, number=1` 可能表示 1 个自然月
- **需要进一步确认 `unit` 的具体含义**

## 设计影响

### 当前数据结构

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct QuotaLimitItem {
    #[serde(rename = "type")]
    pub quota_type: String,
    #[serde(default)]
    pub usage: i64,              // TIME_LIMIT 有效，TOKENS_LIMIT 不返回
    #[serde(rename = "currentValue", default)]
    pub current_value: i64,      // TIME_LIMIT 有效，TOKENS_LIMIT 不返回
    pub percentage: i32,
    #[serde(rename = "nextResetTime", default)]
    pub next_reset_time: Option<i64>,  // 两种类型都返回
}

#[derive(Debug, Clone)]
pub struct QuotaUsage {
    pub used: i64,
    pub limit: i64,
    pub percentage: u8,
    pub time_window: String,
    pub reset_at: Option<i64>,  // 秒级时间戳
}
```

### 倒计时计算注意事项
- `nextResetTime` 是毫秒时间戳，需要除以 1000 转换为秒
- TIME_LIMIT 和 TOKENS_LIMIT 都会返回此字段
- 时间戳示例: `1772615765983` (毫秒) → `1772615765` (秒)

## 时间戳示例
- TIME_LIMIT 示例: `1772615765983` (毫秒) → `1772615765` (秒)
- TOKENS_LIMIT 示例: `1771073738808` (毫秒) → `1771073738` (秒)

## 状态
- ✅ 文档已更新至最新 API 响应格式
- ✅ 代码已正确处理可选字段 (`#[serde(default)]`)
- ✅ 倒计时显示已实现
