# glm-plan-usage

[English](README_en.md)

Claude Code 状态栏插件，实时显示 GLM（智谱/ZAI）算力套餐使用量。

## 功能

- 🪙 5 小时 Token 配额使用率 + 重置时间
- 📊 5 小时模型调用次数
- ⚡ 5 小时 Token 消耗总量
- 📅 周限量百分比（新套餐）
- 🌐 30 天 MCP 配额
- 自动检测智谱（bigmodel.cn）和 ZAI（api.z.ai）平台
- 仅 GLM 模型显示用量
- 2 分钟缓存

## 显示示例

老套餐（无周限量）：
```
🪙 5% (⏰ 23:00) · 📊 93 · 🌐 0/1000 · ⚡ 3.38M
```

新套餐（有周限量）：
```
🪙 5% (⏰ 23:00) · 📊 93 · 📅 25% · 🌐 0/1000 · ⚡ 3.38M
```

## 两个版本

| 版本 | 文件位置 | 说明 |
|------|----------|------|
| Rust | `target/release/glm-plan-usage` | 编译后的二进制文件 |
| Node.js | `npm/main/bin/glm-plan-usage-pure.js` | 纯 JS 实现，无需编译 |

## 安装

### Node.js 版本（推荐）

将 `npm/main/bin/glm-plan-usage-pure.js` 放到 `~/.claude/glm-plan-usage/` 目录。

在 Claude Code 的 `settings.json` 中配置：

```json
{
  "statusLine": {
    "type": "command",
    "command": "node ~/.claude/glm-plan-usage/glm-plan-usage-pure.js",
    "padding": 0
  }
}
```

### Rust 版本

将编译好的 `target/release/glm-plan-usage` 文件放到 `~/.claude/glm-plan-usage/` 目录。

在 Claude Code 的 `settings.json` 中配置：

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/glm-plan-usage/glm-plan-usage",
    "padding": 0
  }
}
```

### Windows 路径

Windows 下将路径中的 `~` 替换为 `C:/Users/你的用户名`。
