use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: String,
    pub os_version: String,
    pub claude_installed: bool,
    pub claude_path: Option<String>,
    pub claude_version: Option<String>,
    pub zh_cn_installed: bool,
    pub zh_cn_version: Option<String>,
    pub app_data_path: Option<String>,
}

/// Detect the operating system
pub fn detect_os() -> String {
    if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Get OS version string
pub fn get_os_version() -> String {
    if cfg!(target_os = "macos") {
        // Try to get macOS version via sw_vers
        if let Ok(output) = std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
        {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            "macOS".to_string()
        }
    } else if cfg!(target_os = "windows") {
        // Windows version
        if let Ok(output) = crate::process::command("cmd")
            .args(["/c", "ver"])
            .output()
        {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            "Windows".to_string()
        }
    } else {
        "Unknown".to_string()
    }
}

/// Find the Claude Desktop installation path
pub fn find_claude_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let app_path = PathBuf::from("/Applications/Claude.app");
        if app_path.exists() {
            return Some(app_path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        return find_windows_claude_path();
    }

    None
}

#[cfg(target_os = "windows")]
fn find_windows_claude_path() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    candidates.extend(find_unpacked_windows_claude_paths());
    candidates.extend(find_appx_windows_claude_paths());

    if let Some(local_app_data) = dirs::data_local_dir() {
        for sub in [
            "Programs\\Claude",
            "Programs\\Claude Desktop",
            "Programs\\AnthropicClaude",
        ] {
            candidates.push(local_app_data.join(sub));
        }
    }

    for env_name in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Ok(base) = std::env::var(env_name) {
            for sub in ["Claude", "Claude Desktop", "AnthropicClaude"] {
                candidates.push(PathBuf::from(&base).join(sub));
            }
        }
    }

    candidates.into_iter().find(|p| get_windows_claude_resources_path(p).is_some())
}

#[cfg(target_os = "windows")]
fn find_unpacked_windows_claude_paths() -> Vec<PathBuf> {
    let Some(local_app_data) = dirs::data_local_dir() else {
        return Vec::new();
    };

    let anthropic_base = local_app_data.join("AnthropicClaude");
    let Ok(entries) = std::fs::read_dir(&anthropic_base) else {
        return Vec::new();
    };

    let mut app_dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("app-"))
        .map(|e| e.path())
        .collect();

    app_dirs.sort_by(|a, b| modified_time(b).cmp(&modified_time(a)));
    app_dirs
}

#[cfg(target_os = "windows")]
fn find_appx_windows_claude_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let command = r#"
$packages = @()
$packages += @(Get-AppxPackage -Name "Claude" -ErrorAction SilentlyContinue)
$packages += @(Get-AppxPackage -Name "*Claude*" -ErrorAction SilentlyContinue |
  Where-Object { $_.Name -match "Claude" -or $_.PackageFullName -match "Claude|Anthropic" })
$packages = @($packages | Where-Object { $_ -and $_.PackageFullName } | Sort-Object PackageFullName -Unique)
foreach ($package in ($packages | Sort-Object InstallDate -Descending)) {
  if ($package.InstallLocation -and (Test-Path -LiteralPath $package.InstallLocation)) {
    $package.InstallLocation
  }
}
$fallback = @(Get-ChildItem "C:\Program Files\WindowsApps\*Claude*" -Directory -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending)
foreach ($dir in $fallback) { $dir.FullName }
"#;

    if let Ok(output) = crate::process::command("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            command,
        ])
        .output()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                paths.push(PathBuf::from(trimmed));
            }
        }
    }

    paths
}

#[cfg(target_os = "windows")]
fn modified_time(path: &Path) -> std::time::SystemTime {
    path.metadata()
        .and_then(|m| m.modified())
        .unwrap_or(std::time::UNIX_EPOCH)
}

#[cfg(target_os = "windows")]
fn get_windows_claude_resources_path(claude_path: &Path) -> Option<PathBuf> {
    let root = if claude_path.is_file() {
        claude_path.parent()?
    } else {
        claude_path
    };

    for candidate in [
        root.join("resources"),
        root.join("app").join("resources"),
    ] {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn get_windows_claude_exe_path(claude_path: &Path) -> Option<PathBuf> {
    let root = if claude_path.is_file() {
        claude_path.parent()?
    } else {
        claude_path
    };

    for candidate in [
        root.join("Claude.exe"),
        root.join("claude.exe"),
        root.join("app").join("Claude.exe"),
        root.join("app").join("claude.exe"),
    ] {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

#[cfg(target_os = "windows")]
pub fn launch_claude_desktop() -> Result<(), String> {
    let claude_path = find_claude_path().ok_or_else(|| "未找到 Claude Desktop 安装".to_string())?;
    let exe_path = get_windows_claude_exe_path(&claude_path).ok_or_else(|| {
        format!("未找到 Claude.exe: {}", claude_path.display())
    })?;

    crate::process::command("cmd.exe")
        .args(["/D", "/C", "start", ""])
        .arg(&exe_path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("无法启动 Claude Desktop: {}", e))
}

/// Get Claude Desktop version from Info.plist (macOS) or executable metadata (Windows)
pub fn get_claude_version(claude_path: &PathBuf) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let plist_path = claude_path.join("Contents").join("Info.plist");
        if plist_path.exists() {
            // Use plutil to read version
            if let Ok(output) = std::process::Command::new("plutil")
                .args([
                    "-extract", "CFBundleShortVersionString", "raw",
                    "-o", "-",
                    plist_path.to_str()?,
                ])
                .output()
            {
                return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        for manifest in [
            claude_path.join("AppxManifest.xml"),
            claude_path.join("app").join("AppxManifest.xml"),
        ] {
            if manifest.exists() {
                if let Ok(content) = std::fs::read_to_string(&manifest) {
                    let re = regex::Regex::new(r#"Version="([^"]+)""#).ok()?;
                    if let Some(caps) = re.captures(&content) {
                        return Some(caps[1].to_string());
                    }
                }
            }
        }

        if let Some(name) = claude_path.file_name().and_then(|n| n.to_str()) {
            if let Some(version) = name.strip_prefix("app-") {
                if !version.is_empty() {
                    return Some(version.to_string());
                }
            }
        }

        if let Some(exe_path) = get_windows_claude_exe_path(claude_path) {
            let ps_path = ps_single_quote(&exe_path.to_string_lossy());
            let command = format!("(Get-Item -LiteralPath {}).VersionInfo.ProductVersion", ps_path);
            if let Ok(output) = crate::process::command("powershell.exe")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &command,
                ])
                .output()
            {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn ps_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Get the app data/config path for Claude
pub fn get_app_data_path() -> Option<String> {
    if cfg!(target_os = "macos") {
        let home = dirs::home_dir()?;
        Some(home.join("Library").join("Application Support").join("Claude")
            .to_string_lossy().to_string())
    } else if cfg!(target_os = "windows") {
        let app_data = dirs::config_dir()?;
        Some(app_data.join("Claude").to_string_lossy().to_string())
    } else {
        None
    }
}

/// Check if zh-CN patch is already installed
pub fn check_zh_cn_from_claude(claude_path: &PathBuf) -> bool {
    #[cfg(target_os = "macos")]
    {
        let i18n_path = claude_path
            .join("Contents")
            .join("Resources")
            .join("ion-dist")
            .join("i18n")
            .join("zh-CN.json");
        return i18n_path.exists();
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(resources) = get_windows_claude_resources_path(claude_path) {
            let i18n_path = resources.join("ion-dist").join("i18n").join("zh-CN.json");
            return i18n_path.exists();
        }
    }

    false
}

/// Get the full system information snapshot
pub fn get_full_info() -> SystemInfo {
    let os = detect_os();
    let os_version = get_os_version();
    let claude_path = find_claude_path();
    let claude_installed = claude_path.is_some();
    let claude_version = claude_path.as_ref().and_then(get_claude_version);
    let claude_path_str = claude_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let zh_cn_installed = claude_path.as_ref()
        .map(|p| check_zh_cn_from_claude(p))
        .unwrap_or(false);
    let app_data_path = get_app_data_path();

    SystemInfo {
        os,
        os_version,
        claude_installed,
        claude_path: claude_path_str,
        claude_version,
        zh_cn_installed,
        zh_cn_version: None, // Can be read from resources/release.json
        app_data_path,
    }
}
