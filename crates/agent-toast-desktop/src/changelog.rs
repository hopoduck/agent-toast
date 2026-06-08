use serde::{Deserialize, Serialize};
use std::time::Duration;

const MARKER_START: &str = "<!-- changelog:start -->";
const MARKER_END: &str = "<!-- changelog:end -->";

/// 릴리즈 body에서 마커 사이 changelog 본문만 추출한다.
/// 마커가 없으면(과거 릴리즈) body 전체를 trim 해서 반환한다.
pub fn extract_changelog(body: &str) -> String {
    if let (Some(s), Some(e)) = (body.find(MARKER_START), body.find(MARKER_END)) {
        if s < e {
            let start = s + MARKER_START.len();
            return body[start..e].trim().to_string();
        }
    }
    body.trim().to_string()
}

/// 프론트로 전달하는 릴리즈 정보 (마커 추출 완료된 changelog 포함)
#[derive(Debug, Serialize, PartialEq)]
pub struct ReleaseInfo {
    pub version: String,
    pub name: String,
    pub published_at: String,
    pub changelog: String,
    pub url: String,
}

/// GitHub Releases API 응답 (필요한 필드만)
#[derive(Debug, Deserialize)]
struct GithubReleaseFull {
    tag_name: String,
    name: Option<String>,
    published_at: Option<String>,
    body: Option<String>,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// draft·prerelease(rc/beta 등)를 제외하고 정식 릴리즈만 매핑한다.
fn filter_and_map(releases: Vec<GithubReleaseFull>) -> Vec<ReleaseInfo> {
    releases
        .into_iter()
        .filter(|r| !r.draft && !r.prerelease)
        .map(to_release_info)
        .collect()
}

fn to_release_info(r: GithubReleaseFull) -> ReleaseInfo {
    let tag = r.tag_name.clone();
    ReleaseInfo {
        name: r
            .name
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| tag.clone()),
        version: tag,
        published_at: r.published_at.unwrap_or_default(),
        changelog: extract_changelog(&r.body.unwrap_or_default()),
        url: r.html_url,
    }
}

#[tauri::command]
pub fn get_releases() -> Result<Vec<ReleaseInfo>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.github.com/repos/hopoduck/agent-toast/releases?per_page=10")
        .header("User-Agent", "agent-toast")
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API 오류: {}", resp.status()));
    }

    let releases: Vec<GithubReleaseFull> = resp.json().map_err(|e| e.to_string())?;
    Ok(filter_and_map(releases))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_between_markers() {
        let body = "<!-- changelog:start -->\n✨ 기능\n- 추가\n<!-- changelog:end -->\n\n## 설치 방법\n- exe";
        assert_eq!(extract_changelog(body), "✨ 기능\n- 추가");
    }

    #[test]
    fn falls_back_to_full_body_when_no_markers() {
        let body = "  Agent Toast 업데이트입니다.\n- 항목  ";
        assert_eq!(
            extract_changelog(body),
            "Agent Toast 업데이트입니다.\n- 항목"
        );
    }

    #[test]
    fn falls_back_when_only_start_marker() {
        let body = "<!-- changelog:start -->\n내용";
        assert_eq!(extract_changelog(body), "<!-- changelog:start -->\n내용");
    }

    #[test]
    fn falls_back_when_markers_reversed() {
        let body = "<!-- changelog:end -->before<!-- changelog:start -->";
        // end가 start보다 앞 → fallback (전체 trim)
        assert_eq!(extract_changelog(body), body.trim());
    }

    #[test]
    fn empty_body_returns_empty() {
        assert_eq!(extract_changelog(""), "");
    }

    #[test]
    fn maps_github_release_with_markers() {
        let json = r#"{
            "tag_name": "v1.0.1",
            "name": "v1.0.1",
            "published_at": "2026-06-01T00:00:00Z",
            "body": "<!-- changelog:start -->\n🐛 버그 수정\n- 고침\n<!-- changelog:end -->\n## 설치 방법",
            "html_url": "https://github.com/hopoduck/agent-toast/releases/tag/v1.0.1",
            "draft": false
        }"#;
        let r: GithubReleaseFull = serde_json::from_str(json).unwrap();
        let info = to_release_info(r);
        assert_eq!(info.version, "v1.0.1");
        assert_eq!(info.changelog, "🐛 버그 수정\n- 고침");
        assert_eq!(info.published_at, "2026-06-01T00:00:00Z");
    }

    #[test]
    fn filter_and_map_excludes_draft_and_prerelease() {
        let json = r#"[
            { "tag_name": "v1.1.0", "body": "stable", "html_url": "u1", "draft": false, "prerelease": false },
            { "tag_name": "v1.2.0-rc1", "body": "rc", "html_url": "u2", "draft": false, "prerelease": true },
            { "tag_name": "v1.3.0-draft", "body": "draft", "html_url": "u3", "draft": true, "prerelease": false }
        ]"#;
        let releases: Vec<GithubReleaseFull> = serde_json::from_str(json).unwrap();
        let result = filter_and_map(releases);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version, "v1.1.0");
    }

    #[test]
    fn name_falls_back_to_tag_when_empty() {
        let json = r#"{
            "tag_name": "v0.9.0",
            "name": "",
            "body": "내용",
            "html_url": "https://x/y"
        }"#;
        let r: GithubReleaseFull = serde_json::from_str(json).unwrap();
        let info = to_release_info(r);
        assert_eq!(info.name, "v0.9.0");
        assert_eq!(info.changelog, "내용");
        assert_eq!(info.published_at, "");
    }
}
