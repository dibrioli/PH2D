//! O teclado de notebook medido: o passo para a caixa, a legenda que cabe em cada tecla (`cap` → `cap_short`, as
//! setas em ícone) e o arranjo — `Wide` se toda tecla tem legenda que cabe, senão `Narrow` (plano §13.2). Funções
//! puras da caixa e do texto: a altura pedida e a pintura chegam ao MESMO arranjo e ao MESMO passo.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_input::Key;
use ph2d_text::TextSystem;

use super::geometry::{self as g, Arrangement, KeyCap};
use super::keys::{BAR_GAP, BAR_H};

/// A tecla nunca é mais baixa que a letra com duas barras por baixo (a 1280 as barras tapavam a letra).
pub(super) const KEY_MIN_H: f32 = m::VALUE_PX + m::SEG_PAD + (BAR_H + BAR_GAP) * 2.0 + m::SEG_INSET * 0.5;

/// A margem mínima de cada lado da legenda, dentro da tecla.
pub(crate) const LEGEND_PAD: f32 = m::SEG_INSET * 0.5;

/// O menor tamanho de legenda (o `LABEL_PX`): nada menor (a porta dos testes).
pub const MIN_LEGEND_PX: f32 = m::LABEL_PX;

/// O topo da pilha de `n` barras dentro da tecla `r` (a legenda fica por cima dele).
#[must_use]
pub fn bars_top(r: ph2d_editor_core::zones::Rect, n: usize) -> f32 {
    r.y + r.h - m::SEG_INSET * 0.5 - (BAR_H + BAR_GAP) * n as f32
}

/// A altura de uma tecla de uma fila para o passo `p`.
pub(super) fn key_height(p: f32) -> f32 {
    (p * g::KEY_H).max(KEY_MIN_H)
}

/// O passo vertical entre filas para o passo `p` (a tecla e o vão).
pub(super) fn row_step(p: f32) -> f32 {
    key_height(p) + p * (g::ROW_PITCH - g::KEY_H)
}

/// A altura de `rows` filas para o passo `p`.
pub(super) fn rows_h(p: f32, rows: f32) -> f32 {
    row_step(p) * rows
}

/// O passo cujas `rows` filas medem `h` (o inverso de [`rows_h`]).
fn pitch_for_rows(h: f32, rows: f32) -> f32 {
    let linear = h / (g::ROW_PITCH * rows);
    if linear * g::KEY_H >= KEY_MIN_H {
        linear
    } else {
        (h / rows - KEY_MIN_H).max(0.0) / (g::ROW_PITCH - g::KEY_H)
    }
}

/// A largura pintada de uma tecla de `span` unidades.
pub(super) fn key_w(span: f32, p: f32) -> f32 {
    (span - (1.0 - g::KEY_FILL)) * p
}

/// A altura pintada de uma tecla de `rows` filas.
pub(super) fn key_h(rows: f32, p: f32) -> f32 {
    key_height(p) + row_step(p) * (rows - 1.0)
}

/// A legenda de uma tecla: texto (a `cap` ou a `cap_short`, ao seu tamanho) ou ícone.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Legend {
    Text(String, f32),
    Icon(IconId),
}

/// O tamanho de uma legenda: glifo único a `VALUE_PX`, palavra a `LABEL_PX` (o menor texto do cartão).
pub(crate) fn legend_px(s: &str) -> f32 {
    if s.chars().count() == 1 {
        m::VALUE_PX
    } else {
        m::LABEL_PX
    }
}

/// As legendas por ordem de preferência: a da foto, a curta (o rótulo se o código não tem tecla conhecida).
fn candidates(key: Key) -> Vec<String> {
    let cap = key.cap();
    if cap.is_empty() {
        return vec![key.label()];
    }
    let mut v = vec![cap.to_owned()];
    if key.cap_short() != cap {
        v.push(key.cap_short().to_owned());
    }
    v
}

/// O símbolo ISO/IEC 9995-7 de uma tecla, o último degrau da legenda (a fonte não tem os glifos ⇱ ⇲ ⇞ ⇟ ↵ ⎙).
pub(crate) fn iso_icon(key: Key) -> Option<IconId> {
    Some(match key.0 {
        0xF729 => IconId::KbdHome,
        0xF72B => IconId::KbdEnd,
        0xF72C => IconId::KbdPageUp,
        0xF72D => IconId::KbdPageDown,
        0xF72E => IconId::KbdPrintScreen,
        _ if key.is_enter_like() => IconId::KbdEnter,
        _ => return None,
    })
}

/// A legenda que cabe na largura `w`, ou `None` (nenhuma cabe sem elidir).
pub(crate) fn legend(ts: &mut TextSystem, key: Key, w: f32) -> Option<Legend> {
    if let Some(icon) = g::arrow_icon(key) {
        return Some(Legend::Icon(icon));
    }
    candidates(key).into_iter().find_map(|s| {
        let px = legend_px(&s);
        (ts.prefix_width(&s, px) + LEGEND_PAD * 2.0 <= w).then_some(Legend::Text(s, px))
    })
    .or_else(|| {
        iso_icon(key)
            .filter(|_| m::VALUE_PX + LEGEND_PAD * 2.0 <= w)
            .map(Legend::Icon)
    })
}

/// A legenda a pintar quando nenhuma cabe: a mais curta (o censo regista-a como cortada — nunca se esconde).
pub(crate) fn shortest(key: Key) -> Legend {
    let s = candidates(key).pop().unwrap_or_default();
    let px = legend_px(&s);
    Legend::Text(s, px)
}

/// O arranjo escolhido e o passo, para uma caixa de largura `w` (e altura `h`, se a caixa já foi dada).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub arrangement: Arrangement,
    pub pitch: f32,
    /// Toda tecla tem uma legenda que cabe.
    pub legible: bool,
}

fn pitch_of(a: Arrangement, w: f32, h: Option<f32>, cap: f32) -> f32 {
    let (cols, rows) = g::extent(a);
    let p = (w / cols).min(m::FIELD_H * cap);
    h.map_or(p, |h| p.min(pitch_for_rows(h, rows)))
}

fn legible(ts: &mut TextSystem, a: Arrangement, p: f32) -> bool {
    g::keyboard(a)
        .iter()
        .all(|c| legend(ts, c.key, key_w(c.span, p)).is_some())
}

/// ⭐ A escolha (plano §13.2): `Wide` se toda tecla tem legenda que cabe; senão `Narrow`.
pub(crate) fn fit(ts: &mut TextSystem, w: f32, h: Option<f32>, cap: f32) -> Fit {
    let wide = pitch_of(Arrangement::Wide, w, h, cap);
    if legible(ts, Arrangement::Wide, wide) {
        return Fit {
            arrangement: Arrangement::Wide,
            pitch: wide,
            legible: true,
        };
    }
    let p = pitch_of(Arrangement::Narrow, w, h, cap);
    Fit {
        arrangement: Arrangement::Narrow,
        pitch: p,
        legible: legible(ts, Arrangement::Narrow, p),
    }
}

/// A altura do desenho (sem o enchimento) para a escolha `f`.
pub(super) fn drawing_rows_h(f: Fit) -> f32 {
    rows_h(f.pitch, g::extent(f.arrangement).1)
}

/// A escolha para a largura `w` (pública para os testes e o relatório de medida).
#[must_use]
pub fn fit_for_width(ts: &mut TextSystem, w: f32, cap_b: bool) -> Fit {
    let cap = if cap_b {
        g::PITCH_MAX_ROWS_B
    } else {
        g::PITCH_MAX_ROWS
    };
    fit(ts, w, None, cap)
}

/// As teclas com a sua caixa relativa à origem do desenho: `(tecla, x, y, w, h)`.
pub(super) fn placed(f: Fit) -> Vec<(KeyCap, f32, f32, f32, f32)> {
    let p = f.pitch;
    g::keyboard(f.arrangement)
        .into_iter()
        .map(|c| {
            let (x, y) = (c.col * p, c.row * row_step(p));
            (c, x, y, key_w(c.span, p), key_h(c.rows, p))
        })
        .collect()
}
