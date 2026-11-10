//! O comando desenhado: o contorno do corpo, gatilhos e ombros, as duas manches (cada uma com as quatro metades
//! como setas à volta: clicar numa seta liga essa metade, sem adivinhar pelo nome), ABXY e a cruz.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_icon, stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use super::geometry::{self as g, PadShape};
use super::keys::paint_bars;
use super::state::KbHit;
use super::{target, users};
use crate::doc::InputMapDoc;
use crate::view_list::text::line;

/// A altura do desenho para a largura `w` (a caixa mantém a proporção da maquete).
pub(super) fn height(w: f32) -> f32 {
    (w / g::PAD_ASPECT).min(m::FIELD_H * g::PITCH_MAX_ROWS * g::WIDTH / g::PAD_ASPECT / 2.0)
}

pub(super) fn paint(
    doc: &mut InputMapDoc,
    draw: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let bh = draw.h - m::CARD_PAD_X * 2.0;
    let bw = bh * g::PAD_ASPECT;
    let ox = draw.x + (draw.w - bw) * 0.5;
    let oy = draw.y + m::CARD_PAD_X;
    let at = |fx: f32, fy: f32| (ox + fx * bw, oy + fy * bh);
    let mut outline: Vec<(f32, f32)> = g::BODY.iter().map(|(fx, fy)| at(*fx, *fy)).collect();
    outline.extend(g::BODY.iter().rev().map(|(fx, fy)| at(1.0 - fx, *fy)));
    stroke_polyline(
        scene,
        &outline,
        m::FIELD_BORDER * 2.0,
        ink.color(ColorToken::Text2),
    );
    for ((fx, fy), key) in g::STICKS {
        let (cx, cy) = at(fx, fy);
        let r = g::STICK_R * bh;
        fill_circle(scene, cx, cy, r, ink.color(ColorToken::CardField));
        let label = tr(key);
        let tw = ts.prefix_width(label, m::VALUE_PX);
        line(
            &mut doc.census,
            ts,
            scene,
            label,
            Rect::new(cx - tw * 0.5, cy - r, tw, r * 2.0),
            m::VALUE_PX,
            ink.color(ColorToken::Text1),
        );
    }
    let open = doc
        .kb
        .open
        .and_then(|a| doc.snap.map.get(a))
        .map(|a| a.bindings.clone())
        .unwrap_or_default();
    for p in g::pad() {
        let (cx, cy) = at(p.cx, p.cy);
        let (hw, hh) = (p.hw * bw, p.hh * bh);
        let r = Rect::new(cx - hw, cy - hh, hw * 2.0, hh * 2.0);
        let who = users(doc, p.binding);
        let lit = open.iter().any(|b| super::same_input(p.binding, *b));
        let hover = doc.kb.hover == Some(KbHit::Pad(p.binding));
        let bg = ink.color(if lit || hover {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        });
        let radius = match p.shape {
            PadShape::Round => hh.min(hw),
            PadShape::Pill => hh,
            PadShape::Arrow(_) => m::SEG_RADIUS,
        };
        fill_rounded_rect(scene, r, radius, bg);
        if lit {
            stroke_rounded_rect(
                scene,
                r,
                radius,
                m::FIELD_BORDER * 2.0,
                ink.color(ColorToken::Accent),
            );
        }
        let fg = ink.color(if who.is_empty() {
            ColorToken::Text2
        } else {
            ColorToken::Text1
        });
        if let PadShape::Arrow(icon) = p.shape {
            let s = m::KEY_MARK_BOX.min(r.w).min(r.h);
            paint_icon(
                scene,
                icon,
                Rect::new(cx - s * 0.5, cy - s * 0.5, s, s),
                fg,
                StrokeToken::Default.px(),
            );
        } else {
            let label = tr(p.short_key);
            let tw = ts.prefix_width(label, m::LABEL_PX).min(r.w);
            line(
                &mut doc.census,
                ts,
                scene,
                label,
                Rect::new(cx - tw * 0.5, r.y, tw, r.h),
                m::LABEL_PX,
                fg,
            );
        }
        let bars = Rect::new(
            r.x,
            r.y + r.h,
            r.w,
            (super::keys::BAR_H + super::keys::BAR_GAP) * who.len() as f32 + m::SEG_INSET * 0.5,
        );
        paint_bars(scene, ink, bars, &who);
        target(doc, KbHit::Pad(p.binding), r);
    }
}
