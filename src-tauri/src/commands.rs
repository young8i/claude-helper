use crate::system_info;
use crate::localizer::{self, LocalizeOptions, LocalizeResult};
use crate::updater::{self, AppUpdateInfo};
use crate::ccswitch;

// ── System & Localization ──────────────────────────────────

#[tauri::command]
pub async fn get_system_info() -> Result<system_info::SystemInfo, String> {
    Ok(system_info::get_full_info())
}

#[tauri::command]
pub async fn check_zh_cn_status() -> Result<bool, String> {
    match system_info::find_claude_path() {
        Some(path) => Ok(system_info::check_zh_cn_from_claude(&path)),
        None => Ok(false),
    }
}

#[tauri::command]
pub async fn install_localization(options: LocalizeOptions) -> Result<LocalizeResult, String> {
    if !localizer::validate_lang_code(&options.lang_code) {
        return Err(format!("不支持的语言代码: {}", options.lang_code));
    }
    if !localizer::validate_mode(&options.mode) {
        return Err(format!("不支持的安装模式: {}", options.mode));
    }
    #[cfg(target_os = "macos")]
    { Ok(localizer::run_install_macos(&options.lang_code, &options.mode)) }
    #[cfg(target_os = "windows")]
    { Ok(localizer::run_install_windows(&options.lang_code, &options.mode)) }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { Err("当前操作系统不受支持".to_string()) }
}

#[tauri::command]
pub async fn uninstall_localization() -> Result<LocalizeResult, String> {
    #[cfg(target_os = "macos")]
    { Ok(localizer::run_uninstall_macos()) }
    #[cfg(target_os = "windows")]
    { Ok(localizer::run_uninstall_windows()) }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { Err("当前操作系统不受支持".to_string()) }
}

// ── Updates ─────────────────────────────────────────────────

#[tauri::command]
pub async fn check_for_updates() -> Result<AppUpdateInfo, String> {
    updater::check_app_update().await
}

#[tauri::command]
pub async fn get_versions() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "app": updater::get_app_version() }))
}

// ── cc-switch ──────────────────────────────────────────────

#[tauri::command]
pub async fn check_ccswitch_status() -> Result<ccswitch::CcSwitchStatus, String> {
    Ok(ccswitch::check_status())
}

#[tauri::command]
pub async fn install_ccswitch() -> Result<String, String> {
    ccswitch::run_one_click_install()
}

#[tauri::command]
pub async fn get_ccswitch_guide() -> Result<String, String> {
    Ok(ccswitch::get_config_guide())
}

// ── API Guide (built-in) ──────────────────────────────────

#[tauri::command]
pub async fn get_api_guide() -> Result<String, String> {
    Ok(build_api_guide())
}

// ── URL / file openers ─────────────────────────────────────

#[tauri::command]
pub async fn open_url_in_browser(url: String) -> Result<(), String> {
    open_url(&url)
}

#[tauri::command]
pub async fn open_ccswitch_site() -> Result<(), String> {
    open_url("https://ccswitch.io")
}

#[tauri::command]
pub async fn open_ccswitch_releases() -> Result<(), String> {
    open_url("https://github.com/farion1231/cc-switch/releases")
}

#[tauri::command]
pub async fn open_config_file() -> Result<String, String> {
    let config_path = if cfg!(target_os = "macos") {
        format!("{}/Library/Application Support/Claude/config.json",
            dirs::home_dir().map(|h| h.to_string_lossy().to_string()).unwrap_or_default())
    } else {
        format!("{}\\Claude\\config.json",
            dirs::config_dir().map(|d| d.to_string_lossy().to_string()).unwrap_or_default())
    };
    #[cfg(target_os = "macos")]
    { std::process::Command::new("open").arg("-t").arg(&config_path).spawn()
        .map_err(|e| format!("{}", e))?; }
    #[cfg(target_os = "windows")]
    { std::process::Command::new("cmd").args(["/c", "start", "", &config_path]).spawn()
        .map_err(|e| format!("{}", e))?; }
    Ok(format!("已打开: {}", config_path))
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    { std::process::Command::new("open").arg(url).spawn().map_err(|e| format!("{}", e))?; }
    #[cfg(target_os = "windows")]
    { std::process::Command::new("cmd").args(["/c", "start", "", url]).spawn()
        .map_err(|e| format!("{}", e))?; }
    Ok(())
}

fn build_api_guide() -> String {
    r##"# Claude Desktop 第三方 API 配置教程

---

## 第一步：启用开发者模式

首次打开 Claude Desktop 时**不要登录**，按以下步骤操作：

1. **重要技巧**：未登录状态下菜单可能点不动——先鼠标点一下邮箱输入框，按 `Tab` 键跳到左上角菜单按钮，按回车打开
2. 菜单路径：**Help → Troubleshooting → Enable Developer Mode**
3. 点击 **Enable** 确认
4. 重启 Claude Desktop 生效

---

## 第二步：配置第三方推理

重启后左上角会出现 **Developer** 菜单：

菜单路径：**Developer → Configure Third-Party Inference**

配置字段说明：

| 字段 | 值 | 说明 |
|------|-----|------|
| Connection | Gateway | 网关模式 |
| Gateway Base URL | `https://api.deepseek.com/anthropic` | 兼容 Anthropic Messages 格式 |
| Gateway API Key | `sk-xxxxxxxx` | 你的 API 密钥 |
| Auth Scheme | x-api-key | 认证方式 |
| Model List | `deepseek-chat` | 每行一个模型名 |

填写后点击 **Apply Locally** 保存，选择 **Local** 进入，即可使用自定义 API。

---

## 兼容 Anthropic 格式的 API 端点

| 服务商 | Anthropic 兼容地址 | 模型 |
|--------|-------------------|------|
| Anthropic 官方 | `https://api.anthropic.com` | claude-sonnet-4-20250514 |
| DeepSeek | `https://api.deepseek.com/anthropic` | deepseek-chat / deepseek-reasoner |
| 阿里云 DashScope | `https://dashscope-intl.aliyuncs.com/apps/anthropic` | qwen3-plus |
| Moonshot (Kimi) | `https://api.moonshot.ai/anthropic` | kimi-k2 |
| 智谱 AI | `https://open.bigmodel.cn/api/anthropic` | glm-4-plus |
| MiniMax | `https://api.minimax.io/anthropic` | minimax-m2 |

> ⚠️ OpenRouter 等 OpenAI 格式需通过 cc-switch 做格式转换。

---

## 常见问题

### 模型列表为空
新版 Claude Desktop 加固了模型名白名单。用 **cc-switch** 将第三方模型别名映射为 Claude 官方名（如 `deepseek-chat` → `claude-sonnet-4-20250514`）。

### HTTP 地址不生效
某些版本仅接受 HTTPS。用 cc-switch 在 `127.0.0.1:5000` 做本地代理。

### 接口灰掉显示 "Organization-managed"
- **Windows**：删注册表 `HKCU\SOFTWARE\Policies\Claude`
- **macOS**：检查 `~/Library/Preferences/com.anthropic.claude.plist`

### Cowork 沙箱不可用
用中文补丁 **模式 1（安全模式）** + cc-switch 代理映射。

---

## 推荐方案

| 你的情况 | API 方案 | 补丁模式 | cc-switch |
|----------|---------|---------|-----------|
| 官方订阅 + 中文 | Anthropic 官方 | 模式 2 | 不需要 |
| 第三方 API + Cowork | OpenRouter/中转 | 模式 1 | 需要 |
| 多模型 + 所有功能 | 任意 API | 模式 3 | 推荐 |

---

> 🔧 **cc-switch**：https://github.com/farion1231/cc-switch
"##.to_string()
}
