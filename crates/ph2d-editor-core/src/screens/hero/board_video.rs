//! W18 ([ADR-0185]) — **o vídeo no quadro, do lado do editor**: um link do YouTube colado (ou largado, ou
//! dado à barra) vira um vídeo; o clique no PLAY (ou num vídeo já seleccionado) activa-o; activo, o clique
//! dentro dele vai para o tocador (`Videos::inbox`), as teclas são dos NOSSOS controles (W19: o tocador vem
//! com `disablekb=1`) e clicar fora desactiva-o (o oráculo). W20: o clique decide-se no LARGAR — segurar e
//! arrastar move o vídeo (activo ou não); desactivar tira o rato e as teclas, não a imagem (o tocador
//! continua com ele, `Videos::player`); os pans nunca chegam ao vídeo. No nosso ecrã inteiro o vídeo toma
//! todos os cliques e teclas; o `Esc` sai dele. Quem toca é o app (`ph2d-app-board::video`).
//!
//! [ADR-0185]: ../../../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

use ph2d_board_edit::Tool;
use ph2d_board_model::ElementId;
use ph2d_board_render::{PlayerInput, VideoCommand};
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

/// O vídeo `id` do quadro activo passa a ser `video` (o link canónico; a caixa fica), UM passo de desfazer.
/// O tocador segue-o (o app recarrega quando o vídeo do mesmo elemento muda). `false` = não é um vídeo.
fn set_video(hero: &mut HeroScreen, id: ElementId, video: Video) -> bool {
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let Some(mut el) = board.doc.get(id).cloned() else {
        return false;
    };
    let Some(e) = el.embed_mut() else {
        return false;
    };
    e.link = ph2d_board_embed::canonical_link(&video);
    e.video = video;
    let history = live.histories.entry(board.id).or_default();
    history.apply(&mut board.doc, vec![ph2d_board_model::BoardOp::Put(el)]);
    true
}

/// O trilho do vídeo `id` do quadro activo, acertado ao vídeo de agora (`None` = não há, ou o vídeo mudou por
/// outra via e o trilho caiu). Devolve também o vídeo de agora.
fn trail_of(hero: &mut HeroScreen, id: ElementId) -> Option<(&mut VideoTrail, Video)> {
    let (board, live) = hero.documents.active_parts()?;
    let now = board.doc.get(id)?.embed()?.video.clone();
    let key = (board.id, id);
    let ok = live.video_trails.get_mut(&key)?.sync(&now);
    if !ok {
        live.video_trails.remove(&key);
        return None;
    }
    Some((live.video_trails.get_mut(&key)?, now))
}

/// ⭐ W20 (o dono, 13/10: «Mais vídeos abre um vídeo no navegador e ambos ficam tocando sobrepostos» — escolheu
/// «troca na mesma janela») — o vídeo `id` passa a ser `video` (a sugestão em que se clicou dentro do tocador),
/// um passo de desfazer, e entra no TRILHO da janela (o que estava à frente dele cai, como num navegador).
/// `false` = o elemento já não é um vídeo.
pub fn navigate(hero: &mut HeroScreen, id: ElementId, video: Video) -> bool {
    let Some(now) = hero
        .documents
        .active_board()
        .and_then(|b| Some(b.doc.get(id)?.embed()?.video.clone()))
    else {
        return false;
    };
    let _ = trail_of(hero, id);
    if let Some((board, live)) = hero.documents.active_parts() {
        let t = live
            .video_trails
            .entry((board.id, id))
            .or_insert_with(|| VideoTrail {
                items: vec![now],
                at: 0,
            });
        t.items.truncate(t.at + 1);
        t.items.push(video.clone());
        t.at = t.items.len() - 1;
    }
    set_video(hero, id, video)
}

/// W20 — as setas da janela: `delta` = −1 o vídeo de antes no trilho, +1 o seguinte (cada passo é um passo de
/// desfazer). `false` = não há para onde ir.
pub fn step(hero: &mut HeroScreen, id: ElementId, delta: isize) -> bool {
    let Some((t, _)) = trail_of(hero, id) else {
        return false;
    };
    let Some(to) =
        t.at.checked_add_signed(delta)
            .filter(|i| *i < t.items.len())
    else {
        return false;
    };
    t.at = to;
    let video = t.items[to].clone();
    set_video(hero, id, video)
}

/// W20 — «voltar ao original»: a janela volta ao 1.º vídeo do trilho e o trilho apaga-se (um passo de
/// desfazer). `false` = não havia navegação.
pub fn reset(hero: &mut HeroScreen, id: ElementId) -> bool {
    let Some((t, now)) = trail_of(hero, id) else {
        return false;
    };
    let first = t.items[0].clone();
    if let Some((board, live)) = hero.documents.active_parts() {
        live.video_trails.remove(&(board.id, id));
    }
    first == now || set_video(hero, id, first)
}

/// W20 — o que as setas da janela `id` podem fazer agora: `Some((antes, depois))` se ela navegou.
#[must_use]
pub fn trail(hero: &mut HeroScreen, id: ElementId) -> Option<(bool, bool)> {
    let (t, _) = trail_of(hero, id)?;
    // Um trilho tem sempre ≥ 2 (nasce com o original e a sugestão; o «voltar ao original» apaga-o).
    Some((t.at > 0, t.at + 1 < t.items.len()))
}

fn button(b: PointerButton) -> u8 {
    match b {
        PointerButton::Primary => 0,
        PointerButton::Middle => 1,
        PointerButton::Secondary => 2,
    }
}

/// O NOSSO ecrã inteiro está ligado (o vídeo activo tapa o quadro).
#[must_use]
pub fn fullscreen(hero: &HeroScreen) -> bool {
    hero.documents.live.render_cache.videos.fullscreen && hero.documents.active().is_some()
}

use crate::documents::{VideoClick, VideoTrail};
use ph2d_board_model::Video;

/// W21 — segurar parado mais do que isto e largar não é um clique: o «segurar» do rato da casa (o oráculo
/// fica perto de 300 ms, `sonda_w21.mjs` T1; a casa vence, `10_video_do_youtube.md` §W21).
pub(crate) const HOLD_NS: u128 = crate::interaction::LONG_PRESS_THRESHOLD_NS;

/// O carregar no quadro. `true` = era do vídeo (no nosso ecrã inteiro, onde o tocador toma tudo, e o de outra
/// ferramenta à espera). Os pans (o botão do meio — também o dos dois dedos —, o Espaço, a mão) nunca são do
/// vídeo. Num vídeo, o carregar fica à espera do largar ([`VideoClick`]): com a ferramenta de seleccionar
/// segue também para o editor; com outra (W21, o dono: «clique = play/pause»), o editor só o recebe se virar
/// arrasto ou segurar ([`deferred_drag`], [`deferred_hold`]). O laser aponta, não é do vídeo. Carregar fora
/// do vídeo activo desactiva-o — tira-lhe o rato, não a imagem (W20).
pub(crate) fn pointer_down(
    hero: &mut HeroScreen,
    x: f32,
    y: f32,
    b: PointerButton,
    now_ns: u128,
) -> bool {
    let p = [f64::from(x), f64::from(y)];
    let space = hero.documents.live.space;
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    live.space_tap = false;
    live.video_click = None;
    let tool = live.editor.as_ref().map(|e| e.tool);
    let vids = &mut live.render_cache.videos;
    let player_at = |vids: &ph2d_board_render::Videos, id: ElementId| {
        let live_now = vids.on_screen.get(&id).is_some_and(|pl| pl.live);
        let el = board.doc.get(id)?;
        vids.player_point(id, el, p).filter(|_| live_now)
    };
    if vids.fullscreen {
        // No ecrã inteiro nada chega ao quadro por baixo; dentro do vídeo, o carregar é do tocador.
        if let Some(id) = vids.active
            && let Some([px, py]) = player_at(vids, id)
        {
            vids.inbox.push(PlayerInput::Down {
                x: px,
                y: py,
                button: button(b),
                clicks: 1,
            });
            live.video_press = true;
        }
        return true;
    }
    if b == PointerButton::Middle || space || tool == Some(Tool::Hand) {
        return false;
    }
    let select = matches!(tool, None | Some(Tool::Select));
    let other = !select && tool != Some(Tool::Laser);
    if let Some(id) = vids.active {
        match player_at(vids, id) {
            Some(pt) if (select || other) && b == PointerButton::Primary => {
                live.video_click = Some(VideoClick {
                    id,
                    at: p,
                    player: Some(pt),
                    dragged: false,
                    at_ns: now_ns,
                    tool: other,
                });
                return other;
            }
            Some(_) if !select => return false,
            // O botão direito é do tocador no carregar (o menu dele).
            Some([px, py]) => {
                vids.inbox.push(PlayerInput::Down {
                    x: px,
                    y: py,
                    button: button(b),
                    clicks: 1,
                });
                live.video_press = true;
                return true;
            }
            None => vids.active = None,
        }
    }
    if !(select || other) || b != PointerButton::Primary {
        return false;
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
    // Com a selecção, o play ou um vídeo já seleccionado; com outra ferramenta, qualquer ponto do tocador.
    let hit = ids.into_iter().find(|&id| {
        board.doc.get(id).is_some_and(|el| {
            let inside = vids.player_point(id, el, p).is_some();
            let again = selected.len() == 1 && selected.contains(&id) && inside;
            (other && inside) || vids.on_play(id, el, p) || again
        })
    });
    live.video_click = hit.map(|id| VideoClick {
        id,
        at: p,
        player: None,
        dragged: false,
        at_ns: now_ns,
        tool: other,
    });
    other && hit.is_some()
}

/// W21 — um carregar num vídeo com outra ferramenta na mão passou o limiar de arrasto em `(x, y)`: deixa de
/// ser um clique e o ponto onde carregou (ecrã) vai para o editor.
pub(crate) fn deferred_drag(hero: &mut HeroScreen, x: f32, y: f32) -> Option<[f64; 2]> {
    let live = &mut hero.documents.live;
    let c = live.video_click.filter(|c| c.tool)?;
    let far = (f64::from(x) - c.at[0]).hypot(f64::from(y) - c.at[1]) > super::board_view::metrics().drag;
    far.then(|| {
        live.video_click = None;
        c.at
    })
}

/// W21 — o largar de um carregar num vídeo com outra ferramenta, segurado parado além de [`HOLD_NS`]: não é
/// clique, e o ponto onde carregou vai para o editor (a ferramenta faz o seu clique).
pub(crate) fn deferred_hold(hero: &mut HeroScreen, now_ns: u128) -> Option<[f64; 2]> {
    let live = &mut hero.documents.live;
    let c = live
        .video_click
        .filter(|c| c.tool && now_ns.saturating_sub(c.at_ns) >= HOLD_NS)?;
    live.video_click = None;
    Some(c.at)
}

/// O clique (sem arrasto) num vídeo: activo e vivo, vai ao tocador; senão activa-o — e o clique num vídeo
/// que o tocador já tem é tocar/pausar (o clique é sempre play/pause, W20).
fn click(hero: &mut HeroScreen, c: VideoClick) {
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let Some(el) = board.doc.get(c.id) else {
        return;
    };
    let vids = &mut live.render_cache.videos;
    if let Some([x, y]) = c.player {
        if vids.active == Some(c.id) {
            vids.inbox.push(PlayerInput::Down {
                x,
                y,
                button: 0,
                clicks: 1,
            });
            vids.inbox.push(PlayerInput::Up {
                x,
                y,
                button: 0,
                clicks: 1,
            });
        }
        return;
    }
    let Some(pl) = vids.on_screen.get(&c.id).copied() else {
        return;
    };
    if vids.player == Some(c.id) && pl.live {
        vids.commands.push(VideoCommand::TogglePlay);
    }
    vids.activate(c.id);
    let s = pl.screen;
    let aabb = el.aabb();
    if let Some(ed) = live.editor.as_mut() {
        ed.select(&board.doc, [c.id]);
    }
    // As regras do YouTube: tapado ou rodado não toca — diz-se porquê; com o tocador abaixo de 200×200 px,
    // a vista vai até ele (o quadro enquadra-o inteiro).
    if s.covered {
        live.notice = Some(("board.video.covered", std::time::Instant::now()));
    } else if s.angle != 0.0 {
        live.notice = Some(("board.video.rotated", std::time::Instant::now()));
    } else if s.size[0].min(s.size[1]) < ph2d_board_embed::MIN_LIVE_SIDE {
        let [x0, y0, x1, y1] = aabb;
        let pad = (x1 - x0).max(y1 - y0) * FRAME_PAD;
        super::board_nav::fit_now(hero, [x0 - pad, y0 - pad, x1 + pad, y1 + pad]);
    }
}

/// A folga à volta de um vídeo que a vista vai buscar (fracção do lado maior dele).
const FRAME_PAD: f64 = 0.1; // LITERAL-PX-OK: proporção da folga, não um píxel

/// O mover: com o botão em baixo dentro do tocador, ou só a passar por cima dele. `true` = era do vídeo.
/// Um carregar à espera ([`VideoClick`]) só mede o arrasto: o mover segue para o editor.
pub(crate) fn pointer_move(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    let p = [f64::from(x), f64::from(y)];
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    if let Some(c) = live.video_click.as_mut() {
        c.dragged |= (p[0] - c.at[0]).hypot(p[1] - c.at[1]) > super::board_view::metrics().drag;
        return false;
    }
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

/// O largar de um carregar que foi do tocador (`true` = era dele), ou de um à espera ([`VideoClick`]): sem
/// arrasto é o clique; o largar segue para o editor (que fecha o gesto dele).
pub(crate) fn pointer_up(hero: &mut HeroScreen, x: f32, y: f32, now_ns: u128) -> bool {
    let p = [f64::from(x), f64::from(y)];
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    if let Some(c) = live.video_click.take() {
        if !c.dragged && now_ns.saturating_sub(c.at_ns) < HOLD_NS {
            click(hero, c);
        }
        // Com outra ferramenta o editor nunca viu o carregar: o largar é todo do vídeo.
        return c.tool;
    }
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

/// O salto das setas e de `j`/`l`, e o passo do volume (os atalhos do tocador do YouTube).
const ARROW_SEEK: f64 = 5.0; // LITERAL-PX-OK: segundos do atalho do YouTube, não px
const JL_SEEK: f64 = 10.0; // LITERAL-PX-OK: segundos do atalho do YouTube, não px
const VOLUME_STEP: i16 = 5;

/// ⭐ W19 — com um vídeo ACTIVO e vivo, as teclas são dos NOSSOS controles, nos atalhos do YouTube:
/// espaço/`k` toca e pausa, ←/→ saltam 5 s (`j`/`l` 10 s), ↑/↓ o volume, `m` o mudo, `f` o nosso ecrã
/// inteiro; `Esc` sai dele. No ecrã inteiro TODAS as teclas são do vídeo (o quadro por baixo não as vê).
/// `true` = foram do vídeo.
pub(crate) fn key(hero: &mut HeroScreen, k: BoardKey, mods: Modifiers, pressed: bool) -> bool {
    let Some((_, live)) = hero.documents.active_parts() else {
        return false;
    };
    let Some(id) = live.render_cache.videos.active else {
        return false;
    };
    let full = live.render_cache.videos.fullscreen;
    if mods.ctrl || mods.meta || mods.alt {
        return full;
    }
    // W20 — no quadro o Espaço é também o pan («Pan pausa o vídeo»): tocar/pausar só no LARGAR de um toque
    // sem nenhum carregar do rato pelo meio (`pointer_down` desarma-o); a tecla segue para o quadro.
    if k == BoardKey::Space && !full {
        if pressed {
            live.space_tap |= !live.space;
        } else if std::mem::take(&mut live.space_tap) {
            let vids = &mut live.render_cache.videos;
            let live_now = vids.on_screen.get(&id).is_some_and(|pl| pl.live);
            if live_now && vids.status.is_some() {
                vids.commands.push(VideoCommand::TogglePlay);
            }
        }
        return false;
    }
    let vids = &mut live.render_cache.videos;
    if full && k == BoardKey::Escape {
        if pressed {
            vids.fullscreen = false;
        }
        return true;
    }
    let live_now = vids.on_screen.get(&id).is_some_and(|pl| pl.live);
    let Some(s) = vids.status.filter(|_| live_now) else {
        return full;
    };
    let seek =
        |d: f64| VideoCommand::Seek(crate::math::safe_clamp_f64(s.time + d, 0.0, s.duration));
    let volume = |d: i16| VideoCommand::Volume((i16::from(s.volume) + d).clamp(0, 100) as u8);
    let cmd = match k {
        BoardKey::Space | BoardKey::Char('k') => Some(VideoCommand::TogglePlay),
        BoardKey::Left => Some(seek(-ARROW_SEEK)),
        BoardKey::Right => Some(seek(ARROW_SEEK)),
        BoardKey::Char('j') => Some(seek(-JL_SEEK)),
        BoardKey::Char('l') => Some(seek(JL_SEEK)),
        BoardKey::Up => Some(volume(VOLUME_STEP)),
        BoardKey::Down => Some(volume(-VOLUME_STEP)),
        BoardKey::Char('m') => Some(VideoCommand::ToggleMute),
        BoardKey::Char('f') => {
            if pressed {
                vids.fullscreen = !full;
            }
            return true;
        }
        _ => None,
    };
    let Some(cmd) = cmd else { return full };
    if pressed {
        vids.commands.push(cmd);
    }
    true
}
