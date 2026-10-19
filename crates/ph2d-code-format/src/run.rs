//! One StyLua run: text in through stdin, the formatted text out through stdout (I0d: never a partial
//! text — exit `2` with an empty stdout on any refusal).

use crate::STYLUA;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

/// The formatting rules (`--config-path`): the editor's indentation, every other StyLua default
/// written out.
pub const CONFIG: &str = include_str!("../ph2d-format.toml");

/// Why a text came back unformatted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The text does not parse (StyLua's own message).
    Syntax(String),
    /// `--verify`: the output would not mean the same as the input (2.5.2 rewrites `{ read T }` as
    /// `{ readT }`, I0d §Achados 4).
    WouldChangeCode,
    /// The program could not run, or said something else.
    Failed(String),
}

/// A formatter that is not the program (the tests' fakes).
pub type FormatFn = Arc<dyn Fn(&str) -> Result<String, Refusal> + Send + Sync>;

#[derive(Clone)]
pub enum Program {
    Stylua { bin: PathBuf, config: PathBuf },
    Custom(FormatFn),
}

impl std::fmt::Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stylua { bin, config } => f
                .debug_struct("Stylua")
                .field("bin", bin)
                .field("config", config)
                .finish(),
            Self::Custom(_) => f.write_str("Custom"),
        }
    }
}

impl Program {
    /// The pinned StyLua ([`STYLUA`]`.resolve()`), with the rules written where it can read them.
    pub fn pinned() -> Result<Self, String> {
        let (bin, _) = STYLUA.resolve().map_err(|e| e.to_string())?;
        Self::stylua(bin)
    }

    /// This StyLua binary with our rules.
    pub fn stylua(bin: PathBuf) -> Result<Self, String> {
        let config = config_file().map_err(|e| format!("ph2d-format.toml: {e}"))?;
        Ok(Self::Stylua { bin, config })
    }

    /// Formats `text` (blocking: call it off the UI thread).
    pub fn format(&self, text: &str) -> Result<String, Refusal> {
        match self {
            Self::Custom(f) => f(text),
            Self::Stylua { bin, config } => run(bin, config, text),
        }
    }
}

/// The rules in `<cache>/ph2d/stylua/<version>/ph2d-format-<hash>.toml` — written once per content
/// (temp file + rename, so two processes never read half a file). Never named `stylua.toml`.
fn config_file() -> std::io::Result<PathBuf> {
    let dir = STYLUA
        .cache_dir()
        .unwrap_or_else(|| std::env::temp_dir().join("ph2d-stylua"));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!(
        "ph2d-format-{:016x}.toml",
        fnv64(CONFIG.as_bytes())
    ));
    if std::fs::read(&path).ok().as_deref() != Some(CONFIG.as_bytes()) {
        // Unique per write: two threads of one process must not rename each other's file away.
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tmp = dir.join(format!("ph2d-format-{}-{n}.tmp", std::process::id()));
        std::fs::write(&tmp, CONFIG)?;
        std::fs::rename(&tmp, &path)?;
    }
    Ok(path)
}

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn run(bin: &Path, config: &Path, text: &str) -> Result<String, Refusal> {
    let mut cmd = Command::new(bin);
    cmd.arg("--config-path")
        .arg(config)
        .args(["--verify", "--color", "never", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| Refusal::Failed(format!("{}: {e}", bin.display())))?;
    // stdin on its own thread: a text bigger than the pipe would block against a full stdout.
    let mut stdin = child.stdin.take().expect("piped stdin");
    let input = text.to_owned();
    let feeder = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let out = child
        .wait_with_output()
        .map_err(|e| Refusal::Failed(e.to_string()))?;
    let fed = feeder.join();
    if out.status.success() {
        if let Ok(Err(e)) = fed {
            return Err(Refusal::Failed(e.to_string()));
        }
        return String::from_utf8(out.stdout).map_err(|e| Refusal::Failed(e.to_string()));
    }
    Err(classify(&String::from_utf8_lossy(&out.stderr)))
}

/// What a failed run's stderr means (the messages of the pinned version — gated against the real
/// binary in `tests/it/stylua.rs`).
pub fn classify(stderr: &str) -> Refusal {
    if stderr.contains("Output AST may be different") {
        return Refusal::WouldChangeCode;
    }
    if let Some(at) = stderr.find("error parsing:") {
        return Refusal::Syntax(stderr[at + "error parsing:".len()..].trim().to_owned());
    }
    Refusal::Failed(stderr.trim().to_owned())
}
