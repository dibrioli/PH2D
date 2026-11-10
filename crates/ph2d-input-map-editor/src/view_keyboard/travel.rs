//! A barra do curso «The stick, from rest to full» (0 · Dead · Press · 1, zona morta hachurada) e os dois campos
//! «Dead zone (0 to 1)» / «Press at (0 to 1)». As marcas arrastam-se pela mesma lei dos campos (`SetZone`).

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::paint_shapes::fill_slash;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::InputAction;
use ph2d_text::TextSystem;
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;

use super::state::KbHit;
use crate::interact::{Control, ControlRole};
use crate::doc::InputMapDoc;
use crate::ids::ListHit;
use crate::view_list::text::line;

const LABEL_H: f32 = m::LABEL_PX + m::LABEL_BOTTOM;
/// A barra do curso: meia linha de campo (maquete B3).
const BAR_H: f32 = m::FIELD_H * 0.5;

pub(super) fn paint(
    doc: &mut InputMapDoc,
    a: &InputAction,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) -> f32 {
    let mut y = r.y;
    line(
        &mut doc.census,
        ts,
        scene,
        tr("input_map_keyboard.travel"),
        Rect::new(r.x, y, r.w, LABEL_H),
        m::LABEL_PX,
        ink.color(ColorToken::CardLabel),
    );
    y += LABEL_H + m::SEG_PAD * 2.0;
    let bar = Rect::new(r.x, y, r.w, BAR_H);
    fill_rounded_rect(scene, bar, m::SEG_RADIUS, ink.color(ColorToken::CardField));
    // De Dead a 1 a barra clareia (o curso que conta); a zona morta fica hachurada, suave.
    let dead_w = bar.w * a.dead_zone;
    let live = Rect::new(bar.x + dead_w, bar.y, (bar.w - dead_w).max(0.0), bar.h);
    fill_rounded_rect(
        scene,
        live,
        m::SEG_RADIUS,
        ink.color(ColorToken::KanbanCardRaised),
    );
    let mut hx = bar.x;
    while hx < bar.x + dead_w {
        let cell = Rect::new(hx, bar.y, bar.h.min(bar.x + dead_w - hx), bar.h);
        fill_slash(scene, cell, m::FIELD_BORDER, ink.color(ColorToken::Text3));
        hx += bar.h * 0.5;
    }
    doc.kb.bar = Some((bar.x, bar.w));
    for (hit, v, tok, label) in [
        (KbHit::BarDead, a.dead_zone, ColorToken::CardLabel, "input_map_keyboard.bar_dead"),
        (KbHit::BarPress, a.press_point, ColorToken::Accent, "input_map_keyboard.bar_press"),
    ] {
        let mx = bar.x + bar.w * v;
        let mark = Rect::new(
            mx - m::FIELD_BORDER,
            bar.y - m::SEG_PAD,
            m::FIELD_BORDER * 2.0,
            bar.h + m::SEG_PAD * 2.0,
        );
        fill_rounded_rect(scene, mark, m::FIELD_BORDER, ink.color(tok));
        let grab = Rect::new(mx - m::BAR_THUMB * 0.5, bar.y, m::BAR_THUMB, bar.h);
        doc.kb.targets.push(hit);
        doc.control(Control::new(hit.id(), grab, ControlRole::DragHandle, tr(label)));
    }
    y += bar.h + m::SEG_PAD;
    let ticks = [
        (0.0, tr("input_map_keyboard.zero").to_owned()),
        (
            a.dead_zone,
            tr("input_map_keyboard.travel_dead").replace("{v}", &format!("{:.2}", a.dead_zone)),
        ),
        (
            a.press_point,
            tr("input_map_keyboard.travel_press").replace("{v}", &format!("{:.2}", a.press_point)),
        ),
        (1.0, tr("input_map_keyboard.one").to_owned()),
    ];
    for (v, s) in ticks {
        let tw = ts.prefix_width(&s, m::LABEL_PX);
        let tx = (bar.x + bar.w * v - tw * 0.5).clamp(bar.x, bar.x + bar.w - tw);
        line(
            &mut doc.census,
            ts,
            scene,
            &s,
            Rect::new(tx, y, tw, LABEL_H),
            m::LABEL_PX,
            ink.color(ColorToken::CardLabel),
        );
    }
    // O comando AO VIVO: a marca de onde a manche está agora (o mesmo cru que o resolvedor lê).
    let live = crate::view_try::pad_raw(&doc.tryit.live_pad, a);
    if live > 0.0 {
        let lx = bar.x + bar.w * live;
        let dot = Rect::new(lx - bar.h * 0.5, bar.y, bar.h, bar.h);
        fill_rounded_rect(scene, dot, bar.h * 0.5, ink.color(ColorToken::Accent));
        doc.kb.live_dot = Some(dot);
    }
    y += LABEL_H + m::FIELD_ROW_GAP;
    zone_fields(doc, a, Rect::new(r.x, y, r.w, 0.0), scene, ts, ink);
    y + LABEL_H + m::FIELD_H
}

/// Os dois campos «Dead zone (0 to 1)» / «Press at (0 to 1)» numa linha a partir de `r.y` (A, B e C usam ESTES).
/// O valor de um campo Dead/Press com a marca «D»/«P» à frente (maquete C3): UM pintor para A, B e C.
pub(crate) fn zone_value(
    doc: &mut InputMapDoc,
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    ink: CardInk,
    f: Rect,
    dead: bool,
    v: &str,
) {
    let mark = tr(if dead {
        "input_map_try.dead_mark"
    } else {
        "input_map_try.press_mark"
    });
    let x = f.x + m::SEG_INSET;
    let mw = ts.prefix_width(mark, m::VALUE_PX);
    let mr = Rect::new(x, f.y, mw, f.h);
    line(
        &mut doc.census,
        ts,
        scene,
        mark,
        mr,
        m::VALUE_PX,
        ink.color(ColorToken::CardLabel),
    );
    let vx = x + mw + m::SEG_INSET;
    let vr = Rect::new(vx, f.y, (f.x + f.w - m::SEG_INSET - vx).max(0.0), f.h);
    line(
        &mut doc.census,
        ts,
        scene,
        v,
        vr,
        m::VALUE_PX,
        ink.color(ColorToken::Text1),
    );
}

pub(crate) fn zone_fields(
    doc: &mut InputMapDoc,
    a: &InputAction,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let y = r.y;
    let half = (r.w - m::FIELD_GAP) * 0.5;
    for (i, (hit, label, value)) in [
        (ListHit::Dead(a.id), "input_map_list.dead", a.dead_zone),
        (ListHit::Press(a.id), "input_map_list.press", a.press_point),
    ]
    .into_iter()
    .enumerate()
    {
        let x = r.x + (half + m::FIELD_GAP) * i as f32;
        line(
            &mut doc.census,
            ts,
            scene,
            tr(label),
            Rect::new(x, y, half, m::LABEL_PX),
            m::LABEL_PX,
            ink.color(ColorToken::CardLabel),
        );
        let f = Rect::new(x, y + LABEL_H, half, m::FIELD_H);
        fill_rounded_rect(scene, f, m::FIELD_RADIUS, ink.color(ColorToken::CardField));
        let typing = doc.list.typing.as_ref().filter(|t| {
            t.action == a.id
                && (t.zone == crate::view_list::state::Zone::Dead)
                    == matches!(hit, ListHit::Dead(_))
        });
        if typing.is_some() {
            crate::view_list::ring(scene, f, ink);
        }
        let v = typing.map_or_else(|| format!("{value:.2}"), |t| t.buffer.clone());
        let dead = matches!(hit, ListHit::Dead(_));
        zone_value(doc, ts, scene, ink, f, dead, &v);
        doc.hits.register(hit.id(), f);
        doc.census.numbers.push(a.id);
    }
}
