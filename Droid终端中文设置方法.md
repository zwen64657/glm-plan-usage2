# Droid 终端中文设置方法

## 问题说明

| 环境 | 语言显示 | 原因 |
|------|---------|------|
| VS Code 内置终端 | 中文 | 读取 VS Code 的 locale 设置 |
| 独立终端（PowerShell/CMD） | 英文 | 无语言环境变量 |

## VS Code 为什么默认显示中文

VS Code 有独立的语言配置文件，droid 在 VS Code 终端中运行时会自动检测并读取这个设置。

**配置文件位置：**
```
C:\Users\<用户名>\.vscode\argv.json
```

**配置内容示例：**
```json
{
  "locale": "zh-cn"
}
```

这就是为什么在 VS Code 内打开终端运行 droid 会显示中文，而独立终端不会的原因。

## 解决方案

### 方法一：永久设置（推荐）

通过 Windows 系统设置环境变量：

1. 打开 **设置 → 系统 → 关于 → 高级系统设置**
2. 点击 **环境变量**
3. 在 **用户变量** 区域点击 **新建**
4. 填写：
   - 变量名：`LANG`
   - 变量值：`zh-CN`
5. 点击 **确定** 保存

### 方法二：命令行设置

```cmd
setx LANG "zh-CN"
```

### 方法三：临时设置（仅当前会话有效）

**PowerShell：**
```powershell
$env:LANG = "zh-CN"
```

**CMD：**
```cmd
set LANG=zh-CN
```

## 使设置生效

设置完成后：

1. 关闭所有已打开的终端窗口
2. 重新打开新终端
3. 运行 `droid` 验证中文显示

> 如果仍然显示英文，尝试重启电脑使环境变量完全生效。

## 验证方法

**PowerShell：**
```powershell
$env:LANG
```

**CMD：**
```cmd
echo %LANG%
```

如果输出 `zh-CN`，说明设置成功。
