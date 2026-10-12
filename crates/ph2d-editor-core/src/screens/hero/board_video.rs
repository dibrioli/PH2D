//! W18 ([ADR-0185]) — **o vídeo no quadro, do lado do editor**: um link do YouTube colado (ou largado, ou
//! dado à barra) vira um vídeo; o clique no PLAY (ou num vídeo já seleccionado) activa-o; activo, o rato
//! dentro dele e as teclas vão para o tocador (`Videos::inbox`), e clicar fora desactiva-o (o oráculo).
//! Quem toca é o app (`ph2d-app-board::video`).
//!
//! [ADR-0185]: ../../../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

use ph2d_board_render::{PlayerInput, PlayerKey};
use ph2d_host::{Modifiers, PointerButton};

use super::board_keys::BoardKey;
use super::board_view::{area, area_of, editor};
use crate::screens::hero::HeroScreen;

/// ⭐ `text` é um link de um vídeo do YouTube: o vídeo entra no quadro activo com o meio no píxel de ecrã
/// `at` (ou no meio da vista). `false` = não é um vídeo (ou não há quadro): o texto segue o caminho dele.
pub fn insert_link(hero: &mut HeroScreen, text: &str, at: Option<[f64; 2]>) -> bool {
    let Some(video) = ph2d_board_embed::youtube(text) else {
        return false;
    };
    let view = hero.last_layout.map(|l| area_of(area(&l)));
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let center = match (at, view) {
        (Some(p), Some(v)) => board.camera.to_world(v, p),
        _ => [board.camera.center_x, board.camera.center_y],
    };
    let z = board.camera.zoom;
    let fit = view.map_or([f64::INFINITY; 2], |v| [v[2] / z / 2.0, v[3] / z / 2.0]);
    let natural = ph2d_board_embed::natural_size(&video);
    let history = live.histories.entry(board.id).or_default();
    let ed = editor(&mut live.editor, theme);
    ed.insert_embed(&mut board.doc, history, (text, video), natural, center, fit);
    true
}

fn button(b: PointerButton) -> u8 {
    match b {
        PointerButton::Primary => 0,
        PointerButton::Middle => 1,
        PointerButton::Secondary => 2,
    }
}

/// O carregar no quadro. `true` = era do vídeo (activá-lo, ou o tocador activo).
pub(crate) fn pointer_down(hero: &mut HeroScreen, x: f32, y: f32, b: PointerButton) -> bool {
    let p = [f64::from(x), f64::from(y)];
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let vids = &mut live.render_cache.videos;
    if let Some(id) = vids.active {
        let live_now = vids.on_screen.get(&id).is_some_and(|pl| pl.live);
        if let Some(el) = board.doc.get(id)
            && live_now
            && let Some([px, py]) = vids.player_point(id, el, p)
        {
            vids.inbox.push(PlayerInput::Down {
                x: px,
                y: py,
                button: button(b),
                clicks: 1,
            });
            live.video_press = true;
            return true;
        }
        // Fora (ou o tocador não está vivo): desactiva; o clique segue para o quadro.
        vids.active = None;
        vids.frame = None;
    }
    let selected = live
        .editor
        .as_ref()
        .map(|e| e.selection().clone())
        .unwrap_or_default();
    // O de cima primeiro (o desenhado por último).
    let mut ids: Vec<_> = vids.on_screen.keys().copied().collect();
    ids.sort_by(|a, c| {
        let za = board.doc.get(*a).map(|e| e.z.clone());
        let zc = board.doc.get(*c).map(|e| e.z.clone());
        zc.cmp(&za)
    });
    for id in ids {
        let Some(el) = board.doc.get(id) else {
            continue;
        };
        let again =
            selected.len() == 1 && selected.contains(&id) && vids.player_point(id, el, p).is_some();
        if b == PointerButton::Primary && (vids.on_play(id, el, p) || again) {
            vids.active = Some(id);
            vids.frame = None;
            let s = vids.on_screen[&id].screen;
            let aabb = el.aabb();
            if let Some(ed) = live.editor.as_mut() {
                ed.select(&board.doc, [id]);
            }
            // As regras do YouTube: tapado ou rodado não toca — diz-se porquê; pequeno ou meio fora da
            // vista, a vista vai até ele (o quadro enquadra-o inteiro).
            if s.covered {
                live.notice = Some(("board.video.covered", std::time::Instant::now()));
            } else if s.angle != 0.0 {
                live.notice = Some(("board.video.rotated", std::time::Instant::now()));
            } else if !s.whole || s.size[0].min(s.size[1]) < ph2d_board_embed::MIN_LIVE_SIDE {
                let [x0, y0, x1, y1] = aabb;
                let pad = (x1 - x0).max(y1 - y0) * FRAME_PAD;
                super::board_nav::show_box(
                    hero,
                    false,
                    false,
                    [x0 - pad, y0 - pad, x1 + pad, y1 + pad],
                );
            }
            return true;
        }
    }
    false
}

/// A folga à volta de um vídeo que a vista vai buscar (fracção do lado maior dele).
const FRAME_PAD: f64 = 0.1; // LITERAL-PX-OK: proporção da folga, não um píxel

/// O mover: com o botão em baixo dentro do tocador, ou só a passar por cima dele. `true` = era do vídeo.
pub(crate) fn pointer_move(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    let p = [f64::from(x), f64::from(y)];
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let vids = &mut live.render_cache.videos;
    let Some(id) = vids.active else { return false };
    let Some(el) = board.doc.get(id) else {
        return false;
    };
    match vids.player_point(id, el, p) {
        Some([px, py]) => {
            vids.inbox.push(PlayerInput::Move { x: px, y: py });
            live.video_press
        }
        None => live.video_press,
    }
}

/// O largar de um carregar que foi do tocador. `true` = era dele.
pub(crate) fn pointer_up(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    let p = [f64::from(x), f64::from(y)];
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    if !std::mem::take(&mut live.video_press) {
        return false;
    }
    let vids = &mut live.render_cache.videos;
    if let Some(id) = vids.active
        && let Some(el) = board.doc.get(id)
    {
        let [px, py] = vids.player_point(id, el, p).unwrap_or([-1.0, -1.0]);
        vids.inbox.push(PlayerInput::Up {
            x: px,
            y: py,
            button: 0,
            clicks: 1,
        });
    }
    true
}

/// A tecla do tocador que `k` é (as que o tocador do YouTube usa: espaço, setas, letras, algarismos).
fn player_key(k: BoardKey) -> Option<PlayerKey> {
    Some(match k {
        BoardKey::Space => PlayerKey::Space,
        BoardKey::Char(c) if c.is_ascii_alphanumeric() => PlayerKey::Char(c.to_ascii_lowercase()),
        BoardKey::Left => PlayerKey::Left,
        BoardKey::Right => PlayerKey::Right,
        BoardKey::Up => PlayerKey::Up,
        BoardKey::Down => PlayerKey::Down,
        BoardKey::Home => PlayerKey::Home,
        BoardKey::End => PlayerKey::End,
        BoardKey::Enter => PlayerKey::Enter,
        BoardKey::Tab => PlayerKey::Tab,
        _ => return None,
    })
}

/// Com um vídeo ACTIVO e vivo, as teclas são do tocador (espaço, setas, `k`/`j`/`l`, `f`, `m`, algarismos)
/// — o `Esc` e os atalhos com `Ctrl` continuam do quadro. `true` = foi para o tocador.
pub(crate) fn key(hero: &mut HeroScreen, k: BoardKey, mods: Modifiers, pressed: bool) -> bool {
    if mods.ctrl || mods.meta || mods.alt {
        return false;
    }
    let Some((_, live)) = hero.documents.active_parts() else {
        return false;
    };
    let vids = &mut live.render_cache.videos;
    let Some(id) = vids.active else { return false };
    if !vids.on_screen.get(&id).is_some_and(|pl| pl.live) {
        return false;
    }
    let Some(key) = player_key(k) else {
        return false;
    };
    vids.inbox.push(PlayerInput::Key { down: pressed, key });
    true
}
