use anyhow::{Context, Result, ensure};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const API: &str = "https://api.github.com/repos/M1styuki/RadeonTuner-zh";
pub const RELEASES: &str = "https://github.com/M1styuki/RadeonTuner-zh/releases";
const ASSET: &str = "RadeonTuner-Rust-x64.exe";
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
        "此仓库需要登录 GitHub。可通过发布页面下载，或在 GitHub CLI 登录后再次检查"
    );
    let mut cmd = Command::new(path);
    cmd.creation_flags(0x08000000)
        .env("GH_PROMPT_DISABLED", "1")
        .args(["api", "--hostname", "github.com"]);
    Ok(cmd)
}
fn request(access: Access, route: &str, accept: &str) -> Result<Command> {
    let mut cmd = match access {
        Access::Public => {
            let mut c = curl()?;
            c.args([
                "--header",
                &format!("Accept: {accept}"),
                &format!("{API}/{route}"),
            ]);
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
    cmd.stdin(Stdio::null());
    Ok(cmd)
}
pub fn check() -> Result<Option<Update>> {
    let current = Version::parse(env!("CARGO_PKG_VERSION"))?;
    let mut access = Access::Public;
    let mut releases = Vec::new();
    for page in 1.. {
        let route = format!("releases?per_page=100&page={page}");
        let mut res = request(access, &route, "application/vnd.github+json")?.output()?;
        if !res.status.success()
            && page == 1
            && String::from_utf8_lossy(&res.stderr).contains("404")
        {
            access = Access::GithubCli;
            res = request(access, &route, "application/vnd.github+json")?.output()?;
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
    let result = request(update.access, &route, "application/octet-stream")?
        .stdout(Stdio::from(file.reopen()?))
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
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
