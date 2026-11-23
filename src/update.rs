use anyhow::{Context, Result, ensure};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const API: &str = "https://api.github.com/repos/M1styuki/RadeonTuner-zh";
pub const RELEASES: &str = "https://github.com/M1styuki/RadeonTuner-zh/releases";
const ASSET: &str = "RadeonTuner-Rust-x64.exe";
const TOKEN_ENV: &str = "RADEONTUNER_GITHUB_TOKEN";
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    prerelease: bool,
    draft: bool,
    assets: Vec<Asset>,
}
#[derive(Clone, Deserialize)]
struct Asset {
    id: u64,
    name: String,
    digest: Option<String>,
    size: u64,
}
#[derive(Clone)]
pub struct Update {
    pub version: Version,
    id: u64,
    digest: String,
    size: u64,
    access: Access,
}
#[derive(Clone, Copy)]
enum Access {
    Public,
    Token,
    GithubCli,
}
fn select(releases: Vec<Release>, current: &Version, access: Access) -> Result<Option<Update>> {
    let candidate = releases
        .into_iter()
        .filter(|r| !r.draft && !r.prerelease)
        .filter_map(|r| {
            let version = Version::parse(r.tag_name.strip_prefix("rust-v")?).ok()?;
            let asset = r.assets.into_iter().find(|a| a.name == ASSET)?;
            (version.pre.is_empty() && &version > current).then_some((version, asset))
        })
        .max_by(|(a, _), (b, _)| a.cmp(b));
    candidate
        .map(|(version, asset)| {
            let digest = asset
                .digest
                .as_deref()
                .and_then(|s| s.strip_prefix("sha256:"))
                .context("GitHub 发布文件缺少 SHA-256 校验值")?;
            ensure!(
                asset.size > 0
                    && digest.len() == 64
                    && digest.bytes().all(|b| b.is_ascii_hexdigit()),
                "GitHub 发布文件信息无效"
            );
            Ok(Update {
                version,
                id: asset.id,
                digest: digest.to_ascii_lowercase(),
                size: asset.size,
                access,
            })
        })
        .transpose()
}
fn curl() -> Result<Command> {
    let mut cmd = Command::new(crate::integration::system_folder()?.join("curl.exe"));
    cmd.creation_flags(0x08000000).args([
        "--disable",
        "--silent",
        "--show-error",
        "--fail",
        "--location",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--connect-timeout",
        "8",
        "--max-time",
        "120",
        "--user-agent",
        "RadeonTuner-zh",
    ]);
    Ok(cmd)
}
fn gh() -> Result<Command> {
    let path = std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .context("未找到 GitHub CLI")?
        .join("GitHub CLI/gh.exe");
    ensure!(
        path.is_file(),
        "私有仓库可通过 GitHub 下载页获取更新，或配置 RADEONTUNER_GITHUB_TOKEN 后在应用内检查"
    );
    let mut cmd = Command::new(path);
    cmd.creation_flags(0x08000000)
        .env("GH_PROMPT_DISABLED", "1")
        .args(["api", "--hostname", "github.com"]);
    Ok(cmd)
}
fn request(access: Access, route: &str, accept: &str) -> Result<Command> {
    let cmd = match access {
        Access::Public | Access::Token => {
            let mut c = curl()?;
            c.args([
                "--header",
                &format!("Accept: {accept}"),
                &format!("{API}/{route}"),
            ]);
            if matches!(access, Access::Token) {
                c.args(["--header", "@-"]);
            }
            c
        }
        Access::GithubCli => {
            let mut c = gh()?;
            c.args([
                "--method",
                "GET",
                "--header",
                &format!("Accept: {accept}"),
                &format!("repos/M1styuki/RadeonTuner-zh/{route}"),
            ]);
            c
        }
    };
    Ok(cmd)
}

fn fetch(access: Access, route: &str, accept: &str, stdout: Stdio) -> Result<Output> {
    let mut cmd = request(access, route, accept)?;
    let token = matches!(access, Access::Token)
        .then(|| std::env::var(TOKEN_ENV).context("未读取到 RadeonTuner GitHub 更新令牌"))
        .transpose()?;
    if let Some(token) = &token {
        ensure!(!token.is_empty() && token.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
            "RadeonTuner GitHub 更新令牌格式无效");
    }
    let mut child = cmd.stdin(if token.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(stdout).stderr(Stdio::piped()).spawn()?;
    if let Some(token) = token {
        let mut input = child.stdin.take().context("无法传入 GitHub 更新令牌")?;
        writeln!(input, "Authorization: Bearer {token}").context("无法传入 GitHub 更新令牌")?;
    }
    Ok(child.wait_with_output()?)
}
pub fn check() -> Result<Option<Update>> {
    let current = Version::parse(env!("CARGO_PKG_VERSION"))?;
    let mut access = if std::env::var_os(TOKEN_ENV).is_some_and(|v| !v.is_empty()) {
        Access::Token
    } else {
        Access::Public
    };
    let mut releases = Vec::new();
    for page in 1.. {
        let route = format!("releases?per_page=100&page={page}");
        let mut res = fetch(access, &route, "application/vnd.github+json", Stdio::piped())?;
        if !res.status.success()
            && page == 1
            && matches!(access, Access::Public)
            && String::from_utf8_lossy(&res.stderr).contains("404")
        {
            access = Access::GithubCli;
            res = fetch(access, &route, "application/vnd.github+json", Stdio::piped())?;
        }
        ensure!(
            res.status.success(),
            "无法检查 GitHub 更新：{}",
            String::from_utf8_lossy(&res.stderr).trim()
        );
        let batch: Vec<Release> = serde_json::from_slice(&res.stdout)?;
        let more = batch.len() == 100;
        releases.extend(batch);
        if !more {
            break;
        }
    }
    select(releases, &current, access)
}
fn verify(path: &Path, update: &Update) -> Result<()> {
    let mut f = File::open(path)?;
    ensure!(f.metadata()?.len() == update.size, "下载的 EXE 大小不匹配");
    let mut hash = Sha256::new();
    let mut buf = [0; 65536];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    ensure!(
        format!("{:x}", hash.finalize()) == update.digest,
        "下载的 EXE 未通过 GitHub 的 SHA-256 校验"
    );
    Ok(())
}
pub fn install(update: &Update) -> Result<()> {
    let mut file = tempfile::Builder::new()
        .prefix("RadeonTuner-update-")
        .suffix(".exe")
        .tempfile()?;
    let route = format!("releases/assets/{}", update.id);
    let result = fetch(
        update.access,
        &route,
        "application/octet-stream",
        Stdio::from(file.reopen()?),
    )?;
    ensure!(
        result.status.success(),
        "下载 Rust 更新失败：{}",
        String::from_utf8_lossy(&result.stderr).trim()
    );
    file.flush()?;
    file.as_file().sync_all()?;
    verify(file.path(), update)?;
    self_replace::self_replace(file.path()).context("替换程序文件失败")?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_rust_channel_excludes_native_preview_and_older_releases() {
        let digest = format!("sha256:{:x}", Sha256::digest(b"fixture"));
        let r = |name: &str, preview: bool| Release {
            tag_name: name.into(),
            prerelease: preview,
            draft: false,
            assets: vec![Asset {
                id: 12,
                name: ASSET.into(),
                digest: Some(digest.clone()),
                size: 7,
            }],
        };
        let check = select(
            vec![
                r("v8.0.0", false),
                r("rust-v2.0.0", true),
                r("rust-v0.74.0", false),
                r("rust-v0.76.0", false),
            ],
            &Version::new(0, 75, 0),
            Access::Public,
        )
        .unwrap()
        .unwrap();
        assert_eq!(check.version, Version::new(0, 76, 0));
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"fixture").unwrap();
        verify(file.path(), &check).unwrap();
        std::fs::write(file.path(), b"changed").unwrap();
        assert!(verify(file.path(), &check).is_err());
    }
}
