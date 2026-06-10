use serde::{Deserialize, Serialize};

const CCSWITCH_RELEASES: &str = "https://github.com/farion1231/cc-switch/releases";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CcSwitchStatus {
    pub installed: bool,
    pub install_path: Option<String>,
    pub version: Option<String>,
}

pub fn check_status() -> CcSwitchStatus {
    let (installed, install_path) = find_ccswitch();
    let version = install_path.as_ref().and_then(|p| get_version(p));
    CcSwitchStatus { installed, install_path, version }
}

fn find_ccswitch() -> (bool, Option<String>) {
    #[cfg(target_os = "macos")]
    {
        // Strategy 1: check /Applications for any app matching "CC-Switch" (various naming)
        if let Ok(entries) = std::fs::read_dir("/Applications") {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_lowercase();
                if name.contains("cc-switch") || name.contains("ccswitch") || name.contains("cc_switch") {
                    return (true, Some(e.path().to_string_lossy().to_string()));
                }
            }
        }

        // Strategy 2: Homebrew Caskroom
        let brew_paths = [
            "/opt/homebrew/Caskroom/cc-switch",
            "/usr/local/Caskroom/cc-switch",
        ];
        for bp in &brew_paths {
            let p = std::path::Path::new(bp);
            if p.exists() {
                // Look for the .app inside the version subdirectory
                if let Ok(entries) = std::fs::read_dir(p) {
                    for e in entries.flatten() {
                        let app = e.path().join("CC-Switch.app");
                        if app.exists() {
                            return (true, Some(app.to_string_lossy().to_string()));
                        }
                        // Also check for .app directly
                        for app_e in std::fs::read_dir(&e.path()).into_iter().flatten().flatten() {
                            let n = app_e.file_name().to_string_lossy().into_owned();
                            if n.ends_with(".app") && n.to_lowercase().contains("switch") {
                                return (true, Some(app_e.path().to_string_lossy().to_string()));
                            }
                        }
                    }
                }
            }
        }

        // Strategy 3: which / type
        for cmd in &["cc-switch", "ccswitch"] {
            if let Ok(out) = std::process::Command::new("which").arg(cmd).output() {
                let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !p.is_empty() && std::path::Path::new(&p).exists() {
                    return (true, Some(p));
                }
            }
        }

        // Strategy 4: mdfind (Spotlight search) — slow but thorough
        if let Ok(out) = std::process::Command::new("mdfind")
            .args(["-name", "CC-Switch"])
            .output()
        {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let p = line.trim();
                if !p.is_empty() && std::path::Path::new(p).exists() && p.contains("CC-Switch") {
                    return (true, Some(p.to_string()));
                }
            }
        }

        // Strategy 5: brew list to find cask
        if let Ok(out) = std::process::Command::new("brew").args(["list", "--cask"]).output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.trim().to_lowercase().contains("cc-switch") {
                    // The cask is installed via brew; we already checked paths above.
                    // Mark as installed even if we can't find the exact path.
                    return (true, Some("/Applications/CC-Switch.app".to_string()));
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Program Files\CC-Switch\CC-Switch.exe",
            r"C:\Program Files\CC Switch\CC Switch.exe",
            r"C:\Program Files\CCSwitch\CCSwitch.exe",
            r"C:\Program Files (x86)\CC-Switch\CC-Switch.exe",
            r"C:\Program Files (x86)\CC Switch\CC Switch.exe",
            r"C:\Program Files (x86)\CCSwitch\CCSwitch.exe",
        ];
        for p in &candidates {
            if std::path::Path::new(p).exists() { return (true, Some(p.to_string())); }
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            for sub in &[
                "Programs\\CC-Switch",
                "Programs\\CC Switch",
                "Programs\\CCSwitch",
                "Programs\\cc-switch",
                "cc-switch",
            ] {
                let base = std::path::PathBuf::from(&local).join(sub);
                for name in &["CC-Switch.exe", "CC Switch.exe", "CCSwitch.exe", "cc-switch.exe", "ccswitch.exe"] {
                    let exe = base.join(name);
                    if exe.exists() { return (true, Some(exe.to_string_lossy().to_string())); }
                }
            }
        }

        if let Some(path) = find_ccswitch_from_windows_registry() {
            return (true, Some(path));
        }

        for cmd in &["cc-switch", "ccswitch", "CC-Switch", "CCSwitch"] {
            if let Ok(out) = std::process::Command::new("where").arg(cmd).output() {
                let p = String::from_utf8_lossy(&out.stdout).lines().next().unwrap_or("").trim().to_string();
                if !p.is_empty() { return (true, Some(p)); }
            }
        }
    }

    (false, None)
}

#[cfg(target_os = "windows")]
fn find_ccswitch_from_windows_registry() -> Option<String> {
    let command = r#"
$roots = @(
  'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
  'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
  'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*'
)
$apps = @(Get-ItemProperty $roots -ErrorAction SilentlyContinue |
  Where-Object { $_.DisplayName -match 'CC[- ]?Switch|ccswitch' })
foreach ($app in $apps) {
  if ($app.InstallLocation -and (Test-Path -LiteralPath $app.InstallLocation)) {
    foreach ($name in @('CC-Switch.exe','CC Switch.exe','CCSwitch.exe','cc-switch.exe','ccswitch.exe')) {
      $candidate = Join-Path $app.InstallLocation $name
      if (Test-Path -LiteralPath $candidate) { $candidate; exit 0 }
    }
  }
  if ($app.DisplayIcon) {
    $icon = ([string]$app.DisplayIcon).Trim('"')
    $icon = $icon -replace ',\d+$',''
    if (Test-Path -LiteralPath $icon) { $icon; exit 0 }
  }
}
"#;

    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            command,
        ])
        .output()
        .ok()?;

    let path = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    if path.is_empty() { None } else { Some(path) }
}

fn get_version(install_path: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        // From Info.plist
        let plist = std::path::Path::new(install_path).join("Contents").join("Info.plist");
        if plist.exists() {
            if let Ok(out) = std::process::Command::new("plutil")
                .args(["-extract", "CFBundleShortVersionString", "raw", "-o", "-", plist.to_str()?])
                .output()
            {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !v.is_empty() { return Some(v); }
            }
        }
        // From Homebrew info
        if let Ok(out) = std::process::Command::new("brew").args(["info", "--cask", "cc-switch"]).output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.trim().starts_with("cc-switch:") {
                    return line.split_whitespace().nth(1).map(|s| s.to_string());
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let command = format!(
            "(Get-Item -LiteralPath {}).VersionInfo.ProductVersion",
            ps_single_quote(install_path),
        );
        if let Ok(out) = std::process::Command::new("powershell.exe")
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
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !v.is_empty() { return Some(v); }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn ps_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(target_os = "windows")]
fn run_powershell_with_timeout(
    command: &str,
    timeout: std::time::Duration,
) -> Result<std::process::Output, String> {
    let mut child = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            command,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("PowerShell 不可用: {}", e))?;

    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let status = child
                    .wait()
                    .map_err(|e| format!("无法读取 PowerShell 退出状态: {}", e))?;
                return Ok(std::process::Output {
                    status,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                });
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("安装命令超过 {} 秒仍未结束", timeout.as_secs()));
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(e) => return Err(format!("等待 PowerShell 结束失败: {}", e)),
        }
    }
}

pub fn run_one_click_install() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        // Step 1: brew tap
        let tap = std::process::Command::new("brew")
            .args(["tap", "farion1231/ccswitch"])
            .output()
            .map_err(|e| format!("无法执行 brew。请先安装 Homebrew:\n/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"\n\n错误: {}", e))?;

        if !tap.status.success() {
            let stderr = String::from_utf8_lossy(&tap.stderr);
            if !stderr.contains("already tapped") && !stderr.contains("already exists") {
                return Err(format!("brew tap 失败。请手动从 {} 下载。\n\n{}", CCSWITCH_RELEASES, stderr));
            }
        }

        // Step 2: brew install --cask
        let install = std::process::Command::new("brew")
            .args(["install", "--cask", "cc-switch"])
            .output()
            .map_err(|e| format!("brew install 失败: {}", e))?;

        if install.status.success() {
            Ok("✅ cc-switch 安装成功！已安装到 /Applications/CC-Switch.app\n\n请点击顶部菜单栏的 cc-switch 图标打开面板。".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&install.stderr);
            if stderr.contains("already installed") {
                Ok("✅ cc-switch 已安装（brew 确认）。".to_string())
            } else {
                Err(format!("安装失败。请手动从 {} 下载 .zip 安装包拖入 Applications。\n\n错误: {}", CCSWITCH_RELEASES, stderr))
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let command = r#"
$ErrorActionPreference = 'Stop'
$packageId = 'farion1231.CC-Switch'
$winget = (Get-Command winget.exe -ErrorAction SilentlyContinue).Source
if ($winget) {
  & $winget install --id $packageId --exact --source winget --silent --accept-package-agreements --accept-source-agreements --disable-interactivity
  if ($LASTEXITCODE -eq 0) { exit 0 }
  Write-Host "winget failed with exit code $LASTEXITCODE; falling back to GitHub Releases."
}
else {
  Write-Host "winget.exe not found; falling back to GitHub Releases."
}

$release = Invoke-RestMethod `
  -Uri 'https://api.github.com/repos/farion1231/cc-switch/releases/latest' `
  -Headers @{ Accept = 'application/vnd.github+json'; 'User-Agent' = 'claude-zh-helper' } `
  -TimeoutSec 20
$asset = @($release.assets | Where-Object { $_.name -match 'Windows.*\.msi$|\.msi$' } | Select-Object -First 1)
if (-not $asset) {
  throw 'GitHub Releases 中未找到 Windows MSI 安装包。'
}
$installer = Join-Path $env:TEMP $asset.name
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $installer -UseBasicParsing
$escapedInstaller = '"' + ($installer -replace '"', '\"') + '"'
$p = Start-Process -FilePath 'msiexec.exe' -ArgumentList "/i $escapedInstaller /qn /norestart" -Verb RunAs -Wait -PassThru -ErrorAction Stop
if ($p.ExitCode -in @(0, 3010, 1638)) { exit 0 }
if ($null -ne $p.ExitCode) { exit $p.ExitCode }
exit 0
"#;

        let out = run_powershell_with_timeout(command, std::time::Duration::from_secs(600));

        match out {
            Ok(o) if o.status.success() || check_status().installed => {
                Ok("✅ cc-switch 安装成功！请在开始菜单或系统托盘中启动。".to_string())
            }
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                Err(format!(
                    "cc-switch 一键安装失败（退出状态: {}）。请手动从 {} 下载 .msi 安装包。\n\n{}\n{}",
                    o.status, CCSWITCH_RELEASES, stdout, stderr,
                ))
            }
            Err(e) if check_status().installed => {
                Ok("✅ cc-switch 已安装。安装器未正常退出，但已检测到安装结果。".to_string())
            }
            Err(e) => Err(format!(
                "cc-switch 一键安装未完成。请手动从 {} 下载。\n\n{}",
                CCSWITCH_RELEASES, e,
            )),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { Err(format!("当前平台不支持。请访问 {} 下载。", CCSWITCH_RELEASES)) }
}

pub fn get_config_guide() -> String {
    r##"# cc-switch 安装与配置教程

cc-switch 是免费开源的 Claude Desktop / Claude Code 配置管理工具（[github.com/farion1231/cc-switch](https://github.com/farion1231/cc-switch)）。

## 安装方式

**macOS**:
```bash
brew tap farion1231/ccswitch
brew install --cask cc-switch
```

**Windows**:
```bash
winget install --id farion1231.CC-Switch --exact --source winget
```

也可从 [GitHub Releases](https://github.com/farion1231/cc-switch/releases) 手动下载安装包。

## 启动与使用

1. 安装后打开 cc-switch，图标出现在系统托盘（macOS 右上角菜单栏 / Windows 右下角）
2. 点击图标打开管理面板
3. 添加 API Provider（Anthropic / OpenRouter / DeepSeek / Moonshot 等）
4. 配置 Claude Desktop 应用关联 → 选择模型 → Apply

## 配合中文补丁

| 需求 | 补丁模式 | cc-switch 作用 |
|------|---------|---------------|
| 多模型 + Cowork | 模式 1（安全） | 别名映射代理 |
| 官方 API 中文 | 模式 2 | 不需要 |
| 全部功能 | 模式 3 | 代理模式 |

> ⚠️ **安全提醒**: cc-switch **完全免费**。任何收费的 "CC Switch" 都是钓鱼网站。只从 https://ccswitch.io 或 GitHub 官方下载。

📖 官网：https://ccswitch.io
💬 反馈：https://github.com/farion1231/cc-switch/issues
"##.to_string()
}
