//! As linhas das acções: o ponto da cor, o nome, os chips (nome inteiro + ícone do aparelho), o «+» que escuta no
//! lugar, o lixo, os dois números só com manche, e o pé.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::widget::{SCROLLBAR_W, ScrollbarState, paint_scrollbar, scrollbar_is_needed};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::{ActionId, Binding, GamepadAxis, InputAction};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use super::{footer, ring, text};
use crate::doc::InputMapDoc;
use crate::ids::ListHit;

const ROW_H: f32 = m::FIELD_H + m::SEG_INSET;
const CHIP_H: f32 = m::FIELD_H - m::SEG_INSET;
/// A linha dos números: o rótulo por cima e o campo.
const ZONE_H: f32 = m::LABEL_PX + m::LABEL_BOTTOM + m::FIELD_H + m::FIELD_ROW_GAP;

fn row_h(a: &InputAction) -> f32 {
    if a.is_analog() { ROW_H + ZONE_H } else { ROW_H }
}

pub(super) fn paint_rows(
    doc: &mut InputMapDoc,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let shown = super::shown(doc);
    let actions: Vec<InputAction> = shown
        .iter()
        .filter_map(|id| doc.snap.map.get(*id).cloned())
        .collect();
    let name_w = actions
        .iter()
        .map(|a| text::title_width(ts, &a.name, m::VALUE_PX))
        .fold(0.0_f32, f32::max)
        + m::KEY_MARK_BOX
        + m::CARD_PAD_X * 2.0;
    let mut phrases = foot(doc);
    if doc.snap.listening.is_none() && actions.iter().any(InputAction::is_analog) {
        phrases.push(footer::zones());
    }
    let foot_h = (m::FIELD_H + m::FIELD_ROW_GAP) * phrases.len() as f32;
    let list_h = (area.h - foot_h).max(0.0);
    let total: f32 = actions.iter().map(|a| row_h(a) + m::FIELD_ROW_GAP).sum();
    doc.list.scroll_max = (total - list_h).max(0.0);
    doc.list.scroll = doc.list.scroll.clamp(0.0, doc.list.scroll_max);
    // A barra do cartão quando a lista rola; as linhas encolhem para ela não tapar o lixo.
    let view = Rect::new(area.x, area.y, area.w, list_h);
    let bar = scrollbar_is_needed(total, list_h);
    let row_w = if bar {
        (area.w - SCROLLBAR_W - m::SEG_INSET).max(0.0)
    } else {
        area.w
    };
    if bar {
        let visual = (ScrollbarState::Normal, 0.0);
        let _ = paint_scrollbar(
            view,
            doc.list.scroll,
            total,
            list_h,
            visual,
            scene,
            ink.theme,
        );
    }
    let mut y = area.y - doc.list.scroll;
    for a in &actions {
        let h = row_h(a);
        let r = Rect::new(area.x, y, row_w, h);
        y += h + m::FIELD_ROW_GAP;
        if r.y < area.y - f32::EPSILON || r.y + r.h > area.y + list_h + f32::EPSILON {
            continue;
        }
        paint_row(doc, a, r, name_w, scene, ts, ink);
    }
    let mut fy = area.y + total.min(list_h);
    for p in phrases {
        let r = Rect::new(area.x, fy, area.w, m::FIELD_H);
        fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(ColorToken::CardField));
        let (lead_c, rest_c) = if p.warn {
            (ColorToken::Warn, ColorToken::Warn)
        } else {
            (ColorToken::Text1, ColorToken::Text2)
        };
        let inner = Rect::new(r.x + m::SEG_INSET, r.y, r.w - m::SEG_INSET * 2.0, r.h);
        let lead_w = text::title_width(ts, &p.lead, m::VALUE_PX);
        text::title(
            &mut doc.census,
            ts,
            scene,
            &p.lead,
            Rect::new(inner.x, r.y, lead_w.min(inner.w), r.h),
            m::VALUE_PX,
            ink.color(lead_c),
        );
        if !p.rest.is_empty() {
            let gap = m::SEG_ICON_GAP;
            let rx = inner.x + lead_w + gap;
            let rw = (inner.x + inner.w - rx).max(0.0);
            text::line(
                &mut doc.census,
                ts,
                scene,
                &p.rest,
                Rect::new(rx, r.y, rw, r.h),
                m::VALUE_PX,
                ink.color(rest_c),
            );
        }
        fy += m::FIELD_H + m::FIELD_ROW_GAP;
    }
}

/// O ícone da direcção de uma metade de manche. ⚠️ Irmã de `view_keyboard::geometry::half_icon` (privada à
/// vista B): a rodada seguinte funde as duas numa só porta.
fn half_icon(b: Binding) -> Option<IconId> {
    let Binding::PadAxis { axis, positive } = b else {
        return None;
    };
    Some(match (axis, positive) {
        (GamepadAxis::LeftStickX | GamepadAxis::RightStickX, false) => IconId::ChevronLeft,
        (GamepadAxis::LeftStickX | GamepadAxis::RightStickX, true) => IconId::ChevronRight,
        (GamepadAxis::LeftStickY | GamepadAxis::RightStickY, true) => IconId::ChevronUp,
        (GamepadAxis::LeftStickY | GamepadAxis::RightStickY, false) => IconId::ChevronDown,
        _ => return None,
    })
}

fn foot(doc: &InputMapDoc) -> Vec<footer::Phrase> {
    if let Some(id) = doc.snap.listening {
        return vec![footer::listening(&doc.snap, id)];
    }
    if let Some(k) = doc.list.search_key {
        return vec![footer::search(&doc.snap, k, &super::shown(doc))];
    }
    footer::summary(&doc.snap)
}

#[allow(clippy::too_many_arguments)]
fn paint_row(
    doc: &mut InputMapDoc,
    a: &InputAction,
    r: Rect,
    name_w: f32,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let head = Rect::new(r.x, r.y, r.w, ROW_H);
    fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(ColorToken::CardField));
    doc.hits.register(ListHit::Row(a.id).id(), r);
    let hover = doc.list.hover_row == Some(a.id);
    let dot = m::KEY_MARK_PX;
    let dot_r = Rect::new(
        head.x + m::SEG_INSET,
        head.y + (head.h - dot) * 0.5,
        dot,
        dot,
    );
    fill_rounded_rect(
        scene,
        dot_r,
        dot * 0.5,
        ink.color(crate::palette::of_action(a.id)),
    );
    let name_r = Rect::new(
        dot_r.x + dot + m::SEG_INSET,
        head.y,
        name_w - dot - m::SEG_INSET * 2.0,
        head.h,
    );
    text::title(
        &mut doc.census,
        ts,
        scene,
        &a.name,
        name_r,
        m::VALUE_PX,
        ink.color(ColorToken::Text1),
    );
    let trash_w = m::ICON_CELL;
    let right = head.x + head.w - trash_w - m::SEG_INSET;
    let mut x = head.x + name_w;
    let cy = head.y + (head.h - CHIP_H) * 0.5;
    let g = m::KEY_MARK_BOX;
    for (n, b) in a.bindings.iter().enumerate() {
        let label = footer::binding_label(*b);
        let tw = ts.prefix_width(&label, m::VALUE_PX);
        // A metade da manche leva a direcção desenhada («Left Stick ←»), nunca um glifo.
        let arrow = half_icon(*b);
        let aw = if arrow.is_some() {
            g + m::SEG_ICON_GAP
        } else {
            0.0
        };
        let w = m::SEG_INSET * 3.0 + g * 2.0 + m::SEG_ICON_GAP + tw + aw;
        let chip = Rect::new(x, cy, w.min((right - x).max(0.0)), CHIP_H);
        let lit = doc.list.search_key.is_some_and(|k| *b == Binding::Key(k));
        let (bg, fg) = if lit {
            (ColorToken::Warn, ColorToken::KanbanBtnOnFg)
        } else {
            (ColorToken::KanbanList, ColorToken::Text1)
        };
        fill_rounded_rect(scene, chip, m::SEG_RADIUS, ink.color(bg));
        let icon = match b {
            Binding::Key(_) => IconId::Kbd,
            Binding::PadButton(_) | Binding::PadAxis { .. } => IconId::Player,
        };
        let ir = Rect::new(chip.x + m::SEG_INSET, chip.y + (chip.h - g) * 0.5, g, g);
        paint_icon(
            scene,
            icon,
            ir,
            ink.color(ColorToken::Text2),
            StrokeToken::Default.px(),
        );
        let tr_r = Rect::new(
            ir.x + g + m::SEG_ICON_GAP,
            chip.y,
            tw.min((chip.x + chip.w - ir.x - g - m::SEG_ICON_GAP - aw).max(0.0)),
            chip.h,
        );
        let used = text::line(
            &mut doc.census,
            ts,
            scene,
            &label,
            tr_r,
            m::VALUE_PX,
            ink.color(fg),
        );
        if let Some(icon) = arrow {
            let ar = Rect::new(tr_r.x + used + m::SEG_ICON_GAP, ir.y, g, g);
            paint_icon(scene, icon, ar, ink.color(fg), StrokeToken::Default.px());
        }
        let xr = Rect::new(chip.x + chip.w - g - m::SEG_INSET, ir.y, g, g);
        if hover {
            paint_icon(
                scene,
                IconId::Close,
                xr,
                ink.color(ColorToken::Text2),
                StrokeToken::Default.px(),
            );
            doc.hits.register(ListHit::RemoveBinding(a.id, n).id(), xr);
        }
        x += chip.w + m::SEG_PAD * 2.0;
    }
    paint_plus(
        doc,
        a.id,
        Rect::new(x, cy, (right - x).max(0.0), CHIP_H),
        scene,
        ts,
        ink,
    );
    if hover {
        let tr_box = Rect::new(right, head.y + (head.h - g) * 0.5, trash_w, g);
        let gl = Rect::new(tr_box.x + (trash_w - g) * 0.5, tr_box.y, g, g);
        paint_icon(
            scene,
            IconId::Trash,
            gl,
            ink.color(ColorToken::Text2),
            StrokeToken::Default.px(),
        );
        doc.hits.register(ListHit::Trash(a.id).id(), tr_box);
    }
    if a.is_analog() {
        paint_zones(
            doc,
            a,
            Rect::new(r.x, r.y + ROW_H, r.w, ZONE_H),
            scene,
            ts,
            ink,
        );
    }
}

/// O «+» fantasma, ou — à escuta — a caixa no lugar onde a tecla vai cair.
fn paint_plus(
    doc: &mut InputMapDoc,
    id: ActionId,
    room: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let g = m::KEY_MARK_BOX;
    if doc.snap.listening == Some(id) {
        let key = if doc.snap.gamepad_connected {
            "input_map_list.listen_pad"
        } else {
            "input_map_list.listen_key"
        };
        let msg = tr(key);
        let w = (ts.prefix_width(msg, m::VALUE_PX) + m::SEG_INSET * 2.0).min(room.w);
        let r = Rect::new(room.x, room.y, w, room.h);
        ring(scene, r, ink);
        let inner = Rect::new(r.x + m::SEG_INSET, r.y, r.w - m::SEG_INSET * 2.0, r.h);
        text::line(
            &mut doc.census,
            ts,
            scene,
            msg,
            inner,
            m::VALUE_PX,
            ink.color(ColorToken::Text2),
        );
        doc.hits.register(ListHit::Listen(id).id(), r);
        return;
    }
    let r = Rect::new(room.x, room.y, (g + m::SEG_INSET * 2.0).min(room.w), room.h);
    if doc.list.hover_row == Some(id) {
        fill_rounded_rect(
            scene,
            r,
            m::SEG_RADIUS,
            ink.color(ColorToken::CardFieldHover),
        );
    }
    let gl = Rect::new(r.x + (r.w - g) * 0.5, r.y + (r.h - g) * 0.5, g, g);
    paint_icon(
        scene,
        IconId::Plus,
        gl,
        ink.color(ColorToken::Text2),
        StrokeToken::Default.px(),
    );
    doc.hits.register(ListHit::Listen(id).id(), r);
}

/// «Dead zone (0 to 1)» e «Press at (0 to 1)»: dois campos lado a lado, arrastáveis.
fn paint_zones(
    doc: &mut InputMapDoc,
    a: &InputAction,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let pad = m::SEG_INSET;
    let half = (r.w - pad * 2.0 - m::FIELD_GAP) * 0.5;
    for (i, (hit, label, value)) in [
        (ListHit::Dead(a.id), "input_map_list.dead", a.dead_zone),
        (ListHit::Press(a.id), "input_map_list.press", a.press_point),
    ]
    .into_iter()
    .enumerate()
    {
        let x = r.x + pad + (half + m::FIELD_GAP) * i as f32;
        let lr = Rect::new(x, r.y, half, m::LABEL_PX + m::LABEL_BOTTOM);
        text::line(
            &mut doc.census,
            ts,
            scene,
            tr(label),
            lr,
            m::LABEL_PX,
            ink.color(ColorToken::CardLabel),
        );
        let f = Rect::new(x, lr.y + lr.h, half, m::FIELD_H);
        let live = doc.list.scrub.is_some_and(|s| {
            ListHit::Dead(s.action) == hit && s.zone == super::state::Zone::Dead
                || ListHit::Press(s.action) == hit && s.zone == super::state::Zone::Press
        });
        fill_rounded_rect(
            scene,
            f,
            m::FIELD_RADIUS,
            ink.color(if live {
                ColorToken::CardFieldHover
            } else {
                ColorToken::KanbanList
            }),
        );
        if live {
            ring(scene, f, ink);
        }
        let typing = doc.list.typing.as_ref().filter(|t| {
            t.action == a.id
                && (t.zone == super::state::Zone::Dead) == matches!(hit, ListHit::Dead(_))
        });
        if typing.is_some() {
            ring(scene, f, ink);
        }
        let v = typing.map_or_else(|| format!("{value:.2}"), |t| t.buffer.clone());
        let vr = Rect::new(f.x + m::SEG_INSET, f.y, f.w - m::SEG_INSET * 2.0, f.h);
        text::line(
            &mut doc.census,
            ts,
            scene,
            &v,
            vr,
            m::VALUE_PX,
            ink.color(ColorToken::Text1),
        );
        doc.hits.register(hit.id(), f);
        doc.census.numbers.push(a.id);
    }
}
