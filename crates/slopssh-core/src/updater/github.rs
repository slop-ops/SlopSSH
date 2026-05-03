//! GitHub Releases API client for checking and downloading updates.

use serde::{Deserialize, Serialize};

/// A GitHub release as returned by the Releases API.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    /// Git tag name (e.g. `"v1.2.3"`).
    pub tag_name: String,
    /// Human-readable release name.
    pub name: String,
    /// URL to the release page on GitHub.
    pub html_url: String,
    /// Markdown release notes body.
    pub body: Option<String>,
    /// Whether this release is a pre-release.
    pub prerelease: bool,
    /// Whether this release is a draft.
    pub draft: bool,
    /// ISO 8601 publication timestamp.
    pub published_at: Option<String>,
    /// Downloadable assets attached to the release.
    pub assets: Vec<GitHubAsset>,
}

/// A single downloadable asset within a GitHub release.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAsset {
    /// Asset filename.
    pub name: String,
    /// Direct download URL.
    pub browser_download_url: String,
    /// Asset size in bytes.
    pub size: u64,
}

/// Information about an available application update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Currently installed version string.
    pub current_version: String,
    /// Latest available version string.
    pub latest_version: String,
    /// URL to download the update.
    pub download_url: String,
    /// Markdown release notes for the update.
    pub release_notes: Option<String>,
    /// URL to view the release on GitHub.
    pub release_url: String,
    /// Whether the latest version is newer than the current one.
    pub is_newer: bool,
}

/// Checks for application updates via the GitHub Releases API.
pub struct UpdateChecker {
    client: reqwest::Client,
    repo_owner: String,
    repo_name: String,
    current_version: String,
}

impl UpdateChecker {
    /// Creates a new update checker for the given GitHub repository.
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

    /// Queries the GitHub API for the latest release and returns update info if newer.
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

    /// Downloads the update file to the config directory and returns its path.
    pub async fn download_update(&self, info: &UpdateInfo) -> anyhow::Result<std::path::PathBuf> {
        let dir = crate::config::paths::config_dir()?.join("updates");
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        let url = &info.download_url;
        let file_name = url.rsplit('/').next().unwrap_or("update").to_string();
        let dest = dir.join(&file_name);

        tracing::info!(url = %url, dest = %dest.display(), "Downloading update");

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Download failed with status {}",
                response.status()
            ));
        }

        let total_size = response.content_length().unwrap_or(0);
        tracing::info!(total_bytes = total_size, "Update download started");

        let bytes = response.bytes().await?;
        std::fs::write(&dest, &bytes)?;

        tracing::info!(path = %dest.display(), size = bytes.len(), "Update downloaded");
        Ok(dest)
    }
}

/// Parses a semver-like version string into numeric parts, ignoring pre-release suffixes.
fn parse_version(v: &str) -> Vec<u64> {
    v.split('-')
        .next()
        .unwrap_or(v)
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect::<Vec<_>>()
}

/// Compares two version strings and returns `true` if `latest` is newer than `current`.
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
