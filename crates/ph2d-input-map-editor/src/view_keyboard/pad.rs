//! O comando desenhado (maquete B3): o corpo em curvas com as pegas, os gatilhos na aresta do topo e os ombros
//! logo abaixo, as duas manches como anéis (as quatro metades são setas pequenas à volta: clicar numa liga essa metade), ABXY em
//! losango, a cruz, Back e Start. Por baixo de cada alvo, uma barra por acção — a mesma lei das teclas.
//! Desvio documentado da maquete B3 (que só desenha LT/RT, as manches e ABXY): LB/RB, Back/Start e a cruz ficam
//! no desenho porque são botões que se ligam — sem eles, uma ligação a um deles não teria onde se clicar.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::ActionId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::{Affine, BezPath, Circle, Stroke, VectorScene};

use super::geometry::{self as g, BodySeg, PadShape};
use super::keys::{BAR_GAP, BAR_H, face_of, open_outline, paint_bars};
use super::state::KbHit;
use super::users;
use crate::doc::InputMapDoc;
use crate::interact::{Control, ControlRole};
use crate::view_list::text::line;

/// A altura do desenho para a largura `w`: o comando enche a largura como na maquete, até ao tecto (mais baixo
/// com a B empilhada).
pub(super) fn height(w: f32, stacked: bool) -> f32 {
    let rows = if stacked {
        g::PAD_MAX_ROWS_STACKED
    } else {
        g::PAD_MAX_ROWS
    };
    (w * g::PAD_FILL / g::PAD_ASPECT).min(m::FIELD_H * rows) + m::CARD_PAD_X * 2.0
}

/// O contorno do corpo, já em pixels: o lado esquerdo e o espelho dele, de volta ao meio do topo.
fn body(at: &dyn Fn(f32, f32) -> (f64, f64)) -> BezPath {
    let mirror = |(x, y): (f32, f32)| at(1.0 - x, y);
    let pt = |(x, y): (f32, f32)| at(x, y);
    let mut path = BezPath::new();
    path.move_to(pt(g::BODY_START));
    let mut ends = vec![g::BODY_START];
    for s in g::BODY {
        match s {
            BodySeg::Line(e) => {
                path.line_to(pt(e));
                ends.push(e);
            }
            BodySeg::Cubic(c1, c2, e) => {
                path.curve_to(pt(c1), pt(c2), pt(e));
                ends.push(e);
            }
        }
    }
    for (i, s) in g::BODY.iter().enumerate().rev() {
        let start = ends[i];
        match *s {
            BodySeg::Line(_) => path.line_to(mirror(start)),
            BodySeg::Cubic(c1, c2, _) => path.curve_to(mirror(c2), mirror(c1), mirror(start)),
        }
    }
    path.close_path();
    path
}

/// As barras por baixo de `r` (uma por acção, na cor dela).
fn bars_under(
    doc: &mut InputMapDoc,
    scene: &mut VectorScene,
    ink: CardInk,
    r: Rect,
    who: &[ActionId],
) {
    let h = (BAR_H + BAR_GAP) * who.len() as f32 + m::SEG_INSET * 0.5;
    paint_bars(
        &mut doc.census,
        scene,
        ink,
        Rect::new(r.x, r.y + r.h, r.w, h),
        who,
    );
}

pub(super) fn paint(
    doc: &mut InputMapDoc,
    draw: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let bh = (draw.h - m::CARD_PAD_X * 2.0).max(0.0);
    let bw = bh * g::PAD_ASPECT;
    let ox = draw.x + (draw.w - bw) * 0.5;
    let oy = draw.y + m::CARD_PAD_X;
    let at = |fx: f32, fy: f32| (ox + fx * bw, oy + fy * bh);
    let at64 = |fx: f32, fy: f32| {
        let (x, y) = at(fx, fy);
        (f64::from(x), f64::from(y))
    };
    let outline = body(&at64);
    let edge = Stroke::new(f64::from(m::FIELD_BORDER * 2.0));
    scene.inner_mut().stroke(
        &edge,
        Affine::IDENTITY,
        ink.color(ColorToken::Text3),
        None,
        &outline,
    );
    let open = doc
        .kb
        .open
        .and_then(|a| doc.snap.map.get(a))
        .map(|a| a.bindings.clone())
        .unwrap_or_default();
    let parts = g::pad();
    // A cruz inteira por baixo dos quatro braços.
    let ((dx, dy), step, (ahw, ahh)) = g::dpad();
    let (cx, cy) = at(dx, dy);
    let (reach, half_w) = ((step + ahh) * bh, ahw * bh);
    for r in [
        Rect::new(cx - half_w, cy - reach, half_w * 2.0, reach * 2.0),
        Rect::new(cx - reach, cy - half_w, reach * 2.0, half_w * 2.0),
    ] {
        fill_rounded_rect(
            scene,
            r,
            m::SEG_RADIUS,
            ink.color(ColorToken::KanbanCardRaised),
        );
    }
    // Os anéis das manches: aceso inteiro quando a aberta usa uma metade dele; as barras ficam em cada metade.
    for (i, ((fx, fy), key, fr)) in g::STICKS.into_iter().enumerate() {
        let (cx, cy) = at(fx, fy);
        let r = fr * bh;
        let halves: Vec<&g::PadPart> = parts
            .iter()
            .filter(|p| matches!(p.shape, PadShape::Half(_, s) if s == i))
            .collect();
        let lit = halves
            .iter()
            .any(|p| open.iter().any(|b| super::same_input(p.binding, *b)));
        let used = halves.iter().any(|p| !users(doc, p.binding).is_empty());
        let ring = Circle::new((f64::from(cx), f64::from(cy)), f64::from(r));
        let face = ColorToken::KanbanCardRaised;
        scene.inner_mut().fill(
            ph2d_vector::Fill::NonZero,
            Affine::IDENTITY,
            ink.color(face),
            None,
            &ring,
        );
        let (tok, wpx) = if lit {
            (ColorToken::Accent, m::FIELD_BORDER * 3.0)
        } else {
            (ColorToken::Text3, m::FIELD_BORDER * 2.0)
        };
        scene.inner_mut().stroke(
            &Stroke::new(f64::from(wpx)),
            Affine::IDENTITY,
            ink.color(tok),
            None,
            &ring,
        );
        let label = tr(key);
        let tw = ts.prefix_width(label, m::VALUE_PX);
        let fg = if !used {
            ColorToken::Text3
        } else {
            ColorToken::Text1
        };
        line(
            &mut doc.census,
            ts,
            scene,
            label,
            Rect::new(cx - tw * 0.5, cy - r, tw, r * 2.0),
            m::VALUE_PX,
            ink.color(fg),
        );
    }
    for p in &parts {
        let (cx, cy) = at(p.cx, p.cy);
        let (mut hw, hh) = (p.hw * bh, p.hh * bh);
        if matches!(p.shape, PadShape::Pill) {
            hw = hw.max(ts.prefix_width(tr(p.short_key), m::LABEL_PX) * 0.5 + m::SEG_INSET);
        }
        let r = Rect::new(cx - hw, cy - hh, hw * 2.0, hh * 2.0);
        let who = users(doc, p.binding);
        let lit = open.iter().any(|b| super::same_input(p.binding, *b));
        let hover = doc.kb.hover == Some(KbHit::Pad(p.binding));
        let radius = match p.shape {
            PadShape::Round | PadShape::Pill => hh,
            PadShape::Arm(_) | PadShape::Half(..) => m::SEG_RADIUS,
        };
        let id = KbHit::Pad(p.binding).id();
        let label = crate::view_list::footer::binding_label(p.binding);
        doc.control(Control::new(id, r, ControlRole::Key, label));
        let pressed = doc.is_pressed(id);
        // A mesma precedência das teclas (`face_of`); a seta de uma metade só ganha face quando premida ou sob o
        // ponteiro (em repouso é uma seta solta à volta do anel, maquete B3).
        let half = matches!(p.shape, PadShape::Half(..));
        let face = (!half || pressed || hover).then(|| face_of(pressed, None, hover));
        if let Some(f) = face {
            super::keys::fill_face(scene, ink, r, radius, f);
        }
        if lit && !half {
            open_outline(scene, ink, r, radius);
        }
        let fg = ink.color(if pressed {
            ColorToken::Bg0
        } else if lit && half {
            ColorToken::Accent
        } else if who.is_empty() {
            ColorToken::Text3
        } else {
            ColorToken::Text1
        });
        match p.shape {
            PadShape::Arm(icon) | PadShape::Half(icon, _) => {
                let s = super::keys::ARROW_BOX.min(r.w).min(r.h);
                let ir = Rect::new(cx - s * 0.5, cy - s * 0.5, s, s);
                // A seta é traço fino: nunca mais apagada que `Text2` (a regra dos símbolos das teclas).
                let tone = if who.is_empty() && !pressed && !lit {
                    ink.color(ColorToken::Text2)
                } else {
                    fg
                };
                paint_icon(scene, icon, ir, tone, StrokeToken::Default.px());
            }
            PadShape::Pill | PadShape::Round => {
                let label = tr(p.short_key);
                let tw = ts.prefix_width(label, m::LABEL_PX).min(r.w);
                let lr = Rect::new(cx - tw * 0.5, r.y, tw, r.h);
                line(&mut doc.census, ts, scene, label, lr, m::LABEL_PX, fg);
            }
        }
        bars_under(doc, scene, ink, r, &who);
        doc.kb.bars.push((KbHit::Pad(p.binding), who));
        doc.kb.targets.push(KbHit::Pad(p.binding));
    }
}
