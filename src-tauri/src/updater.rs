use serde::{Deserialize, Serialize};

const APP_REPO: &str = "young8i/claude-releases";
const GITHUB_API: &str = "https://api.github.com/repos";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub check_time: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

fn parse_ver(v: &str) -> Vec<u32> {
    v.trim_start_matches('v').split('.').filter_map(|s| s.parse().ok()).collect()
}

fn is_newer(latest: &str, current: &str) -> bool {
    let l = parse_ver(latest);
    let c = parse_ver(current);
    for (i, &lp) in l.iter().enumerate() {
        let cp = c.get(i).copied().unwrap_or(0);
        if lp > cp { return true; }
        if lp < cp { return false; }
    }
    l.len() > c.len()
}

pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub async fn check_app_update() -> Result<AppUpdateInfo, String> {
    let current = get_app_version();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| format!("网络错误: {}", e))?;

    let resp = client
        .get(format!("{}/{}/releases/latest", GITHUB_API, APP_REPO))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "claude-zh-helper")
        .send().await.map_err(|e| format!("请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub 返回 {}", resp.status()));
    }

    let release: GitHubRelease = resp.json().await.map_err(|e| format!("解析失败: {}", e))?;
    let latest = release.tag_name.trim_start_matches('v').to_string();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let local = now + 8 * 3600;
    let days = local / 86400;
    let t = local % 86400;
    let y = 1970 + (days / 365) as u64;
    let r = days % 365;
    let mo = (r / 30 + 1) as u64;
    let d = (r % 30 + 1) as u64;
    let check_time = format!("{:04}-{:02}-{:02} {:02}:{:02}", y, mo, d, t / 3600, (t % 3600) / 60);

    Ok(AppUpdateInfo {
        has_update: is_newer(&latest, &current),
        current_version: current,
        latest_version: latest,
        release_url: release.html_url,
        check_time,
    })
}
