use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
        if let Ok(output) = std::process::Command::new("cmd")
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
    if cfg!(target_os = "macos") {
        let app_path = PathBuf::from("/Applications/Claude.app");
        if app_path.exists() {
            return Some(app_path);
        }
    } else if cfg!(target_os = "windows") {
        // Check Windows AppX install
        let local_app_data = dirs::data_local_dir()?;
        let anthropic_base = local_app_data.join("AnthropicClaude");
        if anthropic_base.exists() {
            // Look for app-* directories (unpackaged install)
            if let Ok(entries) = std::fs::read_dir(&anthropic_base) {
                let mut app_dirs: Vec<PathBuf> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .starts_with("app-")
                    })
                    .map(|e| e.path())
                    .collect();
                app_dirs.sort_by(|a, b| {
                    b.metadata().unwrap().modified().unwrap()
                        .cmp(&a.metadata().unwrap().modified().unwrap())
                });
                if let Some(latest) = app_dirs.first() {
                    return Some(latest.clone());
                }
            }
        }

        // Check Program Files
        let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let claude_path = PathBuf::from(&program_files).join("Claude").join("Claude.exe");
        if claude_path.exists() {
            return Some(claude_path);
        }
    }
    None
}

/// Get Claude Desktop version from Info.plist (macOS) or executable metadata (Windows)
pub fn get_claude_version(claude_path: &PathBuf) -> Option<String> {
    if cfg!(target_os = "macos") {
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
    } else if cfg!(target_os = "windows") {
        // Try from appx manifest or just return path
        if let Some(parent) = claude_path.parent() {
            let manifest = parent.join("AppxManifest.xml");
            if manifest.exists() {
                if let Ok(content) = std::fs::read_to_string(&manifest) {
                    let re = regex::Regex::new(r#"Version="([^"]+)""#).ok()?;
                    if let Some(caps) = re.captures(&content) {
                        return Some(caps[1].to_string());
                    }
                }
            }
        }
    }
    None
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
    if cfg!(target_os = "macos") {
        let i18n_path = claude_path
            .join("Contents")
            .join("Resources")
            .join("ion-dist")
            .join("i18n")
            .join("zh-CN.json");
        return i18n_path.exists();
    } else if cfg!(target_os = "windows") {
        let resources = claude_path.join("resources");
        if resources.exists() {
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
