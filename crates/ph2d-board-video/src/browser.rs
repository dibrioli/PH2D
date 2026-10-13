//! O navegador da PESSOA (família Chrome) sem ecrã, conduzido pelo CDP num TUBO (`--remote-debugging-pipe`:
//! nenhuma porta TCP que outro processo local pudesse usar). Fora do Windows lança-se por
//! `sh -c 'exec … 3<&0 4>&1'`: o tubo do PH2D é a entrada e a saída do `sh`, e o `sh` dá-lho nos
//! descritores 3 e 4 que o Chrome lê — sem `unsafe` no PH2D. No Windows (W21) os dois tubos são handles
//! herdados que o Chrome recebe por `--remote-debugging-io-pipes=<leitura>,<escrita>`
//! (`ph2d-win-pipe-spawn`, o único `unsafe`). Quando o PH2D morre o tubo fecha e o navegador sai sozinho
//! (medido no Linux: 0,05 s, `youtube_motor/sonda_tubo.py`).

use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

/// Os nomes da família Chrome no `PATH`, pela ordem em que se procuram.
const NAMES: [&str; 12] = [
    "google-chrome-stable",
    "google-chrome",
    "chromium",
    "chromium-browser",
    "microsoft-edge-stable",
    "microsoft-edge",
    "brave-browser",
    "brave",
    "vivaldi-stable",
    "vivaldi",
    "opera",
    "thorium-browser",
];

/// Os caminhos fixos do macOS.
const MAC_APPS: [&str; 4] = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
];

/// Os sítios fixos do Windows, sob `%ProgramFiles%`, `%ProgramFiles(x86)%` e `%LocalAppData%` (a
/// instalação por utilizador), pela ordem dos [`NAMES`]. O Edge vem com o Windows. Cada uma: as pastas e o
/// programa (o ficheiro é `<programa>.exe`).
const WINDOWS_APPS: [(&[&str], &str); 5] = [
    (&["Google", "Chrome", "Application"], "chrome"),
    (&["Chromium", "Application"], "chrome"),
    (&["Microsoft", "Edge", "Application"], "msedge"),
    (&["BraveSoftware", "Brave-Browser", "Application"], "brave"),
    (&["Vivaldi", "Application"], "vivaldi"),
];

/// As pastas-raiz do Windows onde os navegadores se instalam.
const WINDOWS_ROOTS: [&str; 3] = ["ProgramFiles", "ProgramFiles(x86)", "LocalAppData"];

/// O navegador da família Chrome desta máquina: `PH2D_BROWSER`, depois o `PATH`, depois as apps do macOS
/// ou os sítios fixos do Windows. `None` = não há (o vídeo mostra a miniatura e «Abrir no navegador»).
#[must_use]
pub fn find() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("PH2D_BROWSER").map(PathBuf::from) {
        return p.is_file().then_some(p);
    }
    let path = std::env::var_os("PATH")?;
    for name in NAMES {
        for dir in std::env::split_paths(&path) {
            let p = dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
            if p.is_file() {
                return Some(p);
            }
        }
    }
    if cfg!(windows) {
        return windows_candidates(|k| std::env::var_os(k)).find(|p| p.is_file());
    }
    MAC_APPS.iter().map(PathBuf::from).find(|p| p.is_file())
}

/// Cada aplicação de [`WINDOWS_APPS`] sob cada raiz de [`WINDOWS_ROOTS`] que `env` conhece.
pub(crate) fn windows_candidates(
    env: impl Fn(&str) -> Option<std::ffi::OsString>,
) -> impl Iterator<Item = PathBuf> {
    let roots: Vec<PathBuf> = WINDOWS_ROOTS
        .iter()
        .filter_map(|k| env(k).map(PathBuf::from))
        .collect();
    WINDOWS_APPS.iter().flat_map(move |(dirs, exe)| {
        roots
            .iter()
            .map(|r| {
                let dir = dirs.iter().fold(r.clone(), |p, c| p.join(c));
                dir.join(exe).with_extension("exe")
            })
            .collect::<Vec<_>>()
    })
}

/// Uma mensagem que o navegador mandou.
pub(crate) type Incoming = Value;

/// O lado de escrita do tubo (o que o navegador lê).
#[cfg(not(windows))]
pub(crate) type PipeOut = std::process::ChildStdin;
#[cfg(windows)]
pub(crate) type PipeOut = std::fs::File;

#[cfg(not(windows))]
type Process = std::process::Child;
#[cfg(windows)]
type Process = ph2d_win_pipe_spawn::Child;

/// O processo e o lado de escrita do tubo; quem lê é um fio ([`Browser::spawn`] entrega as mensagens).
pub(crate) struct Browser {
    child: Process,
    stdin: Arc<Mutex<PipeOut>>,
    profile: PathBuf,
    next_id: u64,
}

impl Browser {
    /// Lança `exe` sem ecrã num perfil temporário; cada mensagem que ele manda vai para `on_message` (no
    /// fio de leitura).
    pub(crate) fn spawn(
        exe: &Path,
        on_message: impl FnMut(Incoming) + Send + 'static,
    ) -> std::io::Result<Self> {
        sweep_stale_profiles(&std::env::temp_dir());
        let profile =
            std::env::temp_dir().join(format!("ph2d-cdp-{}-{}", std::process::id(), unique()));
        std::fs::create_dir_all(&profile)?;
        let (child, stdin, stdout) = launch(exe, &profile)?;
        std::thread::Builder::new()
            .name("ph2d-cdp-ler".into())
            .spawn(move || read_messages(stdout, on_message))?;
        Ok(Self {
            child,
            stdin: Arc::new(Mutex::new(stdin)),
            profile,
            next_id: 0,
        })
    }

    /// O lado de escrita (o fio de leitura também responde por ele: o `ack` de cada quadro).
    pub(crate) fn writer(&self) -> Arc<Mutex<PipeOut>> {
        self.stdin.clone()
    }

    /// Manda `method` (numa sessão, se `session`); devolve o id do pedido.
    pub(crate) fn send(&mut self, method: &str, params: Value, session: Option<&str>) -> u64 {
        self.next_id += 1;
        send_on(&self.stdin, self.next_id, method, params, session);
        self.next_id
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.profile);
    }
}

/// Os argumentos do navegador; os descritores ou handles do tubo são de cada SO ([`launch`]).
fn args(profile: &Path) -> Vec<std::ffi::OsString> {
    let mut dir = std::ffi::OsString::from("--user-data-dir=");
    dir.push(profile);
    let mut a: Vec<std::ffi::OsString> =
        vec!["--headless".into(), "--remote-debugging-pipe".into(), dir];
    a.extend(
        [
            "--no-first-run",
            "--no-default-browser-check",
            "--autoplay-policy=no-user-gesture-required",
            "--hide-scrollbars",
        ]
        .map(Into::into),
    );
    #[cfg(test)]
    a.extend(test_args());
    a.push("about:blank".into());
    a
}

/// Só nos testes: argumentos a mais (`PH2D_TEST_BROWSER_ARGS`, separados por espaço) — o `--no-sandbox` que
/// o Chrome pede sob o Wine. O produto nunca os lê.
#[cfg(test)]
fn test_args() -> Vec<std::ffi::OsString> {
    std::env::var("PH2D_TEST_BROWSER_ARGS")
        .unwrap_or_default()
        .split_whitespace()
        .map(Into::into)
        .collect()
}

/// Fora do Windows: o `sh` dá o tubo ao navegador nos descritores 3 e 4.
#[cfg(not(windows))]
fn launch(
    exe: &Path,
    profile: &Path,
) -> std::io::Result<(Process, PipeOut, impl Read + Send + 'static)> {
    use std::process::{Command, Stdio};
    let q = |p: &std::ffi::OsStr| format!("'{}'", p.to_string_lossy().replace('\'', r"'\''"));
    let argv: Vec<String> = args(profile).iter().map(|a| q(a)).collect();
    let cmd = format!(
        "exec {} {} 3<&0 4>&1 </dev/null >/dev/null 2>/dev/null",
        q(exe.as_os_str()),
        argv.join(" ")
    );
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("sem stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("sem stdout"))?;
    Ok((child, stdin, stdout))
}

/// No Windows: dois handles herdados, os valores no `--remote-debugging-io-pipes` (leitura do navegador
/// primeiro). Nenhuma porta TCP — ADR-0185 §W19.8.
#[cfg(windows)]
fn launch(
    exe: &Path,
    profile: &Path,
) -> std::io::Result<(Process, PipeOut, impl Read + Send + 'static)> {
    let s = ph2d_win_pipe_spawn::spawn(exe, |read, write| {
        let mut a = args(profile);
        a.insert(
            2,
            format!("--remote-debugging-io-pipes={read},{write}").into(),
        );
        a
    })?;
    Ok((s.child, s.to_child, s.from_child))
}

/// Escreve uma mensagem no tubo (JSON + NUL). Um tubo partido (o navegador saiu) não pára ninguém.
pub(crate) fn send_on(
    stdin: &Mutex<PipeOut>,
    id: u64,
    method: &str,
    params: Value,
    session: Option<&str>,
) {
    let mut m = json!({ "id": id, "method": method, "params": params });
    if let Some(s) = session {
        m["sessionId"] = json!(s);
    }
    let mut bytes = m.to_string().into_bytes();
    bytes.push(0);
    if let Ok(mut w) = stdin.lock() {
        let _ = w.write_all(&bytes).and_then(|()| w.flush());
    }
}

fn read_messages(stdout: impl Read, mut on_message: impl FnMut(Incoming)) {
    let mut r = BufReader::new(stdout);
    let mut buf = Vec::new();
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let n = match r.read(&mut chunk) {
            Ok(0) | Err(_) => return,
            Ok(n) => n,
        };
        buf.extend_from_slice(&chunk[..n]);
        while let Some(end) = buf.iter().position(|&b| b == 0) {
            let msg: Vec<u8> = buf.drain(..=end).collect();
            if let Ok(v) = serde_json::from_slice::<Value>(&msg[..msg.len() - 1]) {
                on_message(v);
            }
        }
    }
}

/// Os perfis que um PH2D que já MORREU deixou em `dir` (`ph2d-cdp-<pid>-…`, o pid sem processo): um
/// fecho abrupto fecha o navegador (o tubo fecha) mas não apaga a pasta. Onde há `/proc`, e no Windows.
pub(crate) fn sweep_stale_profiles(dir: &Path) {
    #[cfg(not(windows))]
    if !Path::new("/proc/self").exists() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let name = e.file_name();
        let Some(pid) = name
            .to_str()
            .and_then(|n| n.strip_prefix("ph2d-cdp-"))
            .and_then(|r| r.split('-').next())
            .and_then(|p| p.parse::<u32>().ok())
        else {
            continue;
        };
        if pid != std::process::id() && !alive(pid) {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

#[cfg(not(windows))]
fn alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(windows)]
fn alive(pid: u32) -> bool {
    ph2d_win_pipe_spawn::is_alive(pid)
}

fn unique() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos())
}
