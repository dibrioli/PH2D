//! As bolhas de comentário no quadro (módulo filho de [`super`]): desenhá-las (K6/K7) e ARRASTÁ-LAS
//! para outro sítio (K2 — o dono, 10/10: *«não é possível mover o balão de comentário»*). O cartão,
//! a lista e o resto do desenho estão em `board_team_paint.rs`.

use super::super::super::HeroScreen;
use super::paint::{board, btn};
use super::{Hit, TeamOp, hit_of, me, node, open_thread, register};
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_board_model::{Pin, ThreadId};
use ph2d_host::PointerButton;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, TypeToken};
use ph2d_vector::VectorScene;

/// K7 — o selo da contagem: 0,47 do Ø da bolha (medido, `21016057338514`).
const BADGE: f32 = 0.47; // LITERAL-PX-OK: fracção medida da bolha

/// ⭐ K7 — a bolha do fio cuja ponta (o canto de baixo à esquerda) está em `at` (px de ecrã).
#[must_use]
pub fn bubble_rect(at: [f32; 2]) -> Rect {
    let d = btn();
    Rect::new(at[0], at[1] - d, d, d)
}

/// Os fios à vista no quadro: os não resolvidos (e os resolvidos com «Show resolved») cujo alvo
/// existe (K3), com a ponta em px de ecrã.
pub(super) fn visible(hero: &HeroScreen, area: Rect) -> Vec<(ThreadId, [f32; 2])> {
    let Some(b) = board(hero) else {
        return Vec::new();
    };
    let t = &hero.documents.live.team;
    if t.hide_comments {
        return Vec::new();
    }
    let a = super::super::super::board_view::area_of(area);
    b.team
        .threads
        .iter()
        .filter(|th| !th.resolved || t.show_resolved)
        .filter_map(|th| {
            let p = b.camera.to_screen(a, th.anchor(&b.doc)?);
            Some((th.id, [p[0] as f32, p[1] as f32]))
        })
        .filter(|(_, p)| area.contains(p[0], p[1]))
        .collect()
}

/// ⭐ K6/K7 — as bolhas à vista (a arrastada segue o ponteiro).
pub(super) fn paint(
    hero: &mut HeroScreen,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
) {
    let theme = hero.theme;
    let mine = me(hero);
    let d = btn();
    // O contorno da gota é um ANEL: pela porta do tema (o plano não o tem em repouso).
    let ring = crate::paint::fill_ring(
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    let drag = dragged(hero);
    for (id, at) in visible(hero, area) {
        let at = drag.filter(|d| d.thread == id).map_or(at, |d| d.tip());
        let Some(th) = board(hero).and_then(|b| b.team.thread(id)) else {
            continue;
        };
        let (resolved, unread, count) = (th.resolved, th.unread_for(&mine), th.messages.len());
        let initial: String = th
            .messages
            .first()
            .and_then(|m| m.author.chars().next())
            .map(|c| c.to_uppercase().collect())
            .unwrap_or_default();
        let r = bubble_rect(at);
        // K6 — novo azul, lido branco, resolvido com o visto.
        let (bg, fg) = if resolved {
            (ColorToken::Bg2, ColorToken::Text2)
        } else if unread {
            (ColorToken::Accent, ColorToken::AccentFg)
        } else {
            (ColorToken::BgElev, ColorToken::Text1)
        };
        // A gota: um círculo e o quadrante de baixo à esquerda em ponta (a ponta está no alvo).
        let inset = ring.map_or(0.0, |(w, c)| {
            fill_rounded_rect(scene, r, d / 2.0, c);
            w
        });
        let inner = Rect::new(r.x + inset, r.y + inset, d - 2.0 * inset, d - 2.0 * inset);
        fill_rounded_rect(scene, inner, inner.w / 2.0, resolve(bg, theme));
        let tail = Rect::new(r.x, r.y + d / 2.0, d / 2.0, d / 2.0);
        fill_rounded_rect(scene, tail, 0.0, resolve(bg, theme));
        if resolved {
            let i = Spacing::Sm.px();
            let ir = Rect::new(r.x + i, r.y + i, d - 2.0 * i, d - 2.0 * i);
            let stroke = StrokeToken::Thick.px();
            crate::paint::paint_icon(scene, IconId::Check, ir, resolve(fg, theme), stroke);
        } else {
            let font = TypeToken::Sm.px();
            paint_text_centered(ts, scene, &initial, r, font, resolve(fg, theme));
        }
        if count > 1 {
            // K7 — o selo da contagem centrado no canto de cima à direita.
            let s = d * BADGE;
            let br = Rect::new(r.x + d - s / 2.0, r.y - s / 2.0, s, s);
            fill_rounded_rect(scene, br, s / 2.0, resolve(ColorToken::Accent, theme));
            let fg = resolve(ColorToken::AccentFg, theme);
            paint_text_centered(ts, scene, &count.to_string(), br, TypeToken::Xs.px(), fg);
        }
        let id = node(Hit::Bubble(id));
        register(&mut hero.store, id);
        hero.store.set_tooltip(id, tr("board.comment.open"));
        hero.hit_index.register(id, r);
    }
}

/// K2 — a bolha que anda com o ponteiro (só depois do limiar: antes é um clique).
pub(super) fn dragged(hero: &HeroScreen) -> Option<crate::documents::team::BubbleDrag> {
    hero.documents.live.team.drag.filter(|d| d.moved)
}

/// A ponta da bolha do fio `t` em px de ecrã, se ele está à vista (o mesmo cálculo que a pinta).
pub(super) fn tip_of(hero: &HeroScreen, area: Rect, t: ThreadId) -> Option<[f32; 2]> {
    visible(hero, area)
        .into_iter()
        .find(|(id, _)| *id == t)
        .map(|(_, at)| at)
}

/// ⭐ K2 — carregar numa bolha (primário, sem `Espaço`) começa a arrastá-la; o largar sem ter andado é
/// um clique (abre o fio). `true` = era uma bolha.
pub fn bubble_down(hero: &mut HeroScreen, x: f32, y: f32, button: PointerButton) -> bool {
    if button != PointerButton::Primary || hero.documents.live.space {
        return false;
    }
    let Some(Hit::Bubble(t)) = hero.hit_index.hit(x, y).and_then(hit_of) else {
        return false;
    };
    let Some(tip) = tip_of(hero, hero.last_canvas, t) else {
        return false;
    };
    hero.documents.live.team.drag = Some(crate::documents::team::BubbleDrag {
        thread: t,
        grab: [x - tip[0], y - tip[1]],
        from: [x, y],
        at: [x, y],
        moved: false,
    });
    true
}

/// K2 — a bolha segue o ponteiro. `true` = há uma a ser arrastada.
pub fn bubble_drag(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    let Some(d) = hero.documents.live.team.drag.as_mut() else {
        return false;
    };
    d.at = [x, y];
    let far = (x - d.from[0]).hypot(y - d.from[1]) > crate::interaction::TAB_DRAG_THRESHOLD_PX;
    d.moved |= far;
    true
}

/// ⭐ K2 — o largar: sem ter andado abre o fio; depois de andar prende-o ao objecto de cima sob a PONTA
/// (o mesmo pick do comentar), ou ao ponto — um passo do desfazer do quadro. `true` = era uma bolha.
pub fn bubble_up(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    let Some(mut d) = hero.documents.live.team.drag.take() else {
        return false;
    };
    d.at = [x, y];
    if !d.moved {
        open_thread(hero, d.thread);
        return true;
    }
    let area = super::super::super::board_view::area_of(hero.last_canvas);
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return true;
    };
    let tip = d.tip();
    let world = board
        .camera
        .to_world(area, [f64::from(tip[0]), f64::from(tip[1])]);
    let p = ph2d_board_edit::Pointer {
        world,
        px: 1.0 / board.camera.zoom,
        mods: ph2d_board_edit::Mods::default(),
    };
    let on = super::super::super::board_view::editor(&mut live.editor, theme).hit(&board.doc, p);
    let pin = Pin::at(&board.doc, on, world);
    let history = live.histories.entry(board.id).or_default();
    history.apply_team(&mut board.team, TeamOp::Pin(d.thread, pin));
    true
}
