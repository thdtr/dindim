use std::time::Duration;

use serde::Deserialize;

const RELEASES_URL: &str = "https://api.github.com/repos/thdtr/dindim/releases?per_page=30";
pub const RELEASES_PAGE: &str = "https://github.com/thdtr/dindim/releases";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub version: semver::Version,
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// Asks GitHub for a release newer than the running one. Pre-releases count
/// only while the running version is itself a pre-release.
pub fn check(current: &str) -> Option<Release> {
    let current = semver::Version::parse(current).ok()?;
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(concat!("dindim/", env!("CARGO_PKG_VERSION")))
        .build()
        .new_agent();
    let releases: Vec<GithubRelease> = agent
        .get(RELEASES_URL)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| tracing::info!(error = %e, "update check failed"))
        .ok()?
        .body_mut()
        .read_json()
        .map_err(|e| tracing::info!(error = %e, "update check returned bad data"))
        .ok()?;
    newest(&current, releases)
}

fn newest(current: &semver::Version, releases: Vec<GithubRelease>) -> Option<Release> {
    let allow_pre = !current.pre.is_empty();
    releases
        .into_iter()
        .filter(|r| !r.draft && (allow_pre || !r.prerelease))
        .filter_map(|r| {
            let version = semver::Version::parse(r.tag_name.trim_start_matches('v')).ok()?;
            (version > *current).then_some(Release {
                version,
                url: r.html_url,
            })
        })
        .max_by(|a, b| a.version.cmp(&b.version))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, prerelease: bool) -> GithubRelease {
        GithubRelease {
            tag_name: tag.into(),
            html_url: format!("https://example.com/{tag}"),
            draft: false,
            prerelease,
        }
    }

    fn v(s: &str) -> semver::Version {
        semver::Version::parse(s).unwrap()
    }

    #[test]
    fn picks_the_newest_applicable_release() {
        let list = || {
            vec![
                release("v0.1.0-alpha.1", true),
                release("v0.1.0-alpha.3", true),
                release("v0.1.0", false),
                release("v0.2.0-beta.1", true),
                release("garbage", false),
            ]
        };
        let from_alpha = newest(&v("0.1.0-alpha.1"), list()).unwrap();
        assert_eq!(from_alpha.version, v("0.2.0-beta.1"));
        let from_stable = newest(&v("0.1.0"), list());
        assert_eq!(from_stable, None);
        let from_old = newest(&v("0.0.9"), list()).unwrap();
        assert_eq!(from_old.version, v("0.1.0"));
    }
}
