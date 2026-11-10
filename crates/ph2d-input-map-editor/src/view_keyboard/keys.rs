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

use super::geometry::{self as g, Arrangement, KeyCap};
use super::keys_fit::{self as fit, HOVER_FACE, Legend, Level};
use super::state::{KbHit, Sub};
use super::users;
use crate::census::KeyLook;
use crate::doc::InputMapDoc;
use crate::interact::{Control, ControlRole};
use crate::view_list::footer::{binding_label, reader_names, readers_of};
use crate::view_list::text::{line, title};

const CAPTION_H: f32 = m::FIELD_H - m::SEG_PAD;
/// A barra de uma acção numa tecla, e o vão entre barras.
pub(super) const BAR_H: f32 = m::SEG_PAD + m::FIELD_BORDER;
pub(super) const BAR_GAP: f32 = m::FIELD_BORDER;
/// A caixa da seta desenhada (teclas, ladrilhos, comando): da altura da letra, para ler como ↑ e não como ponto.
pub(super) const ARROW_BOX: f32 = m::VALUE_PX + m::SEG_PAD * 2.0;

/// O tecto do passo da C (o Try it).
pub(crate) const PITCH_CAP_C: f32 = g::PITCH_MAX_ROWS;

/// A altura do desenho do teclado para a largura `w` (a vista C reusa-o).
pub(crate) fn keyboard_h(ts: &mut TextSystem, w: f32) -> f32 {
    let inner = w - m::CARD_PAD_X * 2.0;
    fit::drawing_rows_h(fit::fit(ts, inner, None, g::PITCH_MAX_ROWS)) + m::CARD_PAD_X * 2.0
}

/// O teclado da B enche a largura do cartão (maquete B1): o tecto do passo é mais alto que o da C.
fn keyboard_h_b(ts: &mut TextSystem, w: f32) -> f32 {
    let inner = w - m::CARD_PAD_X * 2.0;
    fit::drawing_rows_h(fit::fit(ts, inner, None, g::PITCH_MAX_ROWS_B)) + m::CARD_PAD_X * 2.0
}

fn drawing_h(doc: &InputMapDoc, ts: &mut TextSystem, w: f32) -> f32 {
    match doc.kb.sub {
        Sub::Keyboard => keyboard_h_b(ts, w),
        Sub::Gamepad => super::pad::height(w, doc.kb.stacked),
    }
}

/// O que está ligado e o desenho não mostra (dígitos, Shift, Enter…, ou Mode e os cliques de manche).
pub(super) fn also(doc: &InputMapDoc) -> Vec<Binding> {
    let drawn: Vec<Binding> = match doc.kb.sub {
        Sub::Keyboard => g::drawn_keys().into_iter().map(Binding::Key).collect(),
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

pub(super) fn top_height(doc: &InputMapDoc, ts: &mut TextSystem, w: f32) -> f32 {
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
        + drawing_h(doc, ts, inner)
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
    for (s, k) in [
        (Sub::Keyboard, "input_map_keyboard.sub.keyboard"),
        (Sub::Gamepad, "input_map_keyboard.sub.gamepad"),
    ] {
        let id = KbHit::Sub(s).id();
        if let Some(sr) = doc.hits.rect_for(id) {
            doc.control(Control::new(id, sr, ControlRole::Segment, tr(k)));
        }
    }
    let dh = drawing_h(doc, ts, w);
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
    if let Binding::Key(k) = b
        && let Some(why) = k.system_owned()
    {
        return Some((
            format!("{label}{}", tr("input_map_keyboard.colon")),
            tr(why).to_owned(),
        ));
    }
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

/// Uma barra por acção que usa `b`, em baixo de `r`, na cor da acção (o censo regista cada uma).
pub(super) fn paint_bars(
    census: &mut crate::census::Census,
    scene: &mut VectorScene,
    ink: CardInk,
    r: Rect,
    who: &[ActionId],
) {
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
        census.bars.push((bar, *id));
        by -= BAR_H + BAR_GAP;
    }
}

/// A face de um alvo desenhado (tecla ou botão do comando), por precedência: premido > aceso (a cor da acção, no
/// Try it) > hover (a face clara do campo) > repouso. O anel do foco (o partilhado, `paint_focus_ring`) e o
/// contorno da acção aberta não são faces: juntam-se a qualquer uma, o contorno recuado para dentro do anel.
pub(super) fn face_of(pressed: bool, lit_fill: Option<ColorToken>, hover: bool) -> ColorToken {
    if pressed {
        ColorToken::AccentPress
    } else if let Some(c) = lit_fill {
        c
    } else if hover {
        HOVER_FACE
    } else {
        ColorToken::KanbanCardRaised
    }
}

/// A opacidade da face da tecla do sistema (a de repouso sobre o fundo do desenho): fica entre o fundo e o repouso
/// em qualquer tema — menos ênfase que uma tecla em repouso, sem aresta nenhuma.
pub(crate) const SYSTEM_FACE_ALPHA: f32 = ph2d_tokens::LAYER_OUTLINE_OFF_ALPHA;

/// Pinta a face `tok` (o hover = o repouso + o véu).
pub(super) fn fill_face(
    scene: &mut VectorScene,
    ink: CardInk,
    r: Rect,
    radius: f32,
    tok: ColorToken,
) {
    if tok == HOVER_FACE {
        fill_rounded_rect(scene, r, radius, ink.color(ColorToken::KanbanCardRaised));
    }
    fill_rounded_rect(scene, r, radius, ink.color(tok));
}

/// A faixa por baixo das `n` barras de `r`, no fundo do desenho (`CardField`): as barras mantêm o contraste sobre
/// uma face cheia (premida ou acesa na cor de uma acção).
fn paint_bar_strip(scene: &mut VectorScene, ink: CardInk, r: Rect, n: usize) -> Rect {
    let top = fit::bars_top(r, n) - BAR_GAP;
    let inset = m::SEG_INSET * 0.25;
    let s = Rect::new(
        r.x + inset,
        top,
        (r.w - inset * 2.0).max(0.0),
        (r.y + r.h - inset - top).max(0.0),
    );
    fill_rounded_rect(scene, s, BAR_H, ink.color(ColorToken::CardField));
    s
}

/// O contorno da acção aberta, recuado de uma aresta para dentro (o anel do foco fica na aresta, os dois lêem-se).
pub(super) fn open_outline(scene: &mut VectorScene, ink: CardInk, r: Rect, radius: f32) {
    let d = m::FIELD_BORDER * 2.0;
    let rr = Rect::new(
        r.x + d,
        r.y + d,
        (r.w - d * 2.0).max(0.0),
        (r.h - d * 2.0).max(0.0),
    );
    stroke_rounded_rect(
        scene,
        rr,
        (radius - d).max(0.0),
        m::FIELD_BORDER * 2.0,
        ink.color(ColorToken::Accent),
    );
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
    let style = (Lit::Outline, KeysAlign::Center, g::PITCH_MAX_ROWS_B);
    paint_keys(doc, draw, scene, ts, ink, style, &|k| {
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

/// Onde o teclado assenta na caixa quando ela é mais larga que ele: a B ao centro; o Try it com as letras encostadas
/// à esquerda e as setas à DIREITA do cartão (maquete C2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeysAlign {
    Center,
    Spread,
}

/// O teclado desenhado: UM pintor para B (acesas = as da acção aberta) e C (acesas = as premidas no Try it).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_keys(
    doc: &mut InputMapDoc,
    draw: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
    (style, align, cap): (Lit, KeysAlign, f32),
    lit_of: &dyn Fn(ph2d_input::Key) -> bool,
) {
    let inner = Rect::new(
        draw.x + m::CARD_PAD_X,
        draw.y + m::CARD_PAD_X,
        draw.w - m::CARD_PAD_X * 2.0,
        draw.h - m::CARD_PAD_X * 2.0,
    );
    // O arranjo e o passo saem da caixa recebida (largura e altura) pela MESMA escolha que pediu a altura.
    let f = fit::fit(ts, inner.w, Some(inner.h), cap);
    // O degrau da legenda por GRUPO (nunca por tecla): a B e a C, na mesma caixa, dão o mesmo.
    let levels = fit::levels(ts, f);
    let p = f.pitch;
    let (cols, _) = g::extent(f.arrangement);
    let spare = (inner.w - cols * p).max(0.0);
    // O Try it encosta o principal à esquerda e o numérico à DIREITA do cartão (maquete C2: as setas à direita).
    let (ox, pad_dx) = match (align, f.arrangement) {
        (KeysAlign::Center, _) | (KeysAlign::Spread, Arrangement::Narrow) => {
            (inner.x + spare * 0.5, 0.0)
        }
        // A tinta da última coluna acaba no recuo (sem o vão que segue a última tecla).
        (KeysAlign::Spread, Arrangement::Wide) => (inner.x, spare + (1.0 - g::KEY_FILL) * p),
    };
    doc.kb.fit = Some(f);
    for (KeyCap { key, col, .. }, x, y, w, h) in fit::placed(f) {
        let dx = if col >= g::MAIN_W { pad_dx } else { 0.0 };
        let kr = Rect::new(ox + dx + x, inner.y + y, w, h);
        let level = levels
            .iter()
            .find(|(g, _)| *g == fit::group_of(key))
            .map_or(Level::ShortSmall, |l| l.1);
        paint_key(doc, scene, ts, ink, (style, kr), (key, lit_of(key), level));
    }
}

/// Uma tecla: a face pela precedência de [`face_of`], o contorno da aberta, a legenda no degrau do grupo e as barras.
fn paint_key(
    doc: &mut InputMapDoc,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
    (style, kr): (Lit, Rect),
    (key, lit, level): (ph2d_input::Key, bool, Level),
) {
    let system = key.system_owned().is_some();
    let b = Binding::Key(key);
    let id = KbHit::Key(key).id();
    let who = if system { Vec::new() } else { users(doc, b) };
    let hover = doc.kb.hover == Some(KbHit::Key(key));
    let filled = (lit && !system && style == Lit::Fill).then(|| {
        who.first()
            .map_or(ColorToken::Accent, |a| crate::palette::of_action(*a))
    });
    // O registo de controlos (plano §13.3) ANTES de ler o premido (que pede o rect deste quadro); a do sistema
    // desligada com o porquê.
    doc.control(Control::new(id, kr, ControlRole::Key, key.label()).disabled(key.system_owned()));
    let pressed = !system && doc.is_pressed(id);
    // A do sistema: desenhada, nunca ligável (plano §13.2) — a face de repouso apagada, sem aresta nem estados.
    let bg = if system {
        ColorToken::KanbanCardRaised
    } else {
        face_of(pressed, filled, hover)
    };
    if system {
        let c = ink.color(bg).multiply_alpha(SYSTEM_FACE_ALPHA);
        fill_rounded_rect(scene, kr, m::SEG_RADIUS, c);
    } else {
        fill_face(scene, ink, kr, m::SEG_RADIUS, bg);
    }
    if let Some(c) = filled.filter(|_| !pressed) {
        doc.census.filled.push((kr, c));
    }
    let strip = (!who.is_empty() && (pressed || filled.is_some()))
        .then(|| paint_bar_strip(scene, ink, kr, who.len()));
    let outlined = lit && !system && style == Lit::Outline;
    let edge = if outlined {
        open_outline(scene, ink, kr, m::SEG_RADIUS);
        Some(ColorToken::Accent)
    } else {
        None
    };
    let fg_tok = if system {
        ColorToken::TextDisabled
    } else if pressed || filled.is_some() {
        ColorToken::Bg0
    } else if who.is_empty() {
        ColorToken::Text3
    } else {
        ColorToken::Text1
    };
    let fg = ink.color(fg_tok);
    let label_h = fit::label_h(kr.h, fit::reserved_bars(key, who.len()));
    let legend = fit::choose(ts, key, (level, true), kr.w, label_h)
        .unwrap_or_else(|| fit::legend_at(key, level));
    let text = match &legend {
        Legend::Icon(icon) => {
            // O ícone fica da altura da letra no mínimo e nunca encosta à aresta da tecla.
            let s = ARROW_BOX
                .min(label_h)
                .min(kr.w - fit::LEGEND_PAD * 2.0)
                .max(0.0);
            let ir = Rect::new(kr.x + (kr.w - s) * 0.5, kr.y + (label_h - s) * 0.5, s, s);
            // O símbolo é traço fino: nunca mais apagado que `Text2` numa tecla do jogo (a ligada fica em `Text1`).
            let tone = if fg_tok == ColorToken::Text3 {
                ink.color(ColorToken::Text2)
            } else {
                fg
            };
            paint_icon(scene, *icon, ir, tone, StrokeToken::Default.px());
            doc.census.icons.push((ir, *icon));
            None
        }
        Legend::Text(label, px) => {
            let tw = ts.prefix_width(label, *px).min(kr.w);
            let lr = Rect::new(kr.x + (kr.w - tw) * 0.5, kr.y, tw, label_h);
            line(&mut doc.census, ts, scene, label, lr, *px, fg);
            Some((label.clone(), *px))
        }
        Legend::Two(a, b2, px) => {
            let lh = fit::two_lines_h(*px) * 0.5;
            let top = kr.y + label_h * 0.5 - lh;
            for (i, s) in [a, b2].into_iter().enumerate() {
                let tw = ts.prefix_width(s, *px).min(kr.w);
                let lr = Rect::new(kr.x + (kr.w - tw) * 0.5, top + lh * i as f32, tw, lh);
                line(&mut doc.census, ts, scene, s, lr, *px, fg);
            }
            Some((format!("{a}{b2}"), *px))
        }
    };
    doc.census.keys.push(KeyLook {
        key,
        rect: kr,
        face: bg,
        ink: fg_tok,
        edge,
        dimmed: system,
        strip,
        text,
    });
    paint_bars(&mut doc.census, scene, ink, kr, &who);
    doc.kb.bars.push((KbHit::Key(key), who));
    doc.kb.targets.push(KbHit::Key(key));
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
        paint_bars(&mut doc.census, scene, ink, chip, &who);
        let hit = match b {
            Binding::Key(k) => KbHit::Key(*k),
            other => KbHit::Pad(*other),
        };
        doc.control(Control::new(hit.id(), chip, ControlRole::Chip, label));
        doc.kb.targets.push(hit);
        x += chip.w + m::FIELD_GAP;
    }
}
