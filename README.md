# Claude 中文助手

一个基于 [Tauri](https://tauri.app) 构建的 Claude Desktop 桌面工具，提供：

- 🌐 **一键汉化** — 自动安装 Claude Desktop 中文语言包
- 🔄 **更新检测** — 自动检测汉化包最新版本
- 🔧 **ccSwitch 配置** — 一键安装和配置 ccSwitch 模型路由工具
- 📖 **API 引导** — 完整的第三方 API 配置教程

支持 **macOS** 和 **Windows**。

## 前置条件

### macOS
- Xcode Command Line Tools: `xcode-select --install`
- Node.js 18+: `brew install node`
- Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Claude Desktop 已安装

### Windows
- Microsoft Visual Studio C++ Build Tools
- Node.js 18+: `winget install OpenJS.NodeJS.LTS`
- Rust: `winget install Rustlang.Rustup`
- Claude Desktop 已安装

## 快速开始 (开发)

```bash
# 1. 安装依赖
npm install

# 2. 复制汉化资源文件
bash scripts/copy-resources.sh

# 3. 启动开发模式 (热重载)
npm run tauri:dev

# 4. 构建生产版本
npm run tauri:build
```

## 项目结构

```
desktop-helper/
├── src/                    # 前端 (HTML/CSS/JS)
│   ├── index.html          # 主界面
│   ├── styles.css          # 样式
│   └── app.js              # 应用逻辑
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── main.rs         # 入口
│   │   ├── commands.rs     # Tauri IPC 命令
│   │   ├── localizer.rs    # 汉化逻辑
│   │   ├── updater.rs      # 更新检测
│   │   ├── ccswitch.rs     # ccSwitch 集成
│   │   └── system_info.rs  # 系统信息
│   ├── Cargo.toml
│   └── tauri.conf.json     # Tauri 配置
├── scripts/                # 构建辅助脚本
├── resources/              # 汉化资源 (软链到主项目)
└── package.json
```

## 构建产物

构建完成后，产物位于：

- **macOS**: `src-tauri/target/release/bundle/dmg/Claude中文助手_*.dmg`
- **Windows**: `src-tauri/target/release/bundle/msi/Claude中文助手_*.msi`

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面壳层 | Tauri 2 (Rust) |
| 前端界面 | Vanilla HTML/CSS/JS + Vite |
| 系统调用 | Rust std::process |
| 更新检测 | GitHub Releases API |
| 打包 | Tauri Bundler (DMG/MSI/NSIS) |

## 闲鱼发布指南

见 [XIANYU_GUIDE.md](./XIANYU_GUIDE.md)

## 许可证

MIT License — 仅供学习交流使用。请勿用于商业盈利目的。
