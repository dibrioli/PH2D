//! W18 ([ADR-0185]) — **dá vida aos vídeos do quadro**, a cada quadro do app (pelo `io::service`):
//! - as miniaturas que o desenho pediu buscam-se num fio cada (com prazo, e a cache em disco);
//! - o vídeo ACTIVADO toca no navegador da pessoa ([`ph2d_board_video::Player`]): arranca, segue o tamanho
//!   dele no ecrã, recebe o rato e as teclas, e os quadros dele voltam para o desenho;
//! - deixar de estar vivo (tapado, rodado, pequeno, fora da vista) ou desactivar PAUSA (as regras do
//!   YouTube); o navegador fecha um minuto depois do último vídeo;
//! - sem navegador da família Chrome: abre o link no navegador do sistema e diz porquê.
//!
//! [ADR-0185]: ../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

use std::collections::BTreeSet;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use ph2d_board_model::{ElementId, Video};
use ph2d_board_render::{PlayerInput, PlayerKey, Thumb, set_thumbnail, video_key};
use ph2d_board_video::{Button, Key, Mouse, Player, PlayerState};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vector::StableImage;

/// O prazo de uma miniatura (a rede inteira: ligar, pedir, receber).
const THUMB_DEADLINE: Duration = Duration::from_secs(8);
/// Quanto o navegador fica aberto depois do último vídeo (voltar a um vídeo não espera meio segundo).
const LINGER: Duration = Duration::from_secs(60);

type Fetched = (String, Option<Vec<u8>>);

struct Playing {
    id: ElementId,
    video: Video,
    player: Player,
    size: [u32; 2],
    paused: bool,
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

/// A tecla do tocador com o nome do DOM (`key`, `code`, o código virtual do Windows, o texto).
fn dom_key(down: bool, k: PlayerKey) -> Key {
    let (key, code, vk, text): (String, String, u32, Option<String>) = match k {
        PlayerKey::Char(c) => {
            let up = c.to_ascii_uppercase();
            let code = if c.is_ascii_digit() {
                format!("Digit{c}")
            } else {
                format!("Key{up}")
            };
            (c.to_string(), code, u32::from(up), Some(c.to_string()))
        }
        PlayerKey::Space => (" ".into(), "Space".into(), 32, Some(" ".into())),
        PlayerKey::Left => ("ArrowLeft".into(), "ArrowLeft".into(), 37, None),
        PlayerKey::Up => ("ArrowUp".into(), "ArrowUp".into(), 38, None),
        PlayerKey::Right => ("ArrowRight".into(), "ArrowRight".into(), 39, None),
        PlayerKey::Down => ("ArrowDown".into(), "ArrowDown".into(), 40, None),
        PlayerKey::Home => ("Home".into(), "Home".into(), 36, None),
        PlayerKey::End => ("End".into(), "End".into(), 35, None),
        PlayerKey::Enter => ("Enter".into(), "Enter".into(), 13, None),
        PlayerKey::Tab => ("Tab".into(), "Tab".into(), 9, None),
    };
    Key {
        down,
        key,
        code,
        vk,
        text: text.filter(|_| down),
    }
}

fn mouse(i: &PlayerInput) -> Option<Mouse> {
    let b = |n: u8| match n {
        0 => Button::Left,
        1 => Button::Middle,
        _ => Button::Right,
    };
    Some(match *i {
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
        PlayerInput::Wheel { x, y, dx, dy } => Mouse::Wheel { x, y, dx, dy },
        PlayerInput::Key { .. } => return None,
    })
}

fn player(hero: &mut HeroScreen, st: &mut State) {
    let vids = hero.documents.videos_mut();
    let inbox = std::mem::take(&mut vids.inbox);
    let target = vids.active.and_then(|id| {
        let pl = vids.on_screen.get(&id)?;
        Some((id, vids.videos.get(&id)?.clone(), pl.live, pl.screen.size))
    });
    // Desactivado (ou o vídeo saiu da vista): pausa, e o navegador fica um minuto.
    let Some((id, video, live, size)) = target else {
        if let Some(p) = st.playing.as_mut()
            && !p.paused
        {
            p.player.pause();
            p.paused = true;
        }
        if vids.active.is_none() {
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
            (p.id, p.video, p.size, p.paused) = (id, video.clone(), size, false);
        }
        // Só arranca quando pode estar vivo (senão tocaria com som até a pausa chegar).
        None if !live => return,
        None => {
            st.playing = Some(Playing {
                id,
                video: video.clone(),
                player: Player::start(&video, size),
                size,
                paused: false,
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
    if !live {
        if !p.paused {
            p.player.pause();
            p.paused = true;
        }
        return;
    }
    if p.paused {
        p.player.play();
        p.paused = false;
    }
    if p.size != size {
        p.player.resize(size);
        p.size = size;
    }
    for i in &inbox {
        match i {
            PlayerInput::Key { down, key } => p.player.key(dom_key(*down, *key)),
            other => {
                if let Some(m) = mouse(other) {
                    p.player.mouse(m);
                }
            }
        }
    }
    if let Some(f) = p.player.take_frame()
        && let Some(img) = image_of(&f)
    {
        vids.frame = Some((img, f.size));
    }
}
