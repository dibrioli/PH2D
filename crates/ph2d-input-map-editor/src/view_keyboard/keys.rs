//! O cartão de cima: a legenda, o `Keyboard | Gamepad`, o desenho (teclas com uma barra por acção, as da
//! acção aberta acesas), a fila «Also:» do que está ligado fora do desenho, e o pé da tecla sob o ponteiro.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, stroke_rounded_rect};
use ph2d_editor_core::panel::cards::{CardCtx, Seg, Widget};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::{ActionId, Binding};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use super::geometry::{self as g, KeyCap};
use super::state::{KbHit, Sub};
use super::{target, users};
use crate::doc::InputMapDoc;
use crate::view_list::footer::{binding_label, reader_names, readers_of};
use crate::view_list::text::line;

const CAPTION_H: f32 = m::FIELD_H - m::SEG_PAD;
/// A barra de uma acção numa tecla, e o vão entre barras.
pub(super) const BAR_H: f32 = m::SEG_PAD + m::FIELD_BORDER;
pub(super) const BAR_GAP: f32 = m::FIELD_BORDER;

/// O passo de tecla para a largura `w` (o mesmo que a pintura usa).
fn pitch(w: f32) -> f32 {
    (w / g::WIDTH).min(m::FIELD_H * g::PITCH_MAX_ROWS)
}

fn drawing_h(doc: &InputMapDoc, w: f32) -> f32 {
    match doc.kb.sub {
        Sub::Keyboard => pitch(w) * g::ROW_PITCH * g::ROWS + m::CARD_PAD_X * 2.0,
        Sub::Gamepad => super::pad::height(w),
    }
}

/// O que está ligado e o desenho não mostra (dígitos, Shift, Enter…, ou Mode e os cliques de manche).
pub(super) fn also(doc: &InputMapDoc) -> Vec<Binding> {
    let drawn: Vec<Binding> = match doc.kb.sub {
        Sub::Keyboard => g::keyboard().iter().map(|k| Binding::Key(k.key)).collect(),
        Sub::Gamepad => g::pad().iter().map(|p| p.binding).collect(),
    };
    let mut out: Vec<Binding> = Vec::new();
    for b in doc
        .snap
        .map
        .actions()
        .iter()
        .flat_map(|a| a.bindings.iter().copied())
    {
        let kind = matches!(b, Binding::Key(_)) == (doc.kb.sub == Sub::Keyboard);
        if kind && !drawn.iter().any(|d| super::same_input(*d, b)) && !out.contains(&b) {
            out.push(b);
        }
    }
    out
}

pub(super) fn top_height(doc: &InputMapDoc, w: f32) -> f32 {
    let inner = w - m::CARD_PAD_X * 2.0;
    let also_h = if also(doc).is_empty() {
        0.0
    } else {
        m::FIELD_H + m::FIELD_ROW_GAP
    };
    m::CARD_PAD_TOP
        + CAPTION_H
        + m::FIELD_H
        + m::FIELD_ROW_GAP * 2.0
        + drawing_h(doc, inner)
        + also_h
        + m::FIELD_H
        + m::CARD_PAD_BOTTOM
}

pub(super) fn paint_top(
    doc: &mut InputMapDoc,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let x = r.x + m::CARD_PAD_X;
    let w = (r.w - m::CARD_PAD_X * 2.0).max(0.0);
    let mut y = r.y + m::CARD_PAD_TOP;
    let cap = match doc.kb.sub {
        Sub::Keyboard => "input_map_keyboard.caption.keyboard",
        Sub::Gamepad => "input_map_keyboard.caption.gamepad",
    };
    line(
        &mut doc.census,
        ts,
        scene,
        tr(cap),
        Rect::new(x, y, w, CAPTION_H),
        m::LABEL_PX,
        ink.color(ColorToken::CardLabel),
    );
    y += CAPTION_H;
    let segs = [
        (Sub::Keyboard, "input_map_keyboard.sub.keyboard"),
        (Sub::Gamepad, "input_map_keyboard.sub.gamepad"),
    ]
    .map(|(s, k)| Seg {
        id: KbHit::Sub(s).id(),
        icon: None,
        text: tr(k),
        on: doc.kb.sub == s,
    });
    {
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, x, w);
        y = c.full(None, Widget::Choice(&segs), y) + m::FIELD_ROW_GAP;
    }
    doc.kb
        .targets
        .extend([KbHit::Sub(Sub::Keyboard), KbHit::Sub(Sub::Gamepad)]);
    let dh = drawing_h(doc, w);
    let draw = Rect::new(x, y, w, dh);
    fill_rounded_rect(
        scene,
        draw,
        m::FIELD_RADIUS,
        ink.color(ColorToken::KanbanCard),
    );
    match doc.kb.sub {
        Sub::Keyboard => paint_keyboard(doc, draw, scene, ts, ink),
        Sub::Gamepad => super::pad::paint(doc, draw, scene, ts, ink),
    }
    y += dh + m::FIELD_ROW_GAP;
    let extra = also(doc);
    if !extra.is_empty() {
        paint_also(doc, &extra, Rect::new(x, y, w, m::FIELD_H), scene, ts, ink);
        y += m::FIELD_H + m::FIELD_ROW_GAP;
    }
    let foot = Rect::new(x, y, w, m::FIELD_H);
    fill_rounded_rect(
        scene,
        foot,
        m::FIELD_RADIUS,
        ink.color(ColorToken::CardField),
    );
    let (lead, rest) = hover_text(doc);
    let inner = Rect::new(
        foot.x + m::SEG_INSET,
        foot.y,
        foot.w - m::SEG_INSET * 2.0,
        foot.h,
    );
    let lw = ts.prefix_width(&lead, m::VALUE_PX).min(inner.w);
    line(
        &mut doc.census,
        ts,
        scene,
        &lead,
        Rect::new(inner.x, inner.y, lw, inner.h),
        m::VALUE_PX,
        ink.color(ColorToken::Text1),
    );
    if !rest.is_empty() {
        let rx = inner.x + lw + m::SEG_ICON_GAP;
        let rr = Rect::new(rx, inner.y, (inner.x + inner.w - rx).max(0.0), inner.h);
        line(
            &mut doc.census,
            ts,
            scene,
            &rest,
            rr,
            m::VALUE_PX,
            ink.color(ColorToken::Text2),
        );
    }
}

/// O pé: «Up Arrow: jump (Platform Player) · move_up (Top-Down Player).», ou a dica quando nada está sob o ponteiro.
pub(crate) fn hover_text(doc: &InputMapDoc) -> (String, String) {
    let b = match doc.kb.hover {
        Some(KbHit::Key(k)) => Binding::Key(k),
        Some(KbHit::Pad(b)) => b,
        _ => return (tr("input_map_keyboard.hint").to_owned(), String::new()),
    };
    let label = binding_label(b);
    let who = users(doc, b);
    if who.is_empty() {
        return (
            tr("input_map_keyboard.unused").replace("{key}", &label),
            String::new(),
        );
    }
    let parts: Vec<String> = who
        .iter()
        .filter_map(|id| doc.snap.map.get(*id))
        .map(|a| {
            let rs = readers_of(&doc.snap, &a.name);
            let names = if rs.is_empty() {
                tr("input_map_keyboard.nobody").to_owned()
            } else {
                reader_names(rs)
            };
            tr("input_map_keyboard.user")
                .replace("{action}", &a.name)
                .replace("{who}", &names)
        })
        .collect();
    let lead = format!("{label}{}", tr("input_map_keyboard.colon"));
    let list = parts.join(tr("input_map_keyboard.sep"));
    (lead, format!("{list}{}", tr("input_map_list.period")))
}

/// Uma barra por acção que usa `b`, em baixo de `r`, na cor da acção.
pub(super) fn paint_bars(scene: &mut VectorScene, ink: CardInk, r: Rect, who: &[ActionId]) {
    let mut by = r.y + r.h - m::SEG_INSET * 0.5 - BAR_H;
    for id in who.iter().rev() {
        let bar = Rect::new(
            r.x + m::SEG_INSET * 0.5,
            by,
            (r.w - m::SEG_INSET).max(0.0),
            BAR_H,
        );
        fill_rounded_rect(
            scene,
            bar,
            BAR_H * 0.5,
            ink.color(crate::palette::of_action(*id)),
        );
        by -= BAR_H + BAR_GAP;
    }
}

fn paint_keyboard(
    doc: &mut InputMapDoc,
    draw: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let inner = Rect::new(
        draw.x + m::CARD_PAD_X,
        draw.y + m::CARD_PAD_X,
        draw.w - m::CARD_PAD_X * 2.0,
        draw.h - m::CARD_PAD_X * 2.0,
    );
    let p = pitch(inner.w);
    let open = doc
        .kb
        .open
        .and_then(|a| doc.snap.map.get(a))
        .map(|a| a.bindings.clone());
    for KeyCap {
        key,
        col,
        row,
        span,
    } in g::keyboard()
    {
        let kr = Rect::new(
            inner.x + col * p,
            inner.y + row * p * g::ROW_PITCH,
            (span - (1.0 - g::KEY_FILL)) * p,
            p * g::KEY_H,
        );
        let b = Binding::Key(key);
        let who = users(doc, b);
        let lit = open.as_ref().is_some_and(|o| o.contains(&b));
        let hover = doc.kb.hover == Some(KbHit::Key(key));
        let bg = if lit || hover {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(scene, kr, m::SEG_RADIUS, ink.color(bg));
        if lit {
            stroke_rounded_rect(
                scene,
                kr,
                m::SEG_RADIUS,
                m::FIELD_BORDER * 2.0,
                ink.color(ColorToken::Accent),
            );
        }
        let fg = ink.color(if who.is_empty() {
            ColorToken::Text2
        } else {
            ColorToken::Text1
        });
        let label_h = kr.h - (BAR_H + BAR_GAP) * who.len().max(1) as f32;
        if let Some(icon) = g::arrow_icon(key) {
            let s = m::KEY_MARK_BOX;
            let ir = Rect::new(kr.x + (kr.w - s) * 0.5, kr.y + (label_h - s) * 0.5, s, s);
            paint_icon(scene, icon, ir, fg, StrokeToken::Default.px());
        } else {
            let label = key.label();
            let tw = ts.prefix_width(&label, m::VALUE_PX).min(kr.w);
            let lr = Rect::new(kr.x + (kr.w - tw) * 0.5, kr.y, tw, label_h);
            line(&mut doc.census, ts, scene, &label, lr, m::VALUE_PX, fg);
        }
        paint_bars(scene, ink, kr, &who);
        doc.kb.bars.push((KbHit::Key(key), who));
        target(doc, KbHit::Key(key), kr);
    }
}

/// «Also:» e um chip por ligação fora do desenho (clicar acrescenta à aberta, como uma tecla desenhada).
fn paint_also(
    doc: &mut InputMapDoc,
    extra: &[Binding],
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let lead = tr("input_map_keyboard.also");
    let lw = ts.prefix_width(lead, m::LABEL_PX);
    line(
        &mut doc.census,
        ts,
        scene,
        lead,
        Rect::new(r.x, r.y, lw, r.h),
        m::LABEL_PX,
        ink.color(ColorToken::CardLabel),
    );
    let mut x = r.x + lw + m::SEG_ICON_GAP;
    for b in extra {
        let label = binding_label(*b);
        let who = users(doc, *b);
        let tw = ts.prefix_width(&label, m::VALUE_PX);
        let chip = Rect::new(x, r.y, tw + m::SEG_INSET * 2.0, r.h);
        if chip.x + chip.w > r.x + r.w {
            break;
        }
        fill_rounded_rect(scene, chip, m::SEG_RADIUS, ink.color(ColorToken::CardField));
        let lr = Rect::new(
            chip.x + m::SEG_INSET,
            chip.y,
            tw,
            chip.h - (BAR_H + BAR_GAP) * who.len() as f32,
        );
        line(
            &mut doc.census,
            ts,
            scene,
            &label,
            lr,
            m::VALUE_PX,
            ink.color(ColorToken::Text1),
        );
        paint_bars(scene, ink, chip, &who);
        let hit = match b {
            Binding::Key(k) => KbHit::Key(*k),
            other => KbHit::Pad(*other),
        };
        target(doc, hit, chip);
        x += chip.w + m::FIELD_GAP;
    }
}
