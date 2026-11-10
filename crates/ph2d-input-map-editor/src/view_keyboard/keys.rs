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
use crate::view_list::text::{line, title};

const CAPTION_H: f32 = m::FIELD_H - m::SEG_PAD;
/// A barra de uma acção numa tecla, e o vão entre barras.
pub(super) const BAR_H: f32 = m::SEG_PAD + m::FIELD_BORDER;
pub(super) const BAR_GAP: f32 = m::FIELD_BORDER;
/// A caixa da seta desenhada (teclas, ladrilhos, comando): da altura da letra, para ler como ↑ e não como ponto.
pub(super) const ARROW_BOX: f32 = m::VALUE_PX + m::SEG_PAD * 2.0;

/// O passo de tecla para a largura `w`, até `cap` alturas de campo.
fn pitch_upto(w: f32, cap: f32) -> f32 {
    (w / g::WIDTH).min(m::FIELD_H * cap)
}

/// A tecla nunca é mais baixa que a letra com duas barras por baixo (a 1280 as barras tapavam a letra).
const KEY_MIN_H: f32 = m::VALUE_PX + m::SEG_PAD + (BAR_H + BAR_GAP) * 2.0 + m::SEG_INSET * 0.5;

/// A altura de uma tecla para o passo `p`.
fn key_height(p: f32) -> f32 {
    (p * g::KEY_H).max(KEY_MIN_H)
}

/// A altura das quatro filas para o passo `p` (a tecla e o vão do passo).
fn rows_h(p: f32) -> f32 {
    (key_height(p) + p * (g::ROW_PITCH - g::KEY_H)) * g::ROWS
}

/// O passo cujas filas medem `h` (o inverso de [`rows_h`]): a pintura tira o passo da caixa que recebe.
fn pitch_for_rows(h: f32) -> f32 {
    let linear = h / (g::ROW_PITCH * g::ROWS);
    if linear * g::KEY_H >= KEY_MIN_H {
        linear
    } else {
        (h / g::ROWS - KEY_MIN_H).max(0.0) / (g::ROW_PITCH - g::KEY_H)
    }
}

/// A altura do desenho do teclado para a largura `w` (a vista C reusa-o).
pub(crate) fn keyboard_h(w: f32) -> f32 {
    rows_h(pitch_upto(w, g::PITCH_MAX_ROWS)) + m::CARD_PAD_X * 2.0
}

/// O teclado da B enche a largura do cartão (maquete B1): o tecto do passo é mais alto que o da C.
fn keyboard_h_b(w: f32) -> f32 {
    rows_h(pitch_upto(w - m::CARD_PAD_X * 2.0, g::PITCH_MAX_ROWS_B)) + m::CARD_PAD_X * 2.0
}

fn drawing_h(doc: &InputMapDoc, w: f32) -> f32 {
    match doc.kb.sub {
        Sub::Keyboard => keyboard_h_b(w),
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
        m::FIELD_ROW_GAP + m::FIELD_H
    };
    m::CARD_PAD_TOP
        + CAPTION_H
        + m::FIELD_H
        + m::FIELD_ROW_GAP
        + drawing_h(doc, inner)
        + also_h
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
    let cap_r = Rect::new(x, y, w, CAPTION_H);
    // Desvio documentado da maquete (que põe a frase numa linha de baixo): a frase do hover SUBSTITUI a legenda,
    // na mesma altura — o cartão não salta ao passar o ponteiro pelas teclas.
    if let Some((lead, rest)) = hover_text(doc) {
        let lw = title(
            &mut doc.census,
            ts,
            scene,
            &lead,
            cap_r,
            m::LABEL_PX,
            ink.color(ColorToken::Text1),
        );
        let rx = x + lw + m::SEG_PAD * 2.0;
        let rr = Rect::new(rx, y, (x + w - rx).max(0.0), CAPTION_H);
        line(
            &mut doc.census,
            ts,
            scene,
            &rest,
            rr,
            m::LABEL_PX,
            ink.color(ColorToken::Text2),
        );
    } else {
        let cap = match doc.kb.sub {
            Sub::Keyboard => "input_map_keyboard.caption.keyboard",
            Sub::Gamepad => "input_map_keyboard.caption.gamepad",
        };
        line(
            &mut doc.census,
            ts,
            scene,
            tr(cap),
            cap_r,
            m::LABEL_PX,
            ink.color(ColorToken::CardLabel),
        );
    }
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
        ink.color(ColorToken::CardField),
    );
    match doc.kb.sub {
        Sub::Keyboard => paint_keyboard(doc, draw, scene, ts, ink),
        Sub::Gamepad => super::pad::paint(doc, draw, scene, ts, ink),
    }
    y += dh;
    let extra = also(doc);
    if !extra.is_empty() {
        y += m::FIELD_ROW_GAP;
        paint_also(doc, &extra, Rect::new(x, y, w, m::FIELD_H), scene, ts, ink);
    }
}

/// A frase do hover, no LUGAR da legenda (zero salto): «Up Arrow: jump (Platform Player) · move_up (Top-Down
/// Player).»; `None` sem tecla nem botão sob o ponteiro.
pub(crate) fn hover_text(doc: &InputMapDoc) -> Option<(String, String)> {
    let b = match doc.kb.hover {
        Some(KbHit::Key(k)) => Binding::Key(k),
        Some(KbHit::Pad(b)) => b,
        _ => return None,
    };
    let label = binding_label(b);
    let who = users(doc, b);
    if who.is_empty() {
        let lead = format!("{label}{}", tr("input_map_keyboard.colon"));
        return Some((lead, tr("input_map_keyboard.unused").to_owned()));
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
    Some((lead, format!("{list}{}", tr("input_map_list.period"))))
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
    let open = doc
        .kb
        .open
        .and_then(|a| doc.snap.map.get(a))
        .map(|a| a.bindings.clone())
        .unwrap_or_default();
    paint_keys(doc, draw, scene, ts, ink, Lit::Outline, &|k| {
        open.contains(&Binding::Key(k))
    });
}

/// Como uma tecla acesa se mostra: na B, a da acção ABERTA ganha o contorno; no Try it, a PREMIDA enche-se da cor
/// da primeira acção que ela liga (maquete C2: a ↑ amarela do jump), com as barras de todas por baixo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Lit {
    Outline,
    Fill,
}

/// O teclado desenhado: UM pintor para B (acesas = as da acção aberta) e C (acesas = as premidas no Try it).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_keys(
    doc: &mut InputMapDoc,
    draw: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
    style: Lit,
    lit_of: &dyn Fn(ph2d_input::Key) -> bool,
) {
    let inner = Rect::new(
        draw.x + m::CARD_PAD_X,
        draw.y + m::CARD_PAD_X,
        draw.w - m::CARD_PAD_X * 2.0,
        draw.h - m::CARD_PAD_X * 2.0,
    );
    // O passo sai da caixa recebida (largura e altura): a B e a C pedem alturas diferentes ao mesmo pintor.
    let p = (inner.w / g::WIDTH).min(pitch_for_rows(inner.h));
    let (kh, step) = (key_height(p), key_height(p) + p * (g::ROW_PITCH - g::KEY_H));
    let ox = inner.x + (inner.w - g::WIDTH * p).max(0.0) * 0.5;
    for KeyCap {
        key,
        col,
        row,
        span,
    } in g::keyboard()
    {
        let kr = Rect::new(
            ox + col * p,
            inner.y + row * step,
            (span - (1.0 - g::KEY_FILL)) * p,
            kh,
        );
        let b = Binding::Key(key);
        let who = users(doc, b);
        let lit = lit_of(key);
        let hover = doc.kb.hover == Some(KbHit::Key(key));
        let filled = (lit && style == Lit::Fill).then(|| {
            who.first()
                .map_or(ColorToken::Accent, |a| crate::palette::of_action(*a))
        });
        // Sem acção a tecla apaga-se (face e letra); com acção a face clareia e a letra acende.
        let bg = match filled {
            Some(c) => c,
            None if hover => ColorToken::AccentSoft,
            None => ColorToken::KanbanCardRaised,
        };
        fill_rounded_rect(scene, kr, m::SEG_RADIUS, ink.color(bg));
        if let Some(c) = filled {
            doc.census.filled.push((kr, c));
        }
        if lit && style == Lit::Outline {
            stroke_rounded_rect(
                scene,
                kr,
                m::SEG_RADIUS,
                m::FIELD_BORDER * 2.0,
                ink.color(ColorToken::Accent),
            );
        }
        if hover {
            hover_ring(scene, ink, kr, lit && style == Lit::Outline);
        }
        let fg = ink.color(if filled.is_some() {
            ColorToken::Bg0
        } else if who.is_empty() {
            ColorToken::Text3
        } else {
            ColorToken::Text1
        });
        let label_h = kr.h - m::SEG_INSET * 0.5 - (BAR_H + BAR_GAP) * who.len().max(1) as f32;
        if let Some(icon) = g::arrow_icon(key) {
            let s = ARROW_BOX.min(label_h);
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

/// O contorno branco do hover, distinto do da acção aberta (POR FORA dele quando os dois se juntam, maquete B1).
pub(super) fn hover_ring(scene: &mut VectorScene, ink: CardInk, r: Rect, outlined: bool) {
    let d = if outlined { -m::FIELD_BORDER * 3.0 } else { 0.0 };
    let rr = Rect::new(
        r.x + d,
        r.y + d,
        (r.w - d * 2.0).max(0.0),
        (r.h - d * 2.0).max(0.0),
    );
    stroke_rounded_rect(
        scene,
        rr,
        m::SEG_RADIUS,
        m::FIELD_BORDER,
        ink.color(ColorToken::Text1),
    );
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
