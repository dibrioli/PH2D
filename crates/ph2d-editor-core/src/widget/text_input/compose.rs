//! A pré-edição do IME no texto MOSTRADO de um campo ([`crate::ime`]) — irmão do pintor pelo
//! tecto de LOC dos widgets — e no VALOR que o leitor de ecrã lê ([`a11y_value`]).

use super::{TextInput, TextInputState};

/// O texto MOSTRADO durante uma composição: o valor com a pré-edição no caret.
pub(super) struct Composed {
    pub(super) text: String,
    /// O caret dentro de `text` (o cursor do IME, ou o fim da pré-edição).
    pub(super) caret: usize,
    /// O trecho sublinhado `[start, end)`.
    pub(super) start: usize,
    pub(super) end: usize,
}

pub(super) fn with_composition(value: &str, caret: usize, c: &crate::ime::Composition) -> Composed {
    let at = Some(caret.min(value.len()))
        .filter(|&a| value.is_char_boundary(a))
        .unwrap_or(value.len());
    let cur = c
        .cursor
        .map(|(_, e)| e)
        .filter(|&e| c.text.is_char_boundary(e))
        .unwrap_or(c.text.len());
    Composed {
        text: [&value[..at], c.text.as_str(), &value[at..]].concat(),
        caret: at + cur,
        start: at,
        end: at + c.text.len(),
    }
}

/// ⭐ **O leitor de ecrã lê o MESMO texto que o ecrã mostra** — o valor com a pré-edição no caret
/// enquanto o campo focado compõe (o `´` pendente ouve-se antes do `a`); fora disso, o valor.
pub(super) fn a11y_value(b: ph2d_a11y::NodeBuilder, input: &TextInput) -> ph2d_a11y::NodeBuilder {
    match (input.state == TextInputState::Focused)
        .then(|| crate::ime::composition_for(input.id))
        .flatten()
    {
        Some(c) => b.value(with_composition(&input.value, input.caret_byte, &c).text),
        None => b.value(input.value.clone()),
    }
}

/// How far the single line is slid LEFT so the caret stays inside the box.
///
/// Pure, so the rule can be stated and tested without a scene. Exactly as far as
/// the caret overhangs, plus the caret's own width — a caret parked ON the right
/// boundary is trimmed by the clip precisely while you are typing at the end of
/// the name, which is the one moment you need to see it. An unfocused field never
/// scrolls: there is no caret to follow, and the reader wants the name's start.
pub(super) fn caret_scroll(focused: bool, inner_w: f32, caret_w: f32) -> f32 {
    if focused {
        (caret_w - (inner_w - super::CARET_W)).max(0.0)
    } else {
        0.0
    }
}
