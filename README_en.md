# glm-plan-usage

English | [简体中文](README.md)

Claude Code status bar plugin that displays real-time GLM / MiniMax / Kimi multi-platform usage statistics.

> **Note:** GLM and MiniMax have been tested successfully. Kimi has not been tested yet. The Node.js version is recommended; the Rust version may have compatibility issues in some environments.

## Features

- 🔋 5-hour Token quota usage percentage + reset time
- 📊 5-hour model call count
- ⚡ 5-hour Token consumption
- 📅 Weekly quota percentage (new plans)
- 🌐 30-day MCP quota
- Auto-detect ZHIPU (bigmodel.cn), ZAI (api.z.ai), MiniMax (minimaxi.com), Kimi (kimi.com) platforms
- Auto-identify GLM / MiniMax / Kimi models; non-supported models are hidden
- 2-minute cache
- **Smart character mode detection** - Automatically choose Emoji or ASCII mode
  - Windows 11 → Emoji mode 🔋📊⚡📅🌐⏰
  - Windows 10 → ASCII mode $#k%MT (to avoid garbled text)

## Display Example

### GLM Platform

Old plan (no weekly quota):
```
GLM 🔋 5% · ⏰ 23:00 · 📊 93 · 🌐 0/1000 · ⚡ 3.38M
```

New plan (with weekly quota):
```
GLM 🔋 5% · ⏰ 23:00 · 📊 93 · 📅 25% · 🌐 0/1000 · ⚡ 3.38M
```

### MiniMax Platform

```
MiniMax 🔋 5% · ⏰ 23:00 · 📊 93/1200 · 📅 25%
```

### Kimi Platform

```
Kimi 🔋 12% · ⏰ 18:00 · 📅 8%
```

### ASCII Mode (Windows 10)

GLM old plan (no weekly quota):
```
GLM $ 5% · T 23:00 · # 93 · M 0/1000 · k 3.38M
```

GLM new plan (with weekly quota):
```
GLM $ 5% · T 23:00 · # 93 · % 25% · M 0/1000 · k 3.38M
```

MiniMax:
```
MiniMax $ 5% · T 23:00 · # 93/1200 · % 25%
```

Kimi:
```
Kimi $ 12% · T 18:00 · % 8%
```

**Character Mapping:**
- 🔋 → $ (Token quota)
- 📊 → # (Call count)
- ⚡ → k (Token consumption)
- 📅 → % (Weekly quota)
- 🌐 → M (MCP quota)
- ⏰ → T (Reset time)

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

## Environment Variables

### GLM Platform

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_AUTH_TOKEN` | Yes | ZHIPU API Key |
| `ANTHROPIC_BASE_URL` | No | Default: `https://open.bigmodel.cn/api/anthropic` |

### MiniMax Platform

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_AUTH_TOKEN` | Yes | MiniMax API Key |
| `ANTHROPIC_BASE_URL` | Yes | Set to `https://api.minimaxi.com/anthropic` |
| `HERTZ_SESSION` | Yes | MiniMax Cookie (required for usage query) |

MiniMax usage query API requires Cookie authentication; API Key is not supported. To obtain:

1. Log in to MiniMax Developer Platform
2. Go to **Account Management → Plan Management → Token Plan**
3. Open DevTools (F12) → Network tab → search for `remains`
4. Click the request → check request headers for Cookie → find `HERTZ-SESSION=xxx`
5. Copy the value after `=`

Set environment variable:

```cmd
setx HERTZ_SESSION "the_copied_value"
```

Or via System Settings: Win+R → `sysdm.cpl` → Advanced → Environment Variables → New user variable.

> **Note:** The Cookie expires periodically. After setting, restart your terminal/droid for it to take effect.

### Kimi Platform

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | Yes | Kimi API Key |
| `ANTHROPIC_BASE_URL` | Yes | Set to Kimi's API URL |

## Configuration Options

### Character Mode (Optional)

The program automatically detects the operating system and selects the appropriate character mode. No manual configuration is needed.

**Automatic Detection:**
- Windows 11 (Build >= 22000) → Emoji mode
- Windows 10 (Build < 22000) → ASCII mode

**Manual Override (use in special cases):**

If you want to manually specify the character mode, set the following environment variables:

**Force Emoji Mode:**
```powershell
# Windows PowerShell
$env:GLM_FORCE_EMOJI="1"
```

**Force ASCII Mode:**
```powershell
# Windows PowerShell
$env:GLM_FORCE_ASCII="1"
```

**When to use manual configuration:**
- Your terminal actually supports emoji, but auto-detection incorrectly identifies it as unsupported
- Your terminal doesn't support emoji and displays garbled text
- You want to compare the display of different modes

**Note:** In most cases, manual configuration is not needed. Auto-detection works well enough.
