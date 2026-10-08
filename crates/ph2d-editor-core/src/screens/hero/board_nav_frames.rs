//! ⭐ **O painel de Frames** (o do Miro: *«Quickly jump from one frame to another and reorder your
//! frames»*): um frame por linha na ordem do painel (o `z` dos frames); clicar vai a ele, duplo-
//! clique renomeia, as setas da linha trocam-no com o vizinho (UM passo de desfazer).

use super::HeroScreen;
use crate::icons::IconId;
use crate::interaction::WidgetEvent;
use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_model::{BoardOp, ElementId};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A largura do painel em passos de `Spacing::Xl4`.
const WIDTH_STEPS: f32 = 5.0; // LITERAL-PX-OK: proporção do painel

/// O que se carrega numa linha: ela (ir ao frame), subir, descer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Row,
    Up,
    Down,
}

const PARTS: [Part; 3] = [Part::Row, Part::Up, Part::Down];

/// O id da parte `part` da linha do frame `f`.
#[must_use]
pub fn row_node(f: ElementId, part: Part) -> NodeId {
    debug_assert!(f.0 >> 32 == 0, "id de elemento além de 32 bits");
    let k = PARTS.iter().position(|p| *p == part).expect("na tabela") as u64;
    NodeId(super::SALT_FRAME_ROW ^ (k << 32) ^ f.0)
}

fn of(id: NodeId) -> Option<(ElementId, Part)> {
    let v = id.0 ^ super::SALT_FRAME_ROW;
    if v >> 34 != 0 {
        return None;
    }
    let (k, e) = ((v >> 32) as usize, v & 0xffff_ffff);
    (e != 0).then_some((ElementId(e), *PARTS.get(k)?))
}

fn row_h() -> f32 {
    Spacing::Xl2.px()
}

/// O painel: à direita, em cima.
fn panel_rect(area: Rect, rows: usize) -> Rect {
    let w = Spacing::Xl4.px() * WIDTH_STEPS;
    let h = (rows as f32 + 1.0) * row_h();
    let top = area.y + Spacing::Md.px();
    let room = (area.h - 2.0 * Spacing::Md.px()).max(row_h());
    Rect::new(area.x + area.w - Spacing::Md.px() - w, top, w, h.min(room))
}

pub(super) fn paint(
    hero: &mut HeroScreen,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    if !hero.documents.live.nav.frames_open {
        return;
    }
    let Some(board) = hero.documents.active_board() else {
        return;
    };
    let frames: Vec<(ElementId, String)> = board
        .doc
        .frames_in_panel_order()
        .iter()
        .map(|f| (f.id, f.as_frame().map(|x| x.name.clone()).unwrap_or_default()))
        .collect();
    let p = panel_rect(area, frames.len());
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, p, radius, resolve(ColorToken::BgElev, theme));
    let (font, pad) = (TypeToken::Sm.px(), Spacing::Md.px());
    let text_y = |y: f32| y + (row_h() - font) / 2.0;
    let title = ph2d_i18n::tr("board.nav.frames_title");
    paint_text(ts, scene, title, p.x + pad, text_y(p.y), font, p.w - 2.0 * pad, resolve(ColorToken::Text2, theme));
    if frames.is_empty() {
        let hint = ph2d_i18n::tr("board.nav.frames_empty");
        let y = p.y + row_h();
        paint_text(ts, scene, hint, p.x + pad, text_y(y), font, p.w - 2.0 * pad, resolve(ColorToken::Text3, theme));
        return;
    }
    let renaming = hero.documents.live.nav.renaming;
    let s = row_h();
    for (i, (id, name)) in frames.iter().enumerate() {
        let y = p.y + (i as f32 + 1.0) * s;
        if y + s > p.y + p.h {
            break;
        }
        let row = Rect::new(p.x, y, p.w - 2.0 * s, s);
        let up = Rect::new(p.x + p.w - 2.0 * s, y, s, s);
        let down = Rect::new(p.x + p.w - s, y, s, s);
        for (part, r) in [(Part::Row, row), (Part::Up, up), (Part::Down, down)] {
            super::register(&mut hero.store, row_node(*id, part));
            hero.hit_index.register(row_node(*id, part), r);
        }
        if renaming == Some(*id) {
            let (hit, store) = (&mut hero.hit_index, &hero.store);
            super::paint_field(crate::ids::BOARD_FRAME_RENAME_INPUT, row, scene, ts, theme, hit, store);
        } else {
            paint_text(ts, scene, name, row.x + pad, text_y(y), font, row.w - 2.0 * pad, resolve(ColorToken::Text1, theme));
        }
        let (hit, store) = (&mut hero.hit_index, &hero.store);
        super::button(scene, hit, store, row_node(*id, Part::Up), IconId::ChevronUp, up, false, theme);
        super::button(scene, hit, store, row_node(*id, Part::Down), IconId::ChevronDown, down, false, theme);
    }
}

pub(super) fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let field = crate::ids::BOARD_FRAME_RENAME_INPUT;
    match event {
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == field => {
            commit_rename(hero);
            return true;
        }
        WidgetEvent::Cancel(id) if id == field => {
            hero.documents.live.nav.renaming = None;
            return true;
        }
        _ => {}
    }
    let (double, id) = match event {
        WidgetEvent::Click(id) => (false, id),
        WidgetEvent::DoubleClick(id) => (true, id),
        _ => return false,
    };
    let Some((frame, part)) = of(id) else {
        return false;
    };
    let area = super::super::board_view::area_of(hero.last_canvas);
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return true;
    };
    if board.doc.get(frame).and_then(|f| f.as_frame()).is_none() {
        return true;
    }
    match part {
        Part::Row if double => {
            let name = board.doc.get(frame).and_then(|f| f.as_frame()).map(|f| f.name.clone());
            live.nav.renaming = Some(frame);
            super::open_field(&mut hero.store, crate::ids::BOARD_FRAME_RENAME_INPUT, name.unwrap_or_default());
        }
        Part::Row => {
            super::go_to(board, area, frame, 1.0);
            super::super::board_view::editor(&mut live.editor, theme).select(&board.doc, [frame]);
        }
        Part::Up | Part::Down => {
            let order: Vec<ElementId> = board.doc.frames_in_panel_order().iter().map(|f| f.id).collect();
            let Some(i) = order.iter().position(|x| *x == frame) else {
                return true;
            };
            let j = if part == Part::Up { i.checked_sub(1) } else { Some(i + 1) };
            let Some(other) = j.and_then(|j| order.get(j)).copied() else {
                return true;
            };
            // Trocam as chaves de ordem: UM passo.
            let (mut a, mut b) = (board.doc.get(frame).cloned(), board.doc.get(other).cloned());
            if let (Some(a), Some(b)) = (a.as_mut(), b.as_mut()) {
                std::mem::swap(&mut a.z, &mut b.z);
                let history = live.histories.entry(board.id).or_default();
                history.apply(&mut board.doc, vec![BoardOp::Put(a.clone()), BoardOp::Put(b.clone())]);
            }
        }
    }
    true
}

fn commit_rename(hero: &mut HeroScreen) {
    let Some(frame) = hero.documents.live.nav.renaming.take() else {
        return;
    };
    let text = super::field_text(&hero.store, crate::ids::BOARD_FRAME_RENAME_INPUT);
    let text = text.trim();
    if text.is_empty() {
        return;
    }
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let Some(mut el) = board.doc.get(frame).cloned() else {
        return;
    };
    if let Some(f) = el.as_frame_mut() {
        if f.name == text {
            return;
        }
        f.name = text.to_owned();
    }
    let history = live.histories.entry(board.id).or_default();
    history.apply(&mut board.doc, vec![BoardOp::Put(el)]);
}

