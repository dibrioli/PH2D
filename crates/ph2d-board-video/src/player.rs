//! ⭐ O TOCADOR de um vídeo no navegador da pessoa: um fio de controlo abre uma aba sem ecrã com o tocador
//! oficial (numa página de origem `https://ph2d.local/`, servida pelo próprio CDP — o YouTube pede o
//! `Referer`), pede o screencast e guarda o quadro mais novo; o rato e o teclado do quadro vão por
//! `Input.dispatch*`. Quem desenha só lê o quadro mais novo ([`Player::take_frame`]) — nada bloqueia o fio
//! da janela. Medido (`youtube_motor/saidas/chromium_*`): 29,97 q/s a 560×315, 0,15 núcleo, clique → quadro
//! em 7–28 ms.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use ph2d_board_model::Video;
use serde_json::{Value, json};

use crate::browser::{Browser, send_on};
use crate::playback::{Phase, Playback};

/// A origem da página que leva o tocador (o `Referer` que o YouTube vê).
const ORIGIN: &str = "https://ph2d.local/";
/// O prazo de cada passo do arranque (lançar, abrir a aba, ligar-se a ela).
const STEP: Duration = Duration::from_secs(10);
/// Os ids dos pedidos que o fio de leitura manda (o `ack` de cada quadro, a página) — longe dos do controlo
/// e DENTRO de um inteiro de 32 bits: o CDP recusa outro id («Message must have integer 'id' property»,
/// medido com `1 << 40`: a página nunca se servia e o navegador parava ao 2.º quadro).
pub(crate) const READER_IDS: u64 = 1 << 30;
pub(crate) const READER_IDS_END: u64 = i32::MAX as u64;

/// Um quadro do tocador: RGBA de 8 bits, alfa reto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub size: [u32; 2],
    pub rgba: Vec<u8>,
}

/// Em que pé está o tocador.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlayerState {
    Starting,
    Playing,
    /// Não há navegador da família Chrome nesta máquina.
    NoBrowser,
    Failed(String),
}

/// Um botão do rato.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    None,
    Left,
    Middle,
    Right,
}

/// Um gesto do rato, em píxeis do TOCADOR (`0..size`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mouse {
    Move {
        x: f64,
        y: f64,
    },
    Down {
        x: f64,
        y: f64,
        button: Button,
        clicks: u32,
    },
    Up {
        x: f64,
        y: f64,
        button: Button,
        clicks: u32,
    },
    Wheel {
        x: f64,
        y: f64,
        dx: f64,
        dy: f64,
    },
}

/// Uma tecla (os nomes do DOM: `key` = `" "`, `code` = `"Space"`, o código virtual do Windows).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    pub down: bool,
    pub key: String,
    pub code: String,
    pub vk: u32,
    pub text: Option<String>,
}

enum Cmd {
    Load(Video),
    /// Uma ordem da API do tocador (`pauseVideo`, `seekTo`…) e os argumentos dela (uma lista JSON).
    Call(&'static str, Value),
    Resize([u32; 2]),
    Mouse(Mouse),
    Key(Key),
}

#[derive(Default)]
struct Shared {
    state: Mutex<Option<PlayerState>>,
    frame: Mutex<Option<Frame>>,
    seq: AtomicU64,
    html: Mutex<String>,
    playback: Mutex<Playback>,
    /// Os links que o tocador quis abrir numa aba nova (o canal, o título, «YouTube»): fecham-se lá e o
    /// app abre-os no navegador do sistema.
    outside: Mutex<Vec<String>>,
    /// A aba do tocador.
    target: OnceLock<String>,
    /// As abas novas já fechadas.
    popups: Mutex<BTreeSet<String>>,
}

/// O tocador (o navegador fecha quando ele cai).
pub struct Player {
    cmd: Sender<Cmd>,
    shared: Arc<Shared>,
    seen: u64,
}

impl Player {
    /// Arranca o tocador de `video` a `size` píxeis, num fio (o navegador leva meio segundo a abrir).
    #[must_use]
    pub fn start(video: &Video, size: [u32; 2]) -> Self {
        let shared = Arc::new(Shared::default());
        *lock(&shared.state) = Some(PlayerState::Starting);
        let (tx, rx) = channel();
        let (s, v) = (shared.clone(), video.clone());
        let ok = std::thread::Builder::new()
            .name("ph2d-cdp-tocador".into())
            .spawn(move || {
                let end = run(&s, &v, size, &rx);
                *lock(&s.state) = Some(end);
            })
            .is_ok();
        if !ok {
            *lock(&shared.state) = Some(PlayerState::Failed("thread".into()));
        }
        Self {
            cmd: tx,
            shared,
            seen: 0,
        }
    }

    #[must_use]
    pub fn state(&self) -> PlayerState {
        lock(&self.shared.state)
            .clone()
            .unwrap_or(PlayerState::Starting)
    }

    /// O quadro mais novo que ainda não se leu.
    pub fn take_frame(&mut self) -> Option<Frame> {
        let seq = self.shared.seq.load(Ordering::Acquire);
        if seq == self.seen {
            return None;
        }
        self.seen = seq;
        lock(&self.shared.frame).clone()
    }

    /// Pausa (o vídeo fica no quadro em que parou).
    pub fn pause(&self) {
        self.want(Phase::Paused);
        self.call("pauseVideo", json!([]));
    }

    pub fn play(&self) {
        self.want(Phase::Playing);
        self.call("playVideo", json!([]));
    }

    /// Salta para `secs` segundos.
    pub fn seek(&self, secs: f64) {
        let secs = secs.max(0.0);
        {
            let mut p = lock(&self.shared.playback);
            p.time = secs;
            p.read_at = Some(Instant::now());
        }
        self.call("seekTo", json!([secs, true]));
    }

    pub fn set_muted(&self, muted: bool) {
        lock(&self.shared.playback).muted = muted;
        self.call(if muted { "mute" } else { "unMute" }, json!([]));
    }

    /// O volume `0..=100` (e tira o mudo, como o tocador faz).
    pub fn set_volume(&self, v: u8) {
        let v = v.min(100);
        {
            let mut p = lock(&self.shared.playback);
            p.volume = v;
            p.muted = false;
        }
        self.call("setVolume", json!([v]));
        self.call("unMute", json!([]));
    }

    /// O estado do tocador (o tempo, a duração, a fase, o volume).
    #[must_use]
    pub fn playback(&self) -> Playback {
        lock(&self.shared.playback).clone()
    }

    /// Os links que o tocador quis abrir (o app abre-os no navegador do sistema).
    pub fn take_outside(&self) -> Vec<String> {
        std::mem::take(&mut *lock(&self.shared.outside))
    }

    fn want(&self, p: Phase) {
        lock(&self.shared.playback).want = Some((p, Instant::now()));
    }

    fn call(&self, func: &'static str, args: Value) {
        let _ = self.cmd.send(Cmd::Call(func, args));
    }

    pub fn load(&self, video: &Video) {
        let _ = self.cmd.send(Cmd::Load(video.clone()));
    }

    pub fn resize(&self, size: [u32; 2]) {
        let _ = self.cmd.send(Cmd::Resize(size));
    }

    pub fn mouse(&self, m: Mouse) {
        let _ = self.cmd.send(Cmd::Mouse(m));
    }

    pub fn key(&self, k: Key) {
        let _ = self.cmd.send(Cmd::Key(k));
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
pub(crate) fn page_for_test(video: &Video) -> String {
    page(video)
}

/// O nome do `Runtime.addBinding` por onde as mensagens do tocador chegam ao PH2D.
const BINDING: &str = "ph2dState";

/// A página que leva o tocador oficial a ocupar a aba inteira, e que lhe fala pela API oficial: pede-lhe
/// o estado (`listening`, até ele responder), ouve o `onStateChange` e passa cada mensagem ao PH2D.
fn page(video: &Video) -> String {
    format!(
        "<!doctype html><html><body style=\"margin:0;background:#000;overflow:hidden\">\
         <iframe style=\"border:0;width:100vw;height:100vh\" allow=\"autoplay; encrypted-media\" \
         referrerpolicy=\"strict-origin-when-cross-origin\" src=\"{src}\"></iframe><script>\
         const f=document.querySelector('iframe');\
         const say=(o)=>f.contentWindow.postMessage(JSON.stringify(Object.assign(o,{{id:1,channel:'widget'}})),'*');\
         const ask=setInterval(()=>say({{event:'listening'}}),250);\
         addEventListener('message',(e)=>{{if(typeof e.data!=='string'||!e.origin.includes('youtube'))return;\
         clearInterval(ask);if(self.{BINDING})self.{BINDING}(e.data);\
         if(e.data.includes('\"onReady\"'))say({{event:'command',func:'addEventListener',args:['onStateChange']}});}});\
         self.ph2dCall=(func,args)=>say({{event:'command',func,args}});\
         </script></body></html>",
        src = ph2d_board_embed::player_url(video)
    )
}

/// O fio de controlo: abre, liga, e serve os comandos até o [`Player`] cair. Devolve o estado final.
fn run(s: &Arc<Shared>, video: &Video, size: [u32; 2], rx: &Receiver<Cmd>) -> PlayerState {
    let Some(exe) = crate::browser::find() else {
        return PlayerState::NoBrowser;
    };
    *lock(&s.html) = page(video);
    let (resp_tx, resp_rx) = channel::<Value>();
    let writer: Arc<OnceLock<Arc<Mutex<std::process::ChildStdin>>>> = Arc::new(OnceLock::new());
    let reader_ids = Arc::new(AtomicU64::new(READER_IDS));
    let (sr, w2) = (s.clone(), writer.clone());
    let browser = Browser::spawn(&exe, move |m| {
        on_message(&sr, &w2, &reader_ids, &resp_tx, m)
    });
    let mut b = match browser {
        Ok(b) => b,
        Err(e) => return PlayerState::Failed(e.to_string()),
    };
    let _ = writer.set(b.writer());
    let call = |b: &mut Browser, method: &str, params: Value, sess: Option<&str>| {
        let id = b.send(method, params, sess);
        loop {
            match resp_rx.recv_timeout(STEP) {
                Ok(v) if v["id"].as_u64() == Some(id) => return Some(v),
                Ok(_) => {}
                Err(_) => return None,
            }
        }
    };
    let Some(t) = call(
        &mut b,
        "Target.createTarget",
        json!({ "url": "about:blank" }),
        None,
    ) else {
        return PlayerState::Failed("createTarget".into());
    };
    let target = t["result"]["targetId"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let Some(a) = call(
        &mut b,
        "Target.attachToTarget",
        json!({ "targetId": target, "flatten": true }),
        None,
    ) else {
        return PlayerState::Failed("attachToTarget".into());
    };
    let session = a["result"]["sessionId"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let sess = Some(session.as_str());
    let _ = s.target.set(target);
    // As abas novas que o tocador abre (o canal, o título, «YouTube») — medido: a aba nova ficava à frente
    // e o tocador deixava de pintar (preto).
    if call(
        &mut b,
        "Target.setDiscoverTargets",
        json!({ "discover": true }),
        None,
    )
    .is_none()
    {
        return PlayerState::Failed("setDiscoverTargets".into());
    }
    let pattern = format!("{ORIGIN}*");
    for (m, p) in [
        ("Page.enable", json!({})),
        ("Runtime.enable", json!({})),
        ("Runtime.addBinding", json!({ "name": BINDING })),
        (
            "Fetch.enable",
            json!({ "patterns": [{ "urlPattern": pattern }] }),
        ),
    ] {
        if call(&mut b, m, p, sess).is_none() {
            return PlayerState::Failed(m.into());
        }
    }
    let size = metrics(&mut b, sess, size);
    b.send("Page.navigate", json!({ "url": ORIGIN }), sess);
    screencast(&mut b, sess, size);
    *lock(&s.state) = Some(PlayerState::Playing);
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Cmd::Load(v)) => {
                *lock(&s.html) = page(&v);
                b.send("Page.navigate", json!({ "url": ORIGIN }), sess);
            }
            Ok(Cmd::Call(func, args)) => {
                let js = format!("self.ph2dCall?.('{func}', {args})");
                b.send("Runtime.evaluate", json!({ "expression": js }), sess);
            }
            Ok(Cmd::Resize(sz)) => {
                let sz = metrics(&mut b, sess, sz);
                b.send("Page.stopScreencast", json!({}), sess);
                screencast(&mut b, sess, sz);
            }
            Ok(Cmd::Mouse(m)) => {
                let (method, p) = mouse_params(m);
                b.send(method, p, sess);
            }
            Ok(Cmd::Key(k)) => {
                let mut p = json!({
                    "type": if k.down { "keyDown" } else { "keyUp" },
                    "key": k.key, "code": k.code,
                    "windowsVirtualKeyCode": k.vk, "nativeVirtualKeyCode": k.vk,
                });
                if let (true, Some(t)) = (k.down, k.text) {
                    p["text"] = json!(t);
                }
                b.send("Input.dispatchKeyEvent", p, sess);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return PlayerState::Starting,
        }
    }
}

/// O tamanho da aba E do «ecrã» dela: sem o ecrã, o navegador sem ecrã imagina um de 800×600 e um ecrã
/// inteiro manda quadros 4:3 (W19, medido: 420×315 numa caixa 560×315).
fn metrics(b: &mut Browser, sess: Option<&str>, size: [u32; 2]) -> [u32; 2] {
    let size = size.map(|v| v.max(1));
    b.send(
        "Emulation.setDeviceMetricsOverride",
        json!({ "width": size[0], "height": size[1], "deviceScaleFactor": 1, "mobile": false,
                "screenWidth": size[0], "screenHeight": size[1] }),
        sess,
    );
    size
}

fn screencast(b: &mut Browser, sess: Option<&str>, size: [u32; 2]) {
    b.send(
        "Page.startScreencast",
        json!({ "format": "jpeg", "quality": 85, "maxWidth": size[0], "maxHeight": size[1], "everyNthFrame": 1 }),
        sess,
    );
}

fn mouse_params(m: Mouse) -> (&'static str, Value) {
    let name = |b: Button| match b {
        Button::None => "none",
        Button::Left => "left",
        Button::Middle => "middle",
        Button::Right => "right",
    };
    let p = match m {
        Mouse::Move { x, y } => json!({ "type": "mouseMoved", "x": x, "y": y }),
        Mouse::Down {
            x,
            y,
            button,
            clicks,
        } => {
            json!({ "type": "mousePressed", "x": x, "y": y, "button": name(button), "clickCount": clicks })
        }
        Mouse::Up {
            x,
            y,
            button,
            clicks,
        } => {
            json!({ "type": "mouseReleased", "x": x, "y": y, "button": name(button), "clickCount": clicks })
        }
        Mouse::Wheel { x, y, dx, dy } => {
            json!({ "type": "mouseWheel", "x": x, "y": y, "deltaX": dx, "deltaY": dy })
        }
    };
    ("Input.dispatchMouseEvent", p)
}

/// O fio de leitura: respostas para o controlo; cada quadro descodifica-se AQUI (nunca no fio da janela)
/// e responde-se-lhe com o `ack` (sem ele o navegador pára de mandar); a página do tocador serve-se ao
/// pedido.
/// O id seguinte do leitor: dá a volta DENTRO de `READER_IDS..READER_IDS_END` (o contador começa em
/// `READER_IDS`), distinto do anterior e sempre um inteiro de 32 bits.
pub(crate) fn next_reader_id(ids: &AtomicU64) -> u64 {
    let n = ids
        .fetch_add(1, Ordering::Relaxed)
        .saturating_sub(READER_IDS);
    READER_IDS + n % (READER_IDS_END - READER_IDS)
}

fn on_message(
    s: &Shared,
    writer: &OnceLock<Arc<Mutex<std::process::ChildStdin>>>,
    ids: &AtomicU64,
    resp: &Sender<Value>,
    m: Value,
) {
    if m.get("id").is_some() {
        let _ = resp.send(m);
        return;
    }
    let Some(w) = writer.get() else { return };
    let sess = m["sessionId"].as_str();
    let p = &m["params"];
    match m["method"].as_str() {
        Some("Runtime.bindingCalled") if p["name"] == BINDING => {
            if let Some(v) = p["payload"]
                .as_str()
                .and_then(|t| serde_json::from_str::<Value>(t).ok())
            {
                lock(&s.playback).apply(&v, Instant::now());
            }
        }
        Some("Target.targetCreated" | "Target.targetInfoChanged") => {
            let info = &p["targetInfo"];
            if let Some(url) = popup_url(s, info) {
                let ours = s.target.get().cloned().unwrap_or_default();
                send_on(w, next_reader_id(ids), "Target.closeTarget", json!({ "targetId": info["targetId"] }), None);
                send_on(w, next_reader_id(ids), "Target.activateTarget", json!({ "targetId": ours }), None);
                lock(&s.outside).push(url);
            }
        }
        Some("Page.screencastFrame") => {
            let id = next_reader_id(ids);
            send_on(
                w,
                id,
                "Page.screencastFrameAck",
                json!({ "sessionId": p["sessionId"] }),
                sess,
            );
            if let Some(f) = p["data"]
                .as_str()
                .and_then(|d| decode_jpeg(&STANDARD.decode(d).ok()?))
            {
                *lock(&s.frame) = Some(f);
                s.seq.fetch_add(1, Ordering::Release);
            }
        }
        Some("Fetch.requestPaused") => {
            let id = next_reader_id(ids);
            let body = STANDARD.encode(lock(&s.html).as_bytes());
            send_on(
                w,
                id,
                "Fetch.fulfillRequest",
                json!({
                    "requestId": p["requestId"], "responseCode": 200,
                    "responseHeaders": [{ "name": "content-type", "value": "text/html; charset=utf-8" }],
                    "body": body,
                }),
                sess,
            );
        }
        _ => {}
    }
}

/// Uma aba nova (não a do tocador) que já diz o endereço: o endereço a abrir fora, UMA vez (o
/// `targetCreated` e os `targetInfoChanged` repetem-no; sem endereço ainda, `about:blank`, espera-se).
fn popup_url(s: &Shared, info: &Value) -> Option<String> {
    let id = info["targetId"].as_str()?;
    let url = info["url"].as_str().unwrap_or_default();
    if info["type"] != "page" || s.target.get().is_none_or(|t| t == id) || !url.starts_with("http") {
        return None;
    }
    lock(&s.popups).insert(id.to_owned()).then(|| url.to_owned())
}

pub(crate) fn decode_jpeg(bytes: &[u8]) -> Option<Frame> {
    use zune_core::bytestream::ZCursor;
    use zune_core::colorspace::ColorSpace;
    use zune_core::options::DecoderOptions;
    let opts = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut d = zune_jpeg::JpegDecoder::new_with_options(ZCursor::new(bytes), opts);
    let rgba = d.decode().ok()?;
    let info = d.info()?;
    Some(Frame {
        size: [u32::from(info.width), u32::from(info.height)],
        rgba,
    })
}
