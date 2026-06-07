use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalizeOptions {
    pub lang_code: String,
    pub mode: String, // "safe", "full", "official"
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalizeResult {
    pub success: bool,
    pub message: String,
    pub steps: Vec<String>,
}

/// Validate language code
pub fn validate_lang_code(code: &str) -> bool {
    matches!(code, "zh-CN" | "zh-TW" | "zh-HK")
}

/// Validate install mode
pub fn validate_mode(mode: &str) -> bool {
    matches!(mode, "safe" | "full" | "official")
}

/// Run the localization install (macOS)
#[cfg(target_os = "macos")]
pub fn run_install_macos(lang_code: &str, mode: &str) -> LocalizeResult {
    use std::process::Command;

    let mut steps: Vec<String> = Vec::new();
    let resources_dir = get_resources_dir();

    // Step 1: Check if running with privileges
    steps.push("检查系统权限...".to_string());

    // We'll use osascript to request admin privileges and run the command script
    let skip_asar = if mode == "safe" { "1" } else { "0" };

    let script = format!(
        r#"do shell script "cd '{}' && CLAUDE_ACTION='install' CLAUDE_LANG='{}' CLAUDE_SKIP_ASAR_PATCH='{}' CLAUDE_SKIP_UPDATE_CHECK='1' bash install-mac.command < /dev/null" with administrator privileges"#,
        resources_dir, lang_code, skip_asar
    );

    steps.push(format!("执行安装脚本 (语言: {}, 模式: {})...", lang_code, mode));

    match Command::new("osascript")
        .args(["-e", &script])
        .output()
    {
        Ok(output) => {
            let combined = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            let succeeded = output.status.success() || combined.contains("Done.") || combined.contains("Backup kept at");

            if succeeded {
                steps.push("安装完成！请重启 Claude Desktop 生效。".to_string());
                LocalizeResult {
                    success: true,
                    message: format!("✅ 中文补丁安装成功！语言: {}, 模式: {}\n\n补丁已部署到 /Applications/Claude.app，备份保存在 Applications 文件夹中。", lang_code, mode),
                    steps,
                }
            } else {
                steps.push(format!("安装遇到问题: {}", combined));
                LocalizeResult {
                    success: false,
                    message: format!("❌ 安装失败: {}", combined),
                    steps,
                }
            }
        }
        Err(e) => {
            steps.push(format!("执行脚本失败: {}", e));
            LocalizeResult {
                success: false,
                message: format!("❌ 无法执行安装脚本: {}", e),
                steps,
            }
        }
    }
}

/// Run the localization install (Windows)
#[cfg(target_os = "windows")]
pub fn run_install_windows(lang_code: &str, mode: &str) -> LocalizeResult {
    use std::process::Command;

    let mut steps: Vec<String> = Vec::new();
    let resources_dir = get_resources_dir();

    // Map mode to script parameter
    let (patch_mode, action) = match mode {
        "safe" => ("safe", "install"),
        "official" => ("official", "install"),
        "full" => ("full", "install"),
        _ => ("safe", "install"),
    };

    steps.push("检查系统权限...".to_string());
    steps.push(format!("执行安装脚本 (语言: {}, 模式: {})...", lang_code, mode));

    let script_path = std::path::PathBuf::from(&resources_dir)
        .join("scripts")
        .join("install_windows.ps1");

    match Command::new("powershell")
        .args([
            "-ExecutionPolicy", "Bypass",
            "-File", script_path.to_str().unwrap_or("install_windows.ps1"),
            "-Action", action,
            "-Language", lang_code,
            "-PatchMode", patch_mode,
        ])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                steps.push("安装完成！".to_string());
                LocalizeResult {
                    success: true,
                    message: format!("✅ 中文补丁安装成功！语言: {}, 模式: {}", lang_code, mode),
                    steps,
                }
            } else {
                steps.push(format!("安装细节:\n{}", stdout));
                if !stderr.is_empty() {
                    steps.push(format!("错误信息: {}", stderr));
                }
                LocalizeResult {
                    success: false,
                    message: format!("❌ 安装失败，请以管理员身份运行本程序后重试"),
                    steps,
                }
            }
        }
        Err(e) => {
            steps.push(format!("执行脚本失败: {}", e));
            LocalizeResult {
                success: false,
                message: format!("❌ 无法执行安装脚本: {}", e),
                steps,
            }
        }
    }
}

/// Run the uninstall/restore
#[cfg(target_os = "macos")]
pub fn run_uninstall_macos() -> LocalizeResult {
    use std::process::Command;

    let mut steps: Vec<String> = Vec::new();
    let resources_dir = get_resources_dir();

    steps.push("准备恢复原始版本...".to_string());

    let script = format!(
        r#"do shell script "cd '{}' && CLAUDE_ACTION='restore' bash install-mac.command < /dev/null" with administrator privileges"#,
        resources_dir
    );

    match Command::new("osascript")
        .args(["-e", &script])
        .output()
    {
        Ok(output) => {
            let combined = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            let succeeded = output.status.success() || combined.contains("Done.") || combined.contains("Restored from backup");

            if succeeded {
                steps.push("恢复完成！".to_string());
                LocalizeResult {
                    success: true,
                    message: "✅ 已恢复为原始英文版本".to_string(),
                    steps,
                }
            } else {
                LocalizeResult {
                    success: false,
                    message: format!("❌ 恢复失败: {}", combined),
                    steps,
                }
            }
        }
        Err(e) => {
            LocalizeResult {
                success: false,
                message: format!("❌ 无法执行恢复脚本: {}", e),
                steps,
            }
        }
    }
}

/// Run the uninstall/restore (Windows)
#[cfg(target_os = "windows")]
pub fn run_uninstall_windows() -> LocalizeResult {
    use std::process::Command;

    let mut steps: Vec<String> = Vec::new();
    let resources_dir = get_resources_dir();

    steps.push("准备恢复原始版本...".to_string());

    let script_path = std::path::PathBuf::from(&resources_dir)
        .join("scripts")
        .join("install_windows.ps1");

    match Command::new("powershell")
        .args([
            "-ExecutionPolicy", "Bypass",
            "-File", script_path.to_str().unwrap_or("install_windows.ps1"),
            "-Action", "uninstall",
            "-Language", "zh-CN",
        ])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                steps.push("恢复完成！".to_string());
                LocalizeResult {
                    success: true,
                    message: "✅ 已恢复为原始英文版本".to_string(),
                    steps,
                }
            } else {
                LocalizeResult {
                    success: false,
                    message: "❌ 恢复失败，请以管理员身份运行后重试".to_string(),
                    steps,
                }
            }
        }
        Err(e) => {
            LocalizeResult {
                success: false,
                message: format!("❌ 无法执行恢复脚本: {}", e),
                steps,
            }
        }
    }
}

/// Get the resources directory (where install scripts and translation files live)
pub fn get_resources_dir() -> String {
    #[cfg(debug_assertions)]
    {
        // Dev mode: CARGO_MANIFEST_DIR is desktop-helper/src-tauri
        // Go up 2 levels to desktop-helper, then 1 more to claude-desktop-zh-cn
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let main_project = manifest
            .parent()   // desktop-helper/src-tauri -> desktop-helper
            .and_then(|p| p.parent())  // desktop-helper -> claude-desktop-zh-cn
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        main_project
    }
    #[cfg(not(debug_assertions))]
    {
        // Production: resources bundled alongside the binary
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| ".".to_string())
    }
}
