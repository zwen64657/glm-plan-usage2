# glm-plan-usage

[简体中文](README.md) | English

A Claude Code plugin that displays GLM (ZHIPU/ZAI) coding plan usage statistics in the status bar.

![demo](screenshots/demo.png)

## Features

- 📊 **Real-time Usage Tracking**: Display Token and MCP usage percentages
- 🎨 **Color-coded Warnings**: Green (0-79%), Yellow (80-94%), Red (95-100%)
- ⚡ **Smart Caching**: 5-minute cache to reduce API calls
- 🔍 **Auto Platform Detection**: Supports ZAI and ZHIPU platforms
- 🌍 **Cross-platform Support**: Works on Windows, macOS, and Linux

## Installation

### Install via npm (Recommended)

```bash
npm install -g @jukanntenn/glm-plan-usage
```

For users experiencing network issues, use npm mirror for faster installation:

```bash
npm install -g @jukanntenn/glm-plan-usage --registry https://registry.npmmirror.com
```

Update:

```bash
npm update -g @jukanntenn/glm-plan-usage
```

<details>
<summary>Manual Installation (click to expand)</summary>

Or download manually from [Releases](https://github.com/jukanntenn/glm-plan-usage/releases):

#### Linux

#### Option 1: Dynamically Linked (Recommended)
```bash
mkdir -p ~/.claude/glm-plan-usage
wget https://github.com/jukanntenn/glm-plan-usage/releases/latest/download/glm-plan-usage-linux-x64.tar.gz
tar -xzf glm-plan-usage-linux-x64.tar.gz
cp glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage
```
*System requirements: Ubuntu 22.04+, CentOS 9+, Debian 11+, RHEL 9+ (glibc 2.35+)*

#### Option 2: Statically Linked (Universal Compatibility)
```bash
mkdir -p ~/.claude/glm-plan-usage
wget https://github.com/jukanntenn/glm-plan-usage/releases/latest/download/glm-plan-usage-linux-x64-musl.tar.gz
tar -xzf glm-plan-usage-linux-x64-musl.tar.gz
cp glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage
```
*Works on any Linux distribution (statically linked, no dependencies)*

#### macOS (Intel)

```bash
mkdir -p ~/.claude/glm-plan-usage
wget https://github.com/jukanntenn/glm-plan-usage/releases/latest/download/glm-plan-usage-macos-x64.tar.gz
tar -xzf glm-plan-usage-macos-x64.tar.gz
cp glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage
```

#### macOS (Apple Silicon)

```bash
mkdir -p ~/.claude/glm-plan-usage
wget https://github.com/jukanntenn/glm-plan-usage/releases/latest/download/glm-plan-usage-macos-arm64.tar.gz
tar -xzf glm-plan-usage-macos-arm64.tar.gz
cp glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage
```

#### Windows

```powershell
# Create directory and download
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.claude\glm-plan-usage"
Invoke-WebRequest -Uri "https://github.com/jukanntenn/glm-plan-usage/releases/latest/download/glm-plan-usage-windows-x64.zip" -OutFile "glm-plan-usage-windows-x64.zip"
Expand-Archive -Path "glm-plan-usage-windows-x64.zip" -DestinationPath "."
Move-Item "glm-plan-usage.exe" "$env:USERPROFILE\.claude\glm-plan-usage\"
```

</details>

### Build from Source

```bash
git clone https://github.com/jukanntenn/glm-plan-usage.git
cd glm-plan-usage
cargo build --release
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
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

Restart Claude Code, the status bar will display:

```text
🪙 32% (⌛️ 1:44) · 🌐 20/100
   │  │           │     └─ MCP usage (used/total)
   │  │           └─ Separator
   │  └─ Token countdown (hours:minutes)
   └─ Token usage percentage

```

If you are already using [CCometixLine](https://github.com/Haleclipse/CCometixLine) or other similar plugins, you can create scripts to combine them:

**Linux/macOS:**

`~/.claude/status-line-combined.sh` script example:

```bash
#!/bin/bash

# Read JSON input from stdin
INPUT=$(cat)

# Run both commands with the same input
CCLINE_OUTPUT=$(echo "$INPUT" | ~/.claude/ccline/ccline 2>/dev/null)
GLM_OUTPUT=$(echo "$INPUT" | ~/.claude/glm-plan-usage/glm-plan-usage 2>/dev/null)

# Build combined output
OUTPUT=""

# Add ccline output if available
if [ -n "$CCLINE_OUTPUT" ]; then
    OUTPUT="$CCLINE_OUTPUT"
fi

# Add glm-plan-usage output if available
if [ -n "$GLM_OUTPUT" ]; then
    if [ -n "$OUTPUT" ]; then
        OUTPUT="$OUTPUT | $GLM_OUTPUT"
    else
        OUTPUT="$GLM_OUTPUT"
    fi
fi

# Print combined output
if [ -n "$OUTPUT" ]; then
    printf "%s" "$OUTPUT"
fi
```

Add execution permission: `chmod +x ~/.claude/status-line-combined.sh`

Configure in Claude Code `settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/status-line-combined.sh",
    "padding": 0
  }
}
```

**Windows (PowerShell):**

`%USERPROFILE%\.claude\status-line-combined.ps1` script example:

```powershell
# Read JSON input from stdin
$InputString = [Console]::In.ReadToEnd()

# Run both commands with the same input
$CclineOutput = $InputString | & "$env:USERPROFILE\.claude\ccline\ccline.exe" 2>$null
$GlmOutput = $InputString | & "$env:USERPROFILE\.claude\glm-plan-usage\glm-plan-usage.exe" 2>$null

# Build combined output
$Output = ""

# Add ccline output if available
if (-not [string]::IsNullOrEmpty($CclineOutput)) {
    $Output = $CclineOutput
}

# Add glm-plan-usage output if available
if (-not [string]::IsNullOrEmpty($GlmOutput)) {
    if (-not [string]::IsNullOrEmpty($Output)) {
        $Output = "$Output | $GlmOutput"
    } else {
        $Output = $GlmOutput
    }
}

# Print combined output
if (-not [string]::IsNullOrEmpty($Output)) {
    Write-Host -NoNewline $Output
}
```

Grant script execution permission in PowerShell: `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`

Configure in Claude Code `settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "powershell.exe -File %USERPROFILE%\\.claude\\status-line-combined.ps1",
    "padding": 0
  }
}
```

## Environment Variables

**Note:** These variables are typically already configured in your Claude Code `settings.json`. If not, you can set them manually:

**Linux/macOS:**

```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

**Windows (Command Prompt):**

```cmd
set ANTHROPIC_AUTH_TOKEN=your-token-here
set ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic
```

**Windows (PowerShell):**

```powershell
$env:ANTHROPIC_AUTH_TOKEN="your-token-here"
$env:ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

## License

MIT
