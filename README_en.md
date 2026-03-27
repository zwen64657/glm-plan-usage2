# glm-plan-usage

English | [简体中文](README.md)

Claude Code status bar plugin that displays real-time GLM (ZHIPU/ZAI) usage statistics.

## Features

- 🪙 5-hour Token quota usage percentage + reset time
- 📊 5-hour model call count
- ⚡ 5-hour Token consumption
- 📅 Weekly quota percentage (new plans)
- 🌐 30-day MCP quota
- Auto-detect ZHIPU (bigmodel.cn) and ZAI (api.z.ai) platforms
- Only show usage for GLM models
- 2-minute cache

## Display Example

Old plan (no weekly quota):
```
🪙 5% (⏰ 23:00) · 📊 93 · 🌐 0/1000 · ⚡ 3.38M
```

New plan (with weekly quota):
```
🪙 5% (⏰ 23:00) · 📊 93 · 📅 25% · 🌐 0/1000 · ⚡ 3.38M
```

## Two Versions

| Version | File Location | Description |
|---------|---------------|-------------|
| Rust | `target/release/glm-plan-usage` | Compiled binary |
| Node.js | `npm/main/bin/glm-plan-usage-pure.js` | Pure JS implementation, no compilation needed |

## Installation

### Node.js Version (Recommended)

Place `npm/main/bin/glm-plan-usage-pure.js` in `~/.claude/glm-plan-usage/` directory.

Add to Claude Code `settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "node ~/.claude/glm-plan-usage/glm-plan-usage-pure.js",
    "padding": 0
  }
}
```

### Rust Version

Place the compiled `target/release/glm-plan-usage` file in `~/.claude/glm-plan-usage/` directory.

Add to Claude Code `settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/glm-plan-usage/glm-plan-usage",
    "padding": 0
  }
}
```

### Windows Paths

On Windows, replace `~` with `C:/Users/YourUsername` in the paths.
