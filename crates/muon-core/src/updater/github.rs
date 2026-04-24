use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub html_url: String,
    pub body: Option<String>,
    pub prerelease: bool,
    pub draft: bool,
    pub published_at: Option<String>,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub release_notes: Option<String>,
    pub release_url: String,
    pub is_newer: bool,
}

pub struct UpdateChecker {
    client: reqwest::Client,
    repo_owner: String,
    repo_name: String,
    current_version: String,
}

impl UpdateChecker {
    pub fn new(repo_owner: &str, repo_name: &str, current_version: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(format!("{}-updater/{}", repo_name, current_version))
                .build()
                .unwrap_or_default(),
            repo_owner: repo_owner.to_string(),
            repo_name: repo_name.to_string(),
            current_version: current_version.to_string(),
        }
    }

    pub async fn check_for_update(&self) -> anyhow::Result<Option<UpdateInfo>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            return Err(anyhow::anyhow!(
                "GitHub API returned status {}",
                response.status()
            ));
        }

        let release: GitHubRelease = response.json().await?;

        let latest_version = release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name)
            .to_string();

        let is_newer = is_version_newer(&self.current_version, &latest_version);

        if !is_newer {
            return Ok(None);
        }

        let download_asset = release
            .assets
            .first()
            .map(|a| a.browser_download_url.clone())
            .unwrap_or_default();

        Ok(Some(UpdateInfo {
            current_version: self.current_version.clone(),
            latest_version,
            download_url: download_asset,
            release_notes: release.body,
            release_url: release.html_url,
            is_newer: true,
        }))
    }
}

fn parse_version(v: &str) -> Vec<u64> {
    v.split('-')
        .next()
        .unwrap_or(v)
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect::<Vec<_>>()
}

fn is_version_newer(current: &str, latest: &str) -> bool {
    let cur_parts = parse_version(current);
    let lat_parts = parse_version(latest);

    for i in 0..lat_parts.len().max(cur_parts.len()) {
        let cur = cur_parts.get(i).copied().unwrap_or(0);
        let lat = lat_parts.get(i).copied().unwrap_or(0);
        if lat > cur {
            return true;
        }
        if lat < cur {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_basic() {
        assert_eq!(parse_version("1.2.3"), vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_version_major_only() {
        assert_eq!(parse_version("2"), vec![2]);
    }

    #[test]
    fn test_parse_version_with_suffix() {
        let parts = parse_version("1.2.3-beta");
        assert_eq!(parts, vec![1, 2, 3]);
    }

    #[test]
    fn test_is_version_newer_major() {
        assert!(is_version_newer("1.0.0", "2.0.0"));
        assert!(!is_version_newer("2.0.0", "1.0.0"));
    }

    #[test]
    fn test_is_version_newer_minor() {
        assert!(is_version_newer("1.0.0", "1.1.0"));
        assert!(!is_version_newer("1.1.0", "1.0.0"));
    }

    #[test]
    fn test_is_version_newer_patch() {
        assert!(is_version_newer("1.0.0", "1.0.1"));
        assert!(!is_version_newer("1.0.1", "1.0.0"));
    }

    #[test]
    fn test_is_version_newer_equal() {
        assert!(!is_version_newer("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_is_version_newer_shorter() {
        assert!(is_version_newer("1.0", "1.0.1"));
    }

    #[test]
    fn test_github_release_deserialize() {
        let json = r#"{
            "tag_name": "v1.2.3",
            "name": "Release 1.2.3",
            "html_url": "https://github.com/test/test/releases/tag/v1.2.3",
            "body": "Release notes",
            "prerelease": false,
            "draft": false,
            "published_at": "2026-01-01T00:00:00Z",
            "assets": [{
                "name": "test.tar.gz",
                "browser_download_url": "https://github.com/test/test/releases/download/v1.2.3/test.tar.gz",
                "size": 1024
            }]
        }"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        assert_eq!(release.tag_name, "v1.2.3");
        assert_eq!(release.name, "Release 1.2.3");
        assert!(!release.prerelease);
        assert_eq!(release.assets.len(), 1);
        assert_eq!(release.assets[0].size, 1024);
    }

    #[test]
    fn test_update_checker_new() {
        let checker = UpdateChecker::new("owner", "repo", "1.0.0");
        assert_eq!(checker.repo_owner, "owner");
        assert_eq!(checker.repo_name, "repo");
        assert_eq!(checker.current_version, "1.0.0");
    }
}
