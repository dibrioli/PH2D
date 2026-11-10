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
use super::target;
use crate::doc::InputMapDoc;
use crate::ids::ListHit;
use crate::view_list::text::line;

const LABEL_H: f32 = m::LABEL_PX + m::LABEL_BOTTOM;

pub(super) fn paint(
    doc: &mut InputMapDoc,
    a: &InputAction,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
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
    y += LABEL_H;
    let bar = Rect::new(r.x, y, r.w, m::BAR_THUMB);
    fill_rounded_rect(scene, bar, m::SEG_RADIUS, ink.color(ColorToken::CardField));
    // A zona morta hachurada: um traço por quadrado da altura da barra.
    let dead_w = bar.w * a.dead_zone;
    let mut hx = bar.x;
    while hx < bar.x + dead_w {
        let cell = Rect::new(hx, bar.y, bar.h.min(bar.x + dead_w - hx), bar.h);
        fill_slash(
            scene,
            cell,
            m::FIELD_BORDER * 2.0,
            ink.color(ColorToken::Text2),
        );
        hx += bar.h;
    }
    doc.kb.bar = Some((bar.x, bar.w));
    for (hit, v, tok) in [
        (KbHit::BarDead, a.dead_zone, ColorToken::Text1),
        (KbHit::BarPress, a.press_point, ColorToken::Accent),
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
        target(doc, hit, grab);
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
            ink.color(ColorToken::Text2),
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
}

/// Os dois campos «Dead zone (0 to 1)» / «Press at (0 to 1)» numa linha a partir de `r.y` (A, B e C usam ESTES).
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
            Rect::new(x, y, half, LABEL_H),
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
        let vr = Rect::new(f.x + m::SEG_INSET, f.y, f.w - m::SEG_INSET * 2.0, f.h);
        line(
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
