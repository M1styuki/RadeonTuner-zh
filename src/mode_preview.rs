use crate::adl::{
    Driver,
    display::{Display, Mode, custom::Timing},
};
use anyhow::{Context, Result, ensure};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

pub const TIMEOUT: Duration = Duration::from_secs(12);
pub struct Preview {
    child: Child,
    input: Option<ChildStdin>,
    pub started: Instant,
    pub custom: bool,
}

impl Preview {
    pub fn start(display: &Display, mode: Mode, angle: i32) -> Result<Self> {
        Self::spawn(
            display,
            &[
                mode.width.to_string(),
                mode.height.to_string(),
                mode.refresh.to_bits().to_string(),
                angle.to_string(),
            ],
            false,
        )
    }
    pub fn custom(display: &Display, key: Option<&str>, timing: Timing) -> Result<Self> {
        Self::spawn(
            display,
            &[
                key.unwrap_or_default().to_string(),
                serde_json::to_string(&timing)?,
            ],
            true,
        )
    }
    fn spawn(display: &Display, args: &[String], custom: bool) -> Result<Self> {
        let mut child = Command::new(std::env::current_exe()?)
            .arg("--preview-mode")
            .args([
                display.adapter.to_string(),
                display.logical.to_string(),
                display.name.clone(),
            ])
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let mut line = String::new();
        BufReader::new(child.stdout.take().context("无法读取预览结果")?).read_line(&mut line)?;
        ensure!(line.trim() == "READY", "显示模式预览失败：{}", line.trim());
        Ok(Self {
            input: child.stdin.take(),
            child,
            started: Instant::now(),
            custom,
        })
    }
    pub fn finish(&mut self, keep: bool) -> Result<bool> {
        if let Some(mut input) = self.input.take() {
            if keep {
                let _ = input.write_all(b"K");
            }
            drop(input);
        }
        match self.child.wait()?.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => anyhow::bail!("无法还原显示模式，请检查显示器连接"),
        }
    }
}

// The independent process still owns the original mode if the panel or host exits.
fn run() -> Result<bool> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    ensure!(matches!(args.len(), 5 | 7), "预览参数无效");
    let adapter = args[0].parse()?;
    let logical: i32 = args[1].parse()?;
    let driver = Driver::open()?;
    let display = driver
        .display_connections(adapter)?
        .into_iter()
        .find(|d| d.logical == logical && d.name == args[2])
        .context("显示器连接已变化")?;
    if args.len() == 5 {
        let identity = |d: &Display| {
            d.logical == logical
                && d.name == args[2]
                && (args[3].is_empty()
                    || driver.display_profile_key(d).as_deref() == Some(args[3].as_str()))
        };
        ensure!(identity(&display), "显示器已更换");
        let timing: Timing = serde_json::from_str(&args[4])?;
        let previous = driver
            .custom_timings(&display)?
            .into_iter()
            .find(|v| v.key() == timing.key());
        driver.create_timing(&display, timing)?;
        let keep = await_confirmation();
        if !keep {
            let connected = driver
                .display_connections(adapter)?
                .into_iter()
                .find(identity)
                .context("原显示器已断开")?;
            if let Some(previous) = previous {
                driver.write_timing(&connected, previous)?;
            } else {
                driver.delete_timing(&connected, timing)?;
            }
        }
        return Ok(keep);
    }
    let original = driver.current_mode(&display)?;
    let (width, height, rate, angle) = (
        args[3].parse::<i32>()?,
        args[4].parse::<i32>()?,
        args[5].parse::<u32>()?,
        args[6].parse::<i32>()?,
    );
    let valid_mode =
        |m: &Mode| m.width == width && m.height == height && m.refresh.to_bits() == rate;
    ensure!(
        valid_mode(&original) || driver.possible_modes(&display)?.iter().any(valid_mode),
        "显示模式不受支持"
    );
    ensure!(matches!(angle, 0 | 90 | 180 | 270), "显示方向不受支持");
    let mut target = original;
    target.width = width;
    target.height = height;
    target.refresh = f32::from_bits(rate);
    target.orientation = angle;
    driver.set_mode(&display, &target)?;
    let keep = await_confirmation();
    if !keep {
        ensure!(
            driver
                .display_connections(adapter)?
                .iter()
                .any(|d| d.logical == logical && d.name == display.name),
            "原显示器已断开"
        );
        driver.set_mode(&display, &original)?;
    }
    Ok(keep)
}

fn await_confirmation() -> bool {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut byte = [0u8; 1];
        let keep = io::stdin().read_exact(&mut byte).is_ok() && byte[0] == b'K';
        let _ = tx.send(keep);
    });
    let ready = writeln!(io::stdout(), "READY")
        .and_then(|_| io::stdout().flush())
        .is_ok();
    ready && rx.recv_timeout(TIMEOUT).unwrap_or(false)
}

pub fn worker() -> i32 {
    match run() {
        Ok(true) => 0,
        Ok(false) => 1,
        Err(err) => {
            let _ = writeln!(io::stdout(), "{err:#}");
            2
        }
    }
}
