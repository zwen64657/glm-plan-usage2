# glm-plan-usage

[English](README_en.md)

Claude Code 状态栏插件，实时显示 GLM / MiniMax / Kimi 多平台算力套餐使用量。

> **提示：** GLM 和 MiniMax 已测试通过，Kimi 未实际测试。

## 功能

- 🔋 5 小时 Token 配额使用率 + 重置时间
- 📊 5 小时模型调用次数
- ⚡ 5 小时 Token 消耗总量
- 📅 周限量百分比（新套餐）
- 🌐 30 天 MCP 配额
- 自动检测智谱（bigmodel.cn）、ZAI（api.z.ai）、MiniMax（minimaxi.com）、Kimi（kimi.com）平台
- 自动识别 GLM / MiniMax / Kimi 模型，非支持模型不显示
- 2 分钟缓存
- **智能字符模式检测** - 自动选择 Emoji 或 ASCII 模式
  - Windows 11 → Emoji 模式 🔋📊⚡📅🌐⏰
  - Windows 10 → ASCII 模式 $#k%MT（避免乱码）
- **极简模式** - 通过 `USAGE_MINIMAL=1` 去除所有图标
- **自动检测凭证** - 自动读取 Claude Code `settings.json`，无需额外配置环境变量
- **NO_COLOR 支持** - 遵循 [no-color.org](https://no-color.org) 标准

## 显示示例

### 极简模式（`USAGE_MINIMAL=1`）

```
glm-5.1 5% · 23:00 · 93 · 25% · 0/1000 · 3.38M
```

### 正常模式

#### GLM 平台

老套餐（无周限量）：
```
GLM 🔋 5% · ⏰ 23:00 · 📊 93 · 🌐 0/1000 · ⚡ 3.38M
```

新套餐（有周限量）：
```
GLM 🔋 5% · ⏰ 23:00 · 📊 93 · 📅 25% · 🌐 0/1000 · ⚡ 3.38M
```

### MiniMax 平台

```
MiniMax 🔋 5% · ⏰ 23:00 · 📊 93/1200 · 📅 25%
```

### Kimi 平台

```
Kimi 🔋 12% · ⏰ 18:00 · 📅 8%
```

### ASCII 模式（Windows 10）

GLM 老套餐（无周限量）：
```
GLM $ 5% · T 23:00 · # 93 · M 0/1000 · k 3.38M
```

GLM 新套餐（有周限量）：
```
GLM $ 5% · T 23:00 · # 93 · % 25% · M 0/1000 · k 3.38M
```

MiniMax：
```
MiniMax $ 5% · T 23:00 · # 93/1200 · % 25%
```

Kimi：
```
Kimi $ 12% · T 18:00 · % 8%
```

**字符映射：**
- 🔋 → $ (Token 配额)
- 📊 → # (调用次数)
- ⚡ → k (Token 消耗)
- 📅 → % (周限量)
- 🌐 → M (MCP 配额)
- ⏰ → T (重置时间)

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

## 环境变量

> **自动检测：** 如果已在 Claude Code `settings.json` 中配置了 `ANTHROPIC_AUTH_TOKEN` 和 `ANTHROPIC_BASE_URL`，无需额外设置环境变量。插件会自动读取。

### GLM 平台

| 变量名 | 必需 | 说明 |
|--------|------|------|
| `ANTHROPIC_AUTH_TOKEN` | 是 | 智谱 API Key |
| `ANTHROPIC_BASE_URL` | 否 | 默认 `https://open.bigmodel.cn/api/anthropic` |

### MiniMax 平台

| 变量名 | 必需 | 说明 |
|--------|------|------|
| `ANTHROPIC_AUTH_TOKEN` | 是 | MiniMax API Key |
| `ANTHROPIC_BASE_URL` | 是 | 设为 `https://api.minimaxi.com/anthropic` |
| `USAGE_MINIMAX_COOKIE` | 是 | MiniMax Cookie（用于查询用量） |

MiniMax 用量查询 API 需要 Cookie 认证，不支持 API Key。获取步骤：

1. 登录 MiniMax 开发平台
2. 进入 **账户管理 → 套餐管理 → Token Plan**
3. F12 → 网络（Network）→ 搜索 `remains`
4. 点击请求 → 查看请求头中 Cookie → 找到 `HERTZ-SESSION=xxx`
5. 复制 `=` 后面的值

设置环境变量：

```cmd
setx USAGE_MINIMAX_COOKIE "复制的值"
```

或通过系统设置：Win+R → `sysdm.cpl` → 高级 → 环境变量 → 新建用户变量。

> **注意：** Cookie 会过期，过期后需重新获取。设置后需重启终端/droid 才能生效。

### Kimi 平台

| 变量名 | 必需 | 说明 |
|--------|------|------|
| `ANTHROPIC_API_KEY` | 是 | Kimi API Key |
| `ANTHROPIC_BASE_URL` | 是 | 设为 Kimi 的 API 地址 |

## 配置选项

### 字符模式（可选）

程序会自动检测操作系统并选择合适的字符模式，无需手动配置。

**自动检测：**
- Windows 11（Build >= 22000）→ Emoji 模式
- Windows 10（Build < 22000）→ ASCII 模式

**手动强制覆盖（特殊情况下使用）：**

如果你想手动指定字符模式，可以设置以下环境变量：

**强制使用 Emoji 模式：**
```powershell
# Windows PowerShell
$env:USAGE_FORCE_EMOJI="1"

# Linux/macOS
export USAGE_FORCE_EMOJI=1
```

**强制使用 ASCII 模式：**
```powershell
# Windows PowerShell
$env:USAGE_FORCE_ASCII="1"

# Linux/macOS
export USAGE_FORCE_ASCII=1
```

**何时需要手动配置：**
- 你的终端实际上支持 emoji，但自动检测误判为不支持
- 你的终端不支持 emoji，显示乱码
- 你想对比不同模式的显示效果

**注意：** 大多数情况下不需要手动配置，自动检测已经足够好了。

### 显示控制

| 变量名 | 说明 |
|--------|------|
| `USAGE_MINIMAL=1` | 极简模式，去除所有图标，只显示数据 |
| `NO_COLOR` | 禁用颜色输出（遵循 [no-color.org](https://no-color.org) 标准） |
| `USAGE_NO_COLOR` | 同上，项目专用变量名 |
| `USAGE_DEBUG=1` | 启用调试日志输出到 `~/.claude/glm-plan-usage/debug.log` |
| `USAGE_CLAUDE_CONFIG_PATH` | 自定义 Claude Code 配置文件路径 |

## 安全说明

### 凭证管理

- ✅ **自动读取凭证** - 优先从环境变量读取，自动回退到 Claude Code `settings.json`
- ✅ **HTTPS 传输** - 所有 API 请求通过加密连接传输
- ✅ **无日志输出** - Token 不会出现在日志或错误消息中
- ✅ **无配置文件写入** - API Key 不会被写入任何文件

### 安全最佳实践

1. **不要将 API Key 写入配置文件**
   - 使用环境变量管理凭证
   - 配置文件已加入 `.gitignore` 防止意外提交

2. **不要在 Shell 配置文件中硬编码 Key**
   ```bash
   # ❌ 不推荐：在 .bashrc/.zshrc 中直接写入
   export ANTHROPIC_AUTH_TOKEN="sk-xxxxx"

   # ✅ 推荐：使用 .env 文件（记得添加到 .gitignore）
   # .env 文件内容：
   ANTHROPIC_AUTH_TOKEN=sk-xxxxx
   ```

3. **定期轮换 API Key**
   - 定期更换 API Key 以降低泄漏风险
   - 如果怀疑 Key 泄漏，立即撤销并重新生成

4. **多用户环境注意**
   - 在共享服务器上，环境变量可能被同机其他进程读取
   - 建议使用容器或独立用户环境

### 已知限制

- 环境变量在进程运行期间可被 `ps` 命令或 `/proc/*/environ` 读取（需要系统级访问权限）
- Core dump 可能包含内存中的敏感数据（生产环境建议禁用 coredump）
