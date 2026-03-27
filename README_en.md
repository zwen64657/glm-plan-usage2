# glm-plan-usage

简体中文 | [English](README_en.md)

A Claude Code status bar plugin that displays real-time GLM (ZHIPU/ZAI) coding plan usage statistics.

Forked from [jukanntenn/glm-plan-usage](https://github.com/jukanntenn/glm-plan-usage) with additional features including call count, token consumption, and weekly quota display.

![demo](screenshots/demo.png)

## Features

- **Real-time Usage Tracking**: 5-hour Token quota usage percentage with reset time
- **Call Count Statistics**: Model call count within the 5-hour window vs plan limit
- **Token Consumption**: Total token usage within the 5-hour window (smart K/M formatting)
- **Weekly Quota Support**: Auto-detect and display weekly quota (available on some plans)
- **MCP Quota Display**: 30-day MCP tool call count
- **Color-coded Warnings**: Green (0-79%), Yellow (80-94%), Red (95-100%)
- **Auto Platform Detection**: Supports ZHIPU (bigmodel.cn) and ZAI (api.z.ai) with automatic timezone adaptation
- **Smart Model Filtering**: Automatically hides usage info when using non-GLM models
- **Smart Caching**: 2-minute cache to reduce API calls
- **Cross-platform Support**: Works on Windows, macOS, and Linux

## Status Bar Display

### Old Plan (no weekly quota)

```
🪙 5% (⏰ 23:00) · 📊 93/9000 · 🌐 0/1000 · ⚡ 3.38M
```

### New Plan (with weekly quota)

```
🪙 5% (⏰ 23:00) · 📊 93/6000 · 📅 300/30000 · 🌐 0/1000 · ⚡ 3.38M
```

### Legend

| Icon | Meaning | Description |
|------|---------|-------------|
| 🪙 | 5-hour Token Quota | Usage percentage + reset time |
| 📊 | 5-hour Call Count | Current calls / plan limit |
| 📅 | Weekly Quota (new plan) | Current used / plan limit |
| 🌐 | MCP Quota | 30-day tool call count |
| ⚡ | Token Consumption | Total tokens used in 5-hour window |

### Quota Reference Table

#### Old Plan

| Plan | 5-hour Call Limit |
|------|-------------------|
| Lite | 1,800 |
| Pro | 9,000 |
| Max | 36,000 |

#### New Plan

| Plan | 5-hour Call Limit | Weekly Limit |
|------|-------------------|--------------|
| Lite | 1,200 | 6,000 |
| Pro | 6,000 | 30,000 |
| Max | 24,000 | 120,000 |

## Installation

### Option 1: Pure Node.js Implementation (Recommended)

No compilation needed. Uses Node.js built-in HTTPS module with system certificate store, avoiding TLS compatibility issues.

**Linux/macOS:**

```json
{
  "statusLine": {
    "type": "command",
    "command": "node /path/to/glm-plan-usage-pure.js",
    "padding": 0
  }
}
```

**Windows:**

```json
{
  "statusLine": {
    "type": "command",
    "command": "node C:/Users/YourUsername/.claude/glm-plan-usage/glm-plan-usage-pure.js",
    "padding": 0
  }
}
```

### Option 2: Build from Source

```bash
git clone https://github.com/zwen64657/glm-plan-usage2.git
cd glm-plan-usage2
cargo build --release
```

The compiled binary is at `target/release/glm-plan-usage` (Windows: `glm-plan-usage.exe`).

### Manual Installation (Rust Binary)

Copy the binary to Claude Code's plugin directory:

**Linux/macOS:**

```bash
mkdir -p ~/.claude/glm-plan-usage
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage
```

**Windows:**

```powershell
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.claude\glm-plan-usage"
Copy-Item target\release\glm-plan-usage.exe "$env:USERPROFILE\.claude\glm-plan-usage\"
```

## Configuration

Add to your Claude Code `settings.json`:

**Linux/macOS:**

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/glm-plan-usage/glm-plan-usage",
    "padding": 0
  }
}
```

**Windows:**

```json
{
  "statusLine": {
    "type": "command",
    "command": "%USERPROFILE%\\.claude\\glm-plan-usage\\glm-plan-usage.exe",
    "padding": 0
  }
}
```

Restart Claude Code. The plugin automatically reads `ANTHROPIC_AUTH_TOKEN` and `ANTHROPIC_BASE_URL` from Claude Code — no extra configuration needed.

### Supported Platforms

| Platform | BASE_URL | Timezone |
|----------|----------|----------|
| ZHIPU | `https://open.bigmodel.cn/api/anthropic` | Beijing Time (UTC+8) |
| ZAI | `https://api.z.ai/...` | UTC |

The plugin auto-detects the platform from `ANTHROPIC_BASE_URL` and adapts the timezone accordingly.

## Combining with Other Status Bar Plugins

If you're already using [CCometixLine](https://github.com/Haleclipse/CCometixLine) or similar plugins, create a combined script:

**Linux/macOS:** `~/.claude/status-line-combined.sh`

```bash
#!/bin/bash
INPUT=$(cat)
CCLINE_OUTPUT=$(echo "$INPUT" | ~/.claude/ccline/ccline 2>/dev/null)
GLM_OUTPUT=$(echo "$INPUT" | ~/.claude/glm-plan-usage/glm-plan-usage 2>/dev/null)

OUTPUT=""
[ -n "$CCLINE_OUTPUT" ] && OUTPUT="$CCLINE_OUTPUT"
if [ -n "$GLM_OUTPUT" ]; then
    [ -n "$OUTPUT" ] && OUTPUT="$OUTPUT | $GLM_OUTPUT" || OUTPUT="$GLM_OUTPUT"
fi
[ -n "$OUTPUT" ] && printf "%s" "$OUTPUT"
```

```bash
chmod +x ~/.claude/status-line-combined.sh
```

**Windows (PowerShell):** `%USERPROFILE%\.claude\status-line-combined.ps1`

```powershell
$InputString = [Console]::In.ReadToEnd()
$CclineOutput = $InputString | & "$env:USERPROFILE\.claude\ccline\ccline.exe" 2>$null
$GlmOutput = $InputString | & "$env:USERPROFILE\.claude\glm-plan-usage\glm-plan-usage.exe" 2>$null

$Output = ""
if (-not [string]::IsNullOrEmpty($CclineOutput)) { $Output = $CclineOutput }
if (-not [string]::IsNullOrEmpty($GlmOutput)) {
    if (-not [string]::IsNullOrEmpty($Output)) { $Output = "$Output | $GlmOutput" }
    else { $Output = $GlmOutput }
}
if (-not [string]::IsNullOrEmpty($Output)) { Write-Host -NoNewline $Output }
```

Then point your `settings.json` to the combined script.

## License

MIT
