//! The stick (C3): o círculo da manche — fora = o curso todo, disco = Dead, anel tracejado = Press, o ponto ao vivo.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;

use super::cards::{GAP, PAD, ROW, inner_w};
use super::{line_h, para, para_h};
use crate::doc::InputMapDoc;
use crate::view_list::text::line;
use ph2d_editor_core::paint::{fill_circle, fill_rounded_rect, stroke_polyline};
use ph2d_input::resolve::{pressed_of, strength_of};
use ph2d_input::{ActionId, Binding};

/// Quantos traços tem o anel tracejado, e quantos lados o círculo.
const DASHES: usize = 24;
const SIDES: usize = 64;

/// A acção da manche: a aberta na vista B se usa manche, senão a primeira que usa.
pub(crate) fn stick_action(doc: &InputMapDoc) -> Option<ActionId> {
    let map = doc.snap.read_map();
    doc.kb
        .open
        .and_then(|a| map.get(a))
        .filter(|a| a.is_analog())
        .or_else(|| map.actions().iter().find(|a| a.is_analog()))
        .map(|a| a.id)
}

fn disc(w: f32) -> f32 {
    (inner_w(w) * 0.3).min(ROW * 5.0)
}

pub(super) fn stick_h(doc: &mut InputMapDoc, ts: &mut TextSystem, w: f32) -> f32 {
    let iw = inner_w(w) - m::SEG_INSET * 2.0;
    let body = if stick_action(doc).is_some() {
        disc(w) + GAP + para_h(ts, tr("input_map_try.stick_caption"), m::LABEL_PX, iw)
    } else {
        para_h(ts, tr("input_map_try.no_stick"), m::VALUE_PX, inner_w(w))
    };
    m::CARD_PAD_TOP + line_h(m::LABEL_PX) + body + m::CARD_PAD_BOTTOM
}

/// O círculo (fora = o curso todo, disco = Dead, anel tracejado = Press, o ponto ao vivo) e as contas.
pub(super) fn stick(
    doc: &mut InputMapDoc,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let (x, w) = (r.x + PAD, inner_w(r.w));
    let mut y = r.y + m::CARD_PAD_TOP;
    let label = Rect::new(x, y, w, line_h(m::LABEL_PX));
    line(
        &mut doc.census,
        ts,
        scene,
        tr("input_map_try.stick"),
        label,
        m::LABEL_PX,
        ink.color(ColorToken::CardLabel),
    );
    y += line_h(m::LABEL_PX);
    let Some(a) = stick_action(doc)
        .and_then(|id| doc.snap.read_map().get(id))
        .cloned()
    else {
        para(
            doc,
            ts,
            scene,
            tr("input_map_try.no_stick"),
            (x, y, w),
            m::VALUE_PX,
            ink.color(ColorToken::Text2),
        );
        return;
    };
    let d = disc(r.w);
    let (cx, cy, rad) = (x + d * 0.5, y + d * 0.5, d * 0.5);
    let ring = |f: f32| -> Vec<(f32, f32)> {
        (0..=SIDES)
            .map(|i| {
                let t = std::f32::consts::TAU * i as f32 / SIDES as f32;
                (cx + rad * f * t.cos(), cy + rad * f * t.sin())
            })
            .collect()
    };
    fill_circle(scene, cx, cy, rad, ink.color(ColorToken::KanbanCard));
    stroke_polyline(
        scene,
        &ring(1.0),
        m::FIELD_BORDER,
        ink.color(ColorToken::Text2),
    );
    fill_circle(
        scene,
        cx,
        cy,
        rad * a.dead_zone,
        ink.color(ColorToken::CardField),
    );
    let pr = ring(a.press_point);
    for seg in pr.chunks(SIDES / DASHES).step_by(2) {
        stroke_polyline(
            scene,
            seg,
            m::FIELD_BORDER * 2.0,
            ink.color(ColorToken::Accent),
        );
    }
    let (px, py) = stick_xy(doc, &a);
    let raw = super::pad_raw(&doc.tryit.live_pad, &a);
    let color = ink.color(crate::palette::of_action(a.id));
    stroke_polyline(
        scene,
        &[(cx, cy), (cx + px * rad, cy + py * rad)],
        m::FIELD_BORDER * 2.0,
        color,
    );
    fill_circle(scene, cx + px * rad, cy + py * rad, m::SEG_PAD * 2.0, color);
    let rx = x + d + m::CARD_GAP;
    let rw = (x + w - rx).max(0.0);
    let half = a
        .bindings
        .iter()
        .find(|b| matches!(b, Binding::PadAxis { .. } | Binding::PadButton(_)));
    let title = tr("input_map_try.stick_of")
        .replace("{action}", &a.name)
        .replace(
            "{stick}",
            &half.map_or_else(String::new, |b| crate::view_list::footer::binding_label(*b)),
        );
    let mut ty = para(
        doc,
        ts,
        scene,
        &title,
        (rx, y, rw),
        m::LABEL_PX,
        ink.color(ColorToken::Text2),
    );
    crate::view_keyboard::travel::zone_fields(doc, &a, Rect::new(rx, ty, rw, 0.0), scene, ts, ink);
    ty += line_h(m::LABEL_PX) + ROW + GAP;
    let (s, p) = (
        strength_of(raw, a.dead_zone),
        pressed_of(raw, a.press_point),
    );
    let pressed = tr(if p {
        "input_map_try.pressed"
    } else {
        "input_map_try.not_pressed"
    });
    let read = format!(
        "{} {raw:.2} \u{00B7} {} {s:.2} \u{00B7} {pressed}",
        tr("input_map_try.pushed"),
        tr("input_map_try.strength")
    );
    let tok = if p {
        ColorToken::Success
    } else {
        ColorToken::Text1
    };
    para(
        doc,
        ts,
        scene,
        &read,
        (rx, ty, rw),
        m::VALUE_PX,
        ink.color(tok),
    );
    let cy2 = y + d + GAP;
    let cap = tr("input_map_try.stick_caption");
    let ch = para_h(ts, cap, m::LABEL_PX, w - m::SEG_INSET * 2.0);
    fill_rounded_rect(
        scene,
        Rect::new(x, cy2, w, ch),
        m::FIELD_RADIUS,
        ink.color(ColorToken::CardField),
    );
    para(
        doc,
        ts,
        scene,
        cap,
        (x + m::SEG_INSET, cy2, w - m::SEG_INSET * 2.0),
        m::LABEL_PX,
        ink.color(ColorToken::Text2),
    );
}

/// Onde o ponto está: a manche da 1.ª meia-ligação da acção (x, y em −1..1).
fn stick_xy(doc: &InputMapDoc, a: &ph2d_input::InputAction) -> (f32, f32) {
    use ph2d_input::GamepadAxis as A;
    let pad = &doc.tryit.live_pad;
    let axis = a.bindings.iter().find_map(|b| match b {
        Binding::PadAxis { axis, .. } => Some(*axis),
        _ => None,
    });
    let (ax, ay) = match axis {
        Some(A::RightStickX | A::RightStickY) => (A::RightStickX, A::RightStickY),
        Some(A::LeftStickX | A::LeftStickY) => (A::LeftStickX, A::LeftStickY),
        _ => return (super::pad_raw(pad, a), 0.0),
    };
    let v = |k| {
        let v = pad.axis(k);
        if v.is_finite() {
            v.clamp(-1.0, 1.0)
        } else {
            0.0
        }
    };
    (v(ax), -v(ay))
}
