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
- **Smart character mode detection** - Automatically choose Emoji or ASCII mode
  - Windows 11 → Emoji mode 🪙📊⚡📅🌐⏰
  - Windows 10 → ASCII mode $#k%MT (to avoid garbled text)

## Display Example

### Emoji Mode (Windows 11)

Old plan (no weekly quota):
```
🪙 5% (⏰ 23:00) · 📊 93 · 🌐 0/1000 · ⚡ 3.38M
```

New plan (with weekly quota):
```
🪙 5% (⏰ 23:00) · 📊 93 · 📅 25% · 🌐 0/1000 · ⚡ 3.38M
```

### ASCII Mode (Windows 10)

Old plan (no weekly quota):
```
$ 5% (T 23:00) · # 93 · M 0/1000 · k 3.38M
```

New plan (with weekly quota):
```
$ 5% (T 23:00) · # 93 · % 25% · M 0/1000 · k 3.38M
```

**Character Mapping:**
- 🪙 → $ (Token quota)
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
