//! W18 ([ADR-0185]) — **dá vida aos vídeos do quadro**, a cada quadro do app (pelo `io::service`):
//! - as miniaturas que o desenho pediu buscam-se num fio cada (com prazo, e a cache em disco);
//! - o vídeo ACTIVADO toca no navegador da pessoa ([`ph2d_board_video::Player`]): arranca, segue o tamanho
//!   dele no ecrã, recebe o rato, e os quadros dele voltam para o desenho;
//! - deixar de estar vivo (tapado, rodado, menos de 200×200 px à vista) ou desactivar PAUSA (as regras do
//!   YouTube) e, de volta, retoma só se estava a tocar; o navegador fecha um minuto depois do último vídeo;
//! - W19: os NOSSOS controles (`Videos::commands`) e o estado do tocador para eles (`Videos::status`); os
//!   links que o tocador quis abrir (o canal, o título) vão para o navegador do sistema; o tamanho do
//!   tocador só segue o do ecrã quando o zoom assenta ([`SETTLE`]);
//! - sem navegador da família Chrome: abre o link no navegador do sistema e diz porquê.
//!
//! [ADR-0185]: ../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

use std::collections::BTreeSet;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use ph2d_board_model::{ElementId, Video};
use ph2d_board_render::{
    PlayerInput, PlayerStatus, Thumb, VideoCommand, set_thumbnail, video_key,
};
use ph2d_board_video::{Button, Mouse, Player, PlayerState};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vector::StableImage;

/// O prazo de uma miniatura (a rede inteira: ligar, pedir, receber).
const THUMB_DEADLINE: Duration = Duration::from_secs(8);
/// Quanto o navegador fica aberto depois do último vídeo (voltar a um vídeo não espera meio segundo).
const LINGER: Duration = Duration::from_secs(60);
/// Quanto tempo o tamanho do vídeo no ecrã fica igual antes de o tocador o seguir. Medido
/// (`youtube_motor/saidas/w19`, variante «assenta»): 30 quadros/s e zero pausas; seguir a cada quadro também
/// não pausa, mas recomeça o screencast 60 vezes por segundo — o desenho escala o quadro entretanto.
const SETTLE: Duration = Duration::from_millis(250);

type Fetched = (String, Option<Vec<u8>>);

struct Playing {
    id: ElementId,
    video: Video,
    player: Player,
    size: [u32; 2],
    /// O tamanho que o ecrã pede e desde quando (o tocador segue-o depois de [`SETTLE`]).
    asked: Option<([u32; 2], Instant)>,
    /// Pausado pelas REGRAS (não vivo): `Some(estava a tocar)` — de volta, retoma só nesse caso.
    held: Option<bool>,
}

struct State {
    fetching: BTreeSet<String>,
    tx: Sender<Fetched>,
    rx: Receiver<Fetched>,
    playing: Option<Playing>,
    idle_since: Option<Instant>,
}

fn state() -> std::sync::MutexGuard<'static, State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(|| {
        let (tx, rx) = channel();
        Mutex::new(State {
            fetching: BTreeSet::new(),
            tx,
            rx,
            playing: None,
            idle_since: None,
        })
    })
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn image_of(f: &ph2d_board_video::Frame) -> Option<StableImage> {
    StableImage::from_rgba(Arc::new(f.rgba.clone()), f.size[0], f.size[1])
}

/// ⭐ Um quadro do app: as miniaturas e o tocador.
pub fn service(hero: &mut HeroScreen) {
    if hero.documents.take_video_ask() {
        from_clipboard(hero);
    }
    let mut st = state();
    thumbnails(hero, &mut st);
    player(hero, &mut st);
}

/// W18 — os links LARGADOS (do navegador) que são vídeos do YouTube entram no quadro em `at` (px de
/// ecrã). Devolve quantos entraram (os outros a shell diz que recusou).
pub fn drop_links(hero: &mut HeroScreen, links: &[String], at: [f32; 2]) -> usize {
    let at = [f64::from(at[0]), f64::from(at[1])];
    links
        .iter()
        .filter(|l| ph2d_editor_core::screens::hero::board_video::insert_link(hero, l, Some(at)))
        .count()
}

/// O botão «Video»: o link da área de transferência vira um vídeo no meio da vista; senão, diz como.
fn from_clipboard(hero: &mut HeroScreen) {
    let text = ph2d_app_host::clipboard::read(|c| c.get_text().ok());
    match text {
        Ok(Some(t))
            if ph2d_editor_core::screens::hero::board_video::insert_link(hero, &t, None) => {}
        Ok(_) => hero.documents.warn("board.video.copy_first"),
        Err(_) => hero.documents.warn("clipboard.no_answer"),
    }
}

fn thumbnails(hero: &mut HeroScreen, st: &mut State) {
    let vids = hero.documents.videos_mut();
    let wanted = std::mem::take(&mut vids.wanted);
    for key in wanted {
        if !st.fetching.insert(key.clone()) {
            continue;
        }
        let Some(v) = vids.videos.values().find(|v| video_key(v) == key).cloned() else {
            st.fetching.remove(&key);
            continue;
        };
        let tx = st.tx.clone();
        let ok = std::thread::Builder::new()
            .name("ph2d-video-miniatura".into())
            .spawn(move || {
                let bytes = ph2d_board_video::thumb::fetch(&v, THUMB_DEADLINE).ok();
                let _ = tx.send((key, bytes));
            })
            .is_ok();
        if !ok {
            // Sem fio: a próxima vez que o desenho pedir, tenta-se de novo.
            st.fetching.clear();
        }
    }
    let mut offline = false;
    while let Ok((key, bytes)) = st.rx.try_recv() {
        let thumb = bytes
            .as_deref()
            .and_then(ph2d_board_video::decode_jpeg)
            .and_then(|f| Some(Thumb::Ready(image_of(&f)?, f.size)));
        offline |= thumb.is_none();
        set_thumbnail(key, thumb.unwrap_or(Thumb::Missing));
    }
    if offline {
        hero.documents.warn("board.video.offline");
    }
}

fn mouse(i: &PlayerInput) -> Mouse {
    let b = |n: u8| match n {
        0 => Button::Left,
        1 => Button::Middle,
        _ => Button::Right,
    };
    match *i {
        PlayerInput::Down {
            x,
            y,
            button,
            clicks,
        } => Mouse::Down {
            x,
            y,
            button: b(button),
            clicks,
        },
        PlayerInput::Up {
            x,
            y,
            button,
            clicks,
        } => Mouse::Up {
            x,
            y,
            button: b(button),
            clicks,
        },
        PlayerInput::Move { x, y } => Mouse::Move { x, y },
    }
}

/// Pausa pelas regras (guarda se estava a tocar).
fn hold(p: &mut Playing) {
    if p.held.is_none() {
        let running = p.player.playback().shown(Instant::now()).running();
        if running {
            p.player.pause();
        }
        p.held = Some(running);
    }
}

/// O tamanho que o tocador deve ter agora: o do ecrã, depois de ele ficar [`SETTLE`] igual.
fn settled(asked: &mut Option<([u32; 2], Instant)>, have: [u32; 2], want: [u32; 2], now: Instant) -> Option<[u32; 2]> {
    if want == have {
        *asked = None;
        return None;
    }
    match *asked {
        Some((s, since)) if s == want => (now.duration_since(since) >= SETTLE).then(|| {
            *asked = None;
            want
        }),
        _ => {
            *asked = Some((want, now));
            None
        }
    }
}

fn player(hero: &mut HeroScreen, st: &mut State) {
    let now = Instant::now();
    let vids = hero.documents.videos_mut();
    let inbox = std::mem::take(&mut vids.inbox);
    let commands = std::mem::take(&mut vids.commands);
    let target = vids.active.and_then(|id| {
        let pl = vids.on_screen.get(&id)?;
        Some((id, vids.videos.get(&id)?.clone(), pl.live, pl.screen.size))
    });
    // Desactivado (ou o vídeo saiu da vista): pausa, e o navegador fica um minuto.
    let Some((id, video, live, size)) = target else {
        vids.status = None;
        if let Some(p) = st.playing.as_mut() {
            hold(p);
        }
        if vids.active.is_none() {
            vids.fullscreen = false;
            let since = *st.idle_since.get_or_insert_with(Instant::now);
            if since.elapsed() > LINGER {
                st.playing = None;
            }
        }
        return;
    };
    st.idle_since = None;
    let size = size.map(|v| v.round().max(1.0) as u32);
    match st.playing.as_mut() {
        Some(p) if p.id == id => {}
        Some(p) => {
            p.player.load(&video);
            p.player.resize(size);
            (p.id, p.video, p.size, p.held, p.asked) = (id, video.clone(), size, None, None);
        }
        // Só arranca quando pode estar vivo (senão tocaria com som até a pausa chegar).
        None if !live => return,
        None => {
            st.playing = Some(Playing {
                id,
                video: video.clone(),
                player: Player::start(&video, size),
                size,
                asked: None,
                held: None,
            });
        }
    }
    let Some(p) = st.playing.as_mut() else { return };
    match p.player.state() {
        PlayerState::NoBrowser | PlayerState::Failed(_) => {
            let no = p.player.state() == PlayerState::NoBrowser;
            st.playing = None;
            vids.active = None;
            vids.frame = None;
            vids.status = None;
            vids.fullscreen = false;
            let link = ph2d_board_embed::canonical_link(&video);
            let opened = ph2d_board_video::open_in_browser(&link);
            hero.documents.warn(match (no, opened) {
                (true, true) => "board.video.no_browser",
                (false, true) => "board.video.failed",
                (_, false) => "board.video.cannot_open",
            });
            return;
        }
        PlayerState::Starting | PlayerState::Playing => {}
    }
    let b = p.player.playback();
    vids.status = Some(PlayerStatus {
        time: b.now(now),
        duration: b.duration,
        playing: b.shown(now).running(),
        volume: b.volume,
        muted: b.muted,
    });
    let mut notice = None;
    for c in commands {
        match c {
            VideoCommand::TogglePlay if b.shown(now).running() => p.player.pause(),
            VideoCommand::TogglePlay => p.player.play(),
            VideoCommand::Seek(t) => p.player.seek(t),
            VideoCommand::Volume(v) => p.player.set_volume(v),
            VideoCommand::ToggleMute => p.player.set_muted(!b.muted),
            VideoCommand::OpenOutside => {
                p.player.pause();
                let link = ph2d_board_embed::link_at(&video, b.now(now) as u32);
                if !ph2d_board_video::open_in_browser(&link) {
                    notice = Some("board.video.cannot_open");
                }
            }
        }
    }
    // O canal, o título, «YouTube» dentro do tocador: a aba nova fechou-se lá; o link abre-se aqui.
    for url in p.player.take_outside() {
        if !ph2d_board_video::open_in_browser(&url) {
            notice = Some("board.video.cannot_open");
        }
    }
    if !live {
        hold(p);
    } else if let Some(was) = p.held.take()
        && was
    {
        p.player.play();
    }
    if let Some(sz) = settled(&mut p.asked, p.size, size, now) {
        p.player.resize(sz);
        p.size = sz;
    }
    if live {
        for i in &inbox {
            p.player.mouse(mouse(i));
        }
    }
    if let Some(f) = p.player.take_frame()
        && let Some(img) = image_of(&f)
    {
        vids.frame = Some((img, f.size));
    }
    if let Some(n) = notice {
        hero.documents.warn(n);
    }
}

#[cfg(test)]
#[path = "video_tests.rs"]
mod tests;
