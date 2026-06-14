# Changelog

## v1.1.9 (2026-06-14)

### 优化

- 优化首页排版，突出第三方 API 配置教程，并提示第三方 API 用户优先安装 cc-switch。
- 快速入口改为 Claude 桌面端官网、Claude 桌面端网盘下载和 cc-switch 网盘下载。
- 配置夸克网盘链接用于应用更新下载和快速入口下载。

## v1.1.8 (2026-06-11)

### 修复

- 应用更新检测改为只展示配置的网盘下载链接；未配置链接时提示联系商家，不再回退到 GitHub 下载入口。

## v1.1.7 (2026-06-11)

### 修复

- 调整应用启动更新检查逻辑，先检测 GitHub Release 版本，再尝试 Tauri 自动安装清单，避免缺失 `latest.json` 时完全无法提示更新。
- 发布流程缺少 updater 签名产物时不再阻断普通 release，保证应用至少能自动检查到新版本并提示下载。

## v1.1.6 (2026-06-11)

### 修复

- 修复公开发布流程未上传 updater 签名产物和 `latest.json`，导致应用内自动更新器无法检查和安装更新的问题。

## v1.1.5 (2026-06-11)

### 修复

- 补齐新版 Claude Cowork 菜单项 `Free Up Cowork Disk Space…` 的中文翻译，避免 Windows 安装补丁后触发 `MISSING_TRANSLATION`。

## v1.1.4 (2026-06-11)

### 修复

- 修复 Windows 打包版安装中文补丁时找不到 `resources` 或 `scripts/install_windows.ps1` 的问题。
- 更换应用图标，并补充 macOS `icns` 图标资源。

## v1.0.0 (2026-06-06)

### ✨ 初始版本

- 🌐 一键汉化：支持简体中文、繁体中文（台湾）、繁体中文（香港）
- 🔄 汉化包更新检测：自动对比 GitHub Releases 最新版本
- 🔧 ccSwitch 安装配置教程：内置图文教程和推荐配置
- 📖 第三方 API 配置教程：Anthropic、OpenRouter、中转服务完整指南
- 💻 系统状态面板：实时显示 Claude 安装状态、汉化状态
- 🍎 macOS 支持：安全模式（Cowork 兼容）、完整模式
- 🪟 Windows 支持：三种安装模式、自动备份恢复
- 📝 一键打开 config.json：快速编辑 API 配置
- 🎨 简洁美观的工具面板界面
