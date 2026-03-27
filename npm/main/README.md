# @jukanntenn/glm-plan-usage

GLM Plan Usage - StatusLine plugin for Claude Code

## Installation

```bash
npm install -g @jukanntenn/glm-plan-usage
```

For users experiencing network issues, use npm mirror for faster installation:

```bash
npm install -g @jukanntenn/glm-plan-usage --registry https://registry.npmmirror.com
```

## Features

- 📊 **Monitor**: Display GLM (ZHIPU/ZAI) coding plan usage statistics
- 🌍 **Cross-platform**: Works on Windows, macOS, and Linux
- 📦 **Easy installation**: One command via npm
- 🎨 **Beautiful**: Color-coded warning levels based on usage percentage

## Usage

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

## More Information

- GitHub: https://github.com/jukanntenn/glm-plan-usage
- Issues: https://github.com/jukanntenn/glm-plan-usage/issues
- License: MIT
