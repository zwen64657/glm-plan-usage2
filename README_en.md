# glm-plan-usage

English | [简体中文](README.md)

Claude Code status bar plugin that displays real-time GLM / MiniMax / Kimi multi-platform usage statistics.

> **Note:** GLM and MiniMax have been tested successfully. Kimi has not been tested yet.

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
- **Minimal mode** - Strip all icons via `USAGE_MINIMAL=1`
- **Auto-detect credentials** - Reads Claude Code `settings.json` automatically, no extra env vars needed
- **NO_COLOR support** - Follows [no-color.org](https://no-color.org) convention

## Display Example

### Minimal Mode (`USAGE_MINIMAL=1`)

```
glm-5.1 5% · 23:00 · 93 · 25% · 0/1000 · 3.38M
```

### Normal Mode

#### GLM Platform

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

> **Auto-detection:** If you have `ANTHROPIC_AUTH_TOKEN` and `ANTHROPIC_BASE_URL` configured in Claude Code `settings.json`, no additional environment variable setup is needed. The plugin reads them automatically.

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
| `USAGE_MINIMAX_COOKIE` | Yes | MiniMax Cookie (required for usage query) |

MiniMax usage query API requires Cookie authentication; API Key is not supported. To obtain:

1. Log in to MiniMax Developer Platform
2. Go to **Account Management → Plan Management → Token Plan**
3. Open DevTools (F12) → Network tab → search for `remains`
4. Click the request → check request headers for Cookie → find `HERTZ-SESSION=xxx`
5. Copy the value after `=`

Set environment variable:

```cmd
setx USAGE_MINIMAX_COOKIE "the_copied_value"
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
$env:USAGE_FORCE_EMOJI="1"
```

**Force ASCII Mode:**
```powershell
# Windows PowerShell
$env:USAGE_FORCE_ASCII="1"
```

**When to use manual configuration:**
- Your terminal actually supports emoji, but auto-detection incorrectly identifies it as unsupported
- Your terminal doesn't support emoji and displays garbled text
- You want to compare the display of different modes

**Note:** In most cases, manual configuration is not needed. Auto-detection works well enough.

### Display Control

| Variable | Description |
|----------|-------------|
| `USAGE_MINIMAL=1` | Minimal mode — strip all icons, show data only |
| `NO_COLOR` | Disable color output (follows [no-color.org](https://no-color.org) convention) |
| `USAGE_NO_COLOR` | Same as above, project-specific variable name |
| `USAGE_DEBUG=1` | Enable debug logging to `~/.claude/glm-plan-usage/debug.log` |
| `USAGE_CLAUDE_CONFIG_PATH` | Custom Claude Code config file path |

## Security

### Credential Management

- ✅ **Auto-detect credentials** - Reads from environment variables first, falls back to Claude Code `settings.json`
- ✅ **HTTPS transmission** - All API requests transmitted over encrypted connections
- ✅ **No logging output** - Tokens never appear in logs or error messages
- ✅ **No config file writes** - API Keys are never written to any file

### Security Best Practices

1. **Don't write API Keys to config files**
   - Use environment variables for credential management
   - Config files are already in `.gitignore` to prevent accidental commits

2. **Don't hardcode Keys in Shell config files**
   ```bash
   # ❌ Not recommended: Directly in .bashrc/.zshrc
   export ANTHROPIC_AUTH_TOKEN="sk-xxxxx"

   # ✅ Recommended: Use .env file (remember to add to .gitignore)
   # .env file contents:
   ANTHROPIC_AUTH_TOKEN=sk-xxxxx
   ```

3. **Regularly rotate API Keys**
   - Change API Keys periodically to reduce leakage risk
   - If you suspect a Key has been leaked, revoke it immediately and regenerate

4. **Multi-user environment precautions**
   - On shared servers, environment variables may be readable by other processes on the same machine
   - Consider using containers or isolated user environments

### Known Limitations

- Environment variables can be read by `ps` command or `/proc/*/environ` while the process is running (requires system-level access)
- Core dumps may contain sensitive data from memory (consider disabling coredump in production environments)
