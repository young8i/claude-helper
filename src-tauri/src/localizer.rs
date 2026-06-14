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
pub fn run_install_macos(
    lang_code: &str,
    mode: &str,
    resource_hint: Option<&std::path::Path>,
) -> LocalizeResult {
    let mut steps: Vec<String> = Vec::new();

    // Step 1: Check if running with privileges
    steps.push("检查系统权限...".to_string());

    let skip_asar = if mode == "safe" { "1" } else { "0" };

    steps.push(format!("执行安装脚本 (语言: {}, 模式: {})...", lang_code, mode));

    match run_elevated_macos_installer(
        "install",
        Some(lang_code),
        Some(skip_asar),
        resource_hint,
    ) {
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
pub fn run_install_windows(
    lang_code: &str,
    mode: &str,
    resource_hint: Option<&std::path::Path>,
) -> LocalizeResult {
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

    match run_elevated_windows_installer(action, lang_code, Some(patch_mode), resource_hint) {
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
pub fn run_uninstall_macos(resource_hint: Option<&std::path::Path>) -> LocalizeResult {
    let mut steps: Vec<String> = Vec::new();

    steps.push("准备恢复原始版本...".to_string());

    match run_elevated_macos_installer("restore", None, None, resource_hint) {
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
pub fn run_uninstall_windows(resource_hint: Option<&std::path::Path>) -> LocalizeResult {
    let mut steps: Vec<String> = Vec::new();

    steps.push("准备恢复原始版本...".to_string());
    steps.push("准备 PowerShell 卸载环境...".to_string());

    match run_elevated_windows_installer("uninstall", "zh-CN", None, resource_hint) {
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

#[cfg(target_os = "macos")]
struct MacInstallerSource {
    installer: std::path::PathBuf,
    scripts: std::path::PathBuf,
    resources: std::path::PathBuf,
}

#[cfg(target_os = "macos")]
fn run_elevated_macos_installer(
    action: &str,
    lang_code: Option<&str>,
    skip_asar: Option<&str>,
    resource_hint: Option<&std::path::Path>,
) -> Result<std::process::Output, String> {
    let source = find_macos_installer_source(resource_hint).ok_or_else(|| {
        let checked = project_root_candidates(resource_hint)
            .iter()
            .map(|p| format!("  - {}", p.display()))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "未找到 macOS 打包资源目录：缺少 install-mac.command、resources/release.json 或 scripts/patch_claude_zh_cn.py\n已检查路径：\n{}",
            checked,
        )
    })?;
    let staged_root = stage_macos_installer(&source)?;

    let mut shell_command = format!(
        "cd {} && CLAUDE_ACTION={} CLAUDE_ZH_SKIP_UPDATE_CHECK='1'",
        sh_quote(&staged_root.to_string_lossy()),
        sh_quote(action),
    );

    if let Some(lang) = lang_code {
        shell_command.push_str(&format!(" CLAUDE_LANG={}", sh_quote(lang)));
    }
    if let Some(skip) = skip_asar {
        shell_command.push_str(&format!(" CLAUDE_SKIP_ASAR_PATCH={}", sh_quote(skip)));
    }
    shell_command.push_str(" bash ./install-mac.command < /dev/null");

    let script = format!(
        "do shell script {} with administrator privileges",
        applescript_quote(&shell_command),
    );

    std::process::Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("无法启动 osascript: {}", e))
}

#[cfg(target_os = "macos")]
fn find_macos_installer_source(
    resource_hint: Option<&std::path::Path>,
) -> Option<MacInstallerSource> {
    for root in project_root_candidates(resource_hint) {
        let layouts = [
            (
                root.join("install-mac.command"),
                root.join("scripts"),
                root.join("resources"),
            ),
            (
                root.join("resources").join("install-mac.command"),
                root.join("scripts"),
                root.join("resources"),
            ),
        ];

        for (installer, scripts, resources) in layouts {
            if is_macos_installer_layout(&installer, &scripts, &resources) {
                return Some(MacInstallerSource {
                    installer,
                    scripts,
                    resources,
                });
            }
        }

        if let Some(parent) = root.parent() {
            let installer = root.join("install-mac.command");
            let scripts = parent.join("scripts");
            let resources = root.clone();
            if is_macos_installer_layout(&installer, &scripts, &resources) {
                return Some(MacInstallerSource {
                    installer,
                    scripts,
                    resources,
                });
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn is_macos_installer_layout(
    installer: &std::path::Path,
    scripts: &std::path::Path,
    resources: &std::path::Path,
) -> bool {
    installer.exists()
        && scripts.join("patch_claude_zh_cn.py").exists()
        && resources.join("release.json").exists()
}

#[cfg(target_os = "macos")]
fn stage_macos_installer(source: &MacInstallerSource) -> Result<std::path::PathBuf, String> {
    let staged_root = std::env::temp_dir().join("ClaudeZhHelperMacInstaller");
    let staged_scripts = staged_root.join("scripts");
    let staged_resources = staged_root.join("resources");

    if staged_root.exists() {
        std::fs::remove_dir_all(&staged_root)
            .map_err(|e| format!("无法清理 macOS 临时安装目录 {}: {}", staged_root.display(), e))?;
    }

    std::fs::create_dir_all(&staged_scripts)
        .map_err(|e| format!("无法创建 macOS 临时 scripts 目录: {}", e))?;
    std::fs::create_dir_all(&staged_resources)
        .map_err(|e| format!("无法创建 macOS 临时 resources 目录: {}", e))?;

    std::fs::copy(&source.installer, staged_root.join("install-mac.command")).map_err(|e| {
        format!(
            "无法复制 macOS 安装脚本 {}: {}",
            source.installer.display(),
            e,
        )
    })?;
    copy_dir_contents(&source.scripts, &staged_scripts)?;
    copy_dir_contents(&source.resources, &staged_resources)?;

    Ok(staged_root)
}

#[cfg(target_os = "macos")]
fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
fn applescript_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(target_os = "windows")]
fn run_elevated_windows_installer(
    action: &str,
    lang_code: &str,
    patch_mode: Option<&str>,
    resource_hint: Option<&std::path::Path>,
) -> Result<std::process::Output, String> {
    let candidates = project_root_candidates(resource_hint);
    let project_root = candidates
        .iter()
        .find(|path| is_project_root(path))
        .cloned()
        .ok_or_else(|| {
            let checked = candidates
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "未找到打包资源目录：缺少 resources 或 scripts/install_windows.ps1\n已检查路径：\n{}",
                checked,
            )
        })?;
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
    let staged_root = std::env::temp_dir().join("ClaudeZhHelperInstaller");
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
    project_root_candidates(None)
        .into_iter()
        .find(|path| is_project_root(path))
}

fn project_root_candidates(resource_hint: Option<&std::path::Path>) -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::new();

    if let Some(resource_dir) = resource_hint {
        push_path_variants(&mut candidates, resource_dir);
    }

    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    push_path_variants(&mut candidates, &manifest);

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            push_path_variants(&mut candidates, parent);
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        push_path_variants(&mut candidates, &current_dir);
    }

    candidates
}

fn push_path_variants(candidates: &mut Vec<std::path::PathBuf>, path: &std::path::Path) {
    push_candidate(candidates, path.to_path_buf());
    push_candidate(candidates, path.join("_up_"));
    push_candidate(candidates, path.join("resources"));
    push_candidate(candidates, path.join("resources").join("_up_"));

    if let Some(parent) = path.parent() {
        push_candidate(candidates, parent.to_path_buf());
        push_candidate(candidates, parent.join("_up_"));
        push_candidate(candidates, parent.join("resources"));
        push_candidate(candidates, parent.join("resources").join("_up_"));

        if let Some(grandparent) = parent.parent() {
            push_candidate(candidates, grandparent.to_path_buf());
            push_candidate(candidates, grandparent.join("_up_"));
            push_candidate(candidates, grandparent.join("resources"));
            push_candidate(candidates, grandparent.join("resources").join("_up_"));
        }
    }
}

fn push_candidate(candidates: &mut Vec<std::path::PathBuf>, path: std::path::PathBuf) {
    if !candidates.iter().any(|candidate| candidate == &path) {
        candidates.push(path);
    }
}

fn is_project_root(path: &std::path::Path) -> bool {
    path.join("resources").join("frontend-zh-CN.json").exists()
        && path.join("scripts").join("install_windows.ps1").exists()
}
