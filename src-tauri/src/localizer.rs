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
    let mut steps: Vec<String> = Vec::new();

    // Map mode to script parameter
    let (patch_mode, action) = match mode {
        "safe" => ("safe", "install"),
        "official" => ("official", "install"),
        "full" => ("full", "install"),
        _ => ("safe", "install"),
    };

    steps.push("检查系统权限...".to_string());
    steps.push("准备 PowerShell 安装环境...".to_string());
    steps.push(format!("执行安装脚本 (语言: {}, 模式: {})...", lang_code, mode));

    match run_elevated_windows_installer(action, lang_code, Some(patch_mode)) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                steps.push("PowerShell 安装脚本执行完成！".to_string());
                LocalizeResult {
                    success: true,
                    message: format!("✅ 中文补丁安装成功！语言: {}, 模式: {}\n\n已通过 PowerShell 执行安装脚本，请重启 Claude Desktop 生效。", lang_code, mode),
                    steps,
                }
            } else {
                steps.push(format!("安装细节:\n{}", stdout));
                if !stderr.is_empty() {
                    steps.push(format!("错误信息: {}", stderr));
                }
                LocalizeResult {
                    success: false,
                    message: "❌ 安装失败，请确认已允许 UAC 授权，并查看临时目录中的 install-windows.log".to_string(),
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
    let mut steps: Vec<String> = Vec::new();

    steps.push("准备恢复原始版本...".to_string());
    steps.push("准备 PowerShell 卸载环境...".to_string());

    match run_elevated_windows_installer("uninstall", "zh-CN", None) {
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

#[cfg(target_os = "windows")]
fn run_elevated_windows_installer(
    action: &str,
    lang_code: &str,
    patch_mode: Option<&str>,
) -> Result<std::process::Output, String> {
    let project_root = find_project_root()
        .ok_or_else(|| "未找到打包资源目录：缺少 resources 或 scripts/install_windows.ps1".to_string())?;
    let staged_root = stage_windows_installer(&project_root)?;
    let script_path = staged_root.join("scripts").join("install_windows.ps1");

    if !script_path.exists() {
        return Err(format!("未找到 Windows 安装脚本: {}", script_path.display()));
    }

    let mut child_args = vec![
        "-NoLogo".to_string(),
        "-NoProfile".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-File".to_string(),
        script_path.to_string_lossy().to_string(),
        "-Action".to_string(),
        action.to_string(),
        "-Language".to_string(),
        lang_code.to_string(),
    ];

    if let Some(mode) = patch_mode {
        child_args.push("-PatchMode".to_string());
        child_args.push(mode.to_string());
    }

    let command = format!(
        "$env:CLAUDE_ZH_SKIP_UPDATE_CHECK='1'; \
         $p = Start-Process -FilePath {} -ArgumentList {} -WorkingDirectory {} -Verb RunAs -Wait -PassThru -ErrorAction Stop; \
         if ($null -ne $p.ExitCode) {{ exit $p.ExitCode }}; exit 0",
        ps_single_quote("powershell.exe"),
        ps_single_quote(&windows_command_line(&child_args)),
        ps_single_quote(&staged_root.to_string_lossy()),
    );

    std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &command,
        ])
        .output()
        .map_err(|e| format!("无法启动 PowerShell: {}", e))
}

#[cfg(target_os = "windows")]
fn stage_windows_installer(project_root: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let staged_root = std::env::temp_dir().join("ClaudeDesktopZhCnInstaller");
    let staged_scripts = staged_root.join("scripts");
    let staged_resources = staged_root.join("resources");

    if staged_root.exists() {
        std::fs::remove_dir_all(&staged_root)
            .map_err(|e| format!("无法清理临时安装目录 {}: {}", staged_root.display(), e))?;
    }

    std::fs::create_dir_all(&staged_scripts)
        .map_err(|e| format!("无法创建临时 scripts 目录: {}", e))?;
    std::fs::create_dir_all(&staged_resources)
        .map_err(|e| format!("无法创建临时 resources 目录: {}", e))?;

    let script_src = project_root.join("scripts").join("install_windows.ps1");
    std::fs::copy(&script_src, staged_scripts.join("install_windows.ps1"))
        .map_err(|e| format!("无法复制 Windows 安装脚本 {}: {}", script_src.display(), e))?;

    copy_dir_contents(&project_root.join("resources"), &staged_resources)?;

    Ok(staged_root)
}

#[cfg(target_os = "windows")]
fn copy_dir_contents(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    if !src.exists() {
        return Err(format!("资源目录不存在: {}", src.display()));
    }

    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("无法读取资源目录 {}: {}", src.display(), e))?
    {
        let entry = entry.map_err(|e| format!("无法读取资源目录项: {}", e))?;
        let source_path = entry.path();
        let target_path = dst.join(entry.file_name());

        if source_path.is_dir() {
            std::fs::create_dir_all(&target_path)
                .map_err(|e| format!("无法创建目录 {}: {}", target_path.display(), e))?;
            copy_dir_contents(&source_path, &target_path)?;
        } else {
            std::fs::copy(&source_path, &target_path)
                .map_err(|e| format!("无法复制资源文件 {}: {}", source_path.display(), e))?;
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn windows_command_line(values: &[String]) -> String {
    values
        .iter()
        .map(|v| windows_command_arg(v))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(target_os = "windows")]
fn windows_command_arg(value: &str) -> String {
    let escaped = value.replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

#[cfg(target_os = "windows")]
fn ps_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn find_project_root() -> Option<std::path::PathBuf> {
    project_root_candidates()
        .into_iter()
        .find(|path| is_project_root(path))
}

fn project_root_candidates() -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::new();

    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest.clone());
    if let Some(parent) = manifest.parent() {
        candidates.push(parent.to_path_buf());
        if let Some(grandparent) = parent.parent() {
            candidates.push(grandparent.to_path_buf());
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.to_path_buf());
            candidates.push(parent.join("resources"));
            if let Some(grandparent) = parent.parent() {
                candidates.push(grandparent.to_path_buf());
            }
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.clone());
        candidates.push(current_dir.join("resources"));
    }

    candidates
}

fn is_project_root(path: &std::path::Path) -> bool {
    path.join("resources").join("frontend-zh-CN.json").exists()
        && path.join("scripts").join("install_windows.ps1").exists()
}

fn find_macos_installer_root() -> Option<std::path::PathBuf> {
    project_root_candidates()
        .into_iter()
        .find(|path| {
            path.join("install-mac.command").exists()
                && path.join("scripts").join("patch_claude_zh_cn.py").exists()
                && path.join("resources").join("release.json").exists()
        })
}

/// Get the resources directory (where install scripts and translation files live)
pub fn get_resources_dir() -> String {
    find_macos_installer_root()
        .or_else(find_project_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string())
}
