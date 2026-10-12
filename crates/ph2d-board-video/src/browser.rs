//! O navegador da PESSOA (família Chrome) sem ecrã, conduzido pelo CDP num TUBO (`--remote-debugging-pipe`:
//! nenhuma porta TCP que outro processo local pudesse usar). Lança-se por `sh -c 'exec … 3<&0 4>&1'`: o tubo
//! do PH2D é a entrada e a saída do `sh`, e o `sh` dá-lho nos descritores 3 e 4 que o Chrome lê — sem
//! `unsafe` no PH2D. Quando o PH2D morre o tubo fecha e o navegador sai sozinho (medido: 0,05 s,
//! `youtube_motor/sonda_tubo.py`).

use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
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

/// O navegador da família Chrome desta máquina: `PH2D_BROWSER`, depois o `PATH`, depois as apps do macOS.
/// `None` = não há (o vídeo mostra a miniatura e «Abrir no navegador»).
#[must_use]
pub fn find() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("PH2D_BROWSER").map(PathBuf::from) {
        return p.is_file().then_some(p);
    }
    let path = std::env::var_os("PATH")?;
    for name in NAMES {
        for dir in std::env::split_paths(&path) {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    MAC_APPS.iter().map(PathBuf::from).find(|p| p.is_file())
}

/// Uma mensagem que o navegador mandou.
pub(crate) type Incoming = Value;

/// O processo e o lado de escrita do tubo; quem lê é um fio ([`Browser::spawn`] entrega as mensagens).
pub(crate) struct Browser {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
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
        let profile =
            std::env::temp_dir().join(format!("ph2d-cdp-{}-{}", std::process::id(), unique()));
        std::fs::create_dir_all(&profile)?;
        let q = |p: &Path| format!("'{}'", p.display().to_string().replace('\'', r"'\''"));
        let cmd = format!(
            "exec {} --headless --remote-debugging-pipe --user-data-dir={} --no-first-run \
             --no-default-browser-check --autoplay-policy=no-user-gesture-required \
             --hide-scrollbars about:blank 3<&0 4>&1 </dev/null >/dev/null 2>/dev/null",
            q(exe),
            q(&profile)
        );
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| std::io::Error::other("sem stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| std::io::Error::other("sem stdout"))?;
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
    pub(crate) fn writer(&self) -> Arc<Mutex<ChildStdin>> {
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

/// Escreve uma mensagem no tubo (JSON + NUL). Um tubo partido (o navegador saiu) não pára ninguém.
pub(crate) fn send_on(
    stdin: &Mutex<ChildStdin>,
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

fn unique() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos())
}
