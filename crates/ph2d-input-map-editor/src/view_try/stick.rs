//! The stick (C3): o círculo da manche — fora = o curso todo, disco = Dead, anel tracejado = Press, o ponto ao vivo.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use super::cards::{GAP, PAD, ROW, inner_w};
use super::{line_h, para, para_h};
use crate::doc::InputMapDoc;
// O corpo da C no peso Regular (como a A e a B); só os destaques e os nomes de acção vão em SemiBold.
use crate::view_list::text::body as line;
use ph2d_editor_core::paint::{fill_circle, fill_rounded_rect, paint_icon, stroke_polyline};
use ph2d_input::resolve::{pressed_of, strength_of};
use ph2d_input::{ActionId, Binding};

/// Quantos traços tem o anel tracejado, e quantos lados o círculo.
const DASHES: usize = 24;
const SIDES: usize = 64;

/// As tintas do círculo (maquete C3): o curso todo escuro, o disco Dead num cinza MAIS CLARO, o ponto ao vivo e o
/// raio até ele no destaque da aba. O disco é o cinza neutro do Kanban (69 79 89 no escuro, um degrau claro e
/// menos azul que o `kanban-card-raised`, que mal se via): os `bg-3`/`bg-elev` mudam com o tema e no Forge ficam
/// MAIS ESCUROS que o cartão, que é sempre o do Kanban.
pub const OUTER_INK: ColorToken = ColorToken::KanbanCard;
pub const DEAD_INK: ColorToken = ColorToken::KanbanBlackSubtle;
pub const DOT_INK: ColorToken = ColorToken::InputMapHighlight;

/// O raio do ponto ao vivo em fracção do raio do círculo (maquete C3, medido: ponto de 14 px num círculo de 138 px).
pub const DOT_OF_RADIUS: f32 = 0.1;

/// A altura do bloco de texto ao lado do círculo: o título, o vão de linha do cartão, os números Dead/Press e a
/// leitura.
fn block_h() -> f32 {
    line_h(m::LABEL_PX) + GAP + line_h(m::LABEL_PX) + ROW + GAP + line_h(m::VALUE_PX)
}

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
        disc(w).max(block_h())
            + GAP
            + para_h(ts, tr("input_map_try.stick_caption"), m::LABEL_PX, iw)
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
    fill_circle(scene, cx, cy, rad, ink.color(OUTER_INK));
    stroke_polyline(
        scene,
        &ring(1.0),
        m::FIELD_BORDER,
        ink.color(ColorToken::Text2),
    );
    fill_circle(scene, cx, cy, rad * a.dead_zone, ink.color(DEAD_INK));
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
    let color = ink.color(DOT_INK);
    stroke_polyline(
        scene,
        &[(cx, cy), (cx + px * rad, cy + py * rad)],
        m::FIELD_BORDER * 2.0,
        color,
    );
    let dot = rad * DOT_OF_RADIUS;
    fill_circle(scene, cx + px * rad, cy + py * rad, dot, color);
    let rx = x + d + m::CARD_GAP;
    let rw = (x + w - rx).max(0.0);
    // O bloco de texto ao centro da altura do círculo (maquete C3).
    let disc_top = y;
    let y = disc_top + ((d - block_h()) * 0.5).max(0.0);
    let half = a
        .bindings
        .iter()
        .find(|b| matches!(b, Binding::PadAxis { .. } | Binding::PadButton(_)));
    let title = tr("input_map_try.stick_of")
        .replace("{action}", &a.name)
        .replace(
            "{stick}",
            &half.map_or_else(String::new, |b| stick_name(*b)),
        );
    // «move_left · Left Stick ←»: a direcção é um ÍCONE (a mesma porta do ladrilho da B), nunca um glifo.
    let icon = half.and_then(|b| crate::view_keyboard::half_icon(*b));
    let iw = icon.map_or(0.0, |_| m::LABEL_PX + m::SEG_PAD);
    let tw = ts.prefix_width(&title, m::LABEL_PX).min((rw - iw).max(0.0));
    let lh = line_h(m::LABEL_PX);
    let fg = ink.color(ColorToken::CardLabel);
    line(
        &mut doc.census,
        ts,
        scene,
        &title,
        Rect::new(rx, y, tw, lh),
        m::LABEL_PX,
        fg,
    );
    if let Some(icon) = icon {
        let s = m::LABEL_PX;
        let ir = Rect::new(rx + tw + m::SEG_PAD, y + (lh - s) * 0.5, s, s);
        paint_icon(scene, icon, ir, fg, StrokeToken::Default.px());
        doc.census.icons.push((ir, icon));
    }
    let mut ty = y + lh + GAP;
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
    // «Pushed 0.62  Strength 0.55  Pressed» (maquete C3): os rótulos no texto calmo e no Regular, os valores em
    // negrito no texto normal (a leitura É o destaque do cartão), e SÓ o «Pressed» na cor do sucesso.
    let state = if p {
        ColorToken::Success
    } else {
        ColorToken::Text2
    };
    let runs = [
        (
            tr("input_map_try.pushed").to_owned(),
            ColorToken::Text2,
            false,
        ),
        (format!("{raw:.2}"), ColorToken::Text1, true),
        (
            tr("input_map_try.strength").to_owned(),
            ColorToken::Text2,
            false,
        ),
        (format!("{s:.2}"), ColorToken::Text1, true),
        (pressed.to_owned(), state, p),
    ];
    let mut cx = rx;
    let vh = line_h(m::VALUE_PX);
    for (i, (t, tok, bold)) in runs.iter().enumerate() {
        cx += match i {
            0 => 0.0,
            _ if i % 2 == 1 => m::SEG_PAD * 2.0,
            _ => m::CARD_GAP,
        };
        let room = (rx + rw - cx).max(0.0);
        let w = if *bold {
            crate::view_list::text::title_width(ts, t, m::VALUE_PX)
        } else {
            ts.prefix_width(t, m::VALUE_PX)
        }
        .min(room);
        let r = Rect::new(cx, ty, w, vh);
        let c = ink.color(*tok);
        if *bold {
            crate::view_list::text::title(&mut doc.census, ts, scene, t, r, m::VALUE_PX, c);
        } else {
            line(&mut doc.census, ts, scene, t, r, m::VALUE_PX, c);
        }
        doc.census.runs.push((t.clone(), *tok, *bold));
        cx += w;
    }
    let cy2 = disc_top + d.max(block_h()) + GAP;
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

/// O nome da manche sem o eixo («Left Stick»); um gatilho ou botão fica com o nome dele.
fn stick_name(b: Binding) -> String {
    use ph2d_input::GamepadAxis as A;
    match b {
        Binding::PadAxis {
            axis: A::LeftStickX | A::LeftStickY,
            ..
        } => tr("input_map_try.left_stick").to_owned(),
        Binding::PadAxis {
            axis: A::RightStickX | A::RightStickY,
            ..
        } => tr("input_map_try.right_stick").to_owned(),
        other => crate::view_list::footer::binding_label(other),
    }
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
