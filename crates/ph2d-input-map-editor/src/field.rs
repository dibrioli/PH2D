//! **O campo numérico da aba** (partilhado: a vista A agora, a B a seguir; plano §13.3): o foco abre o texto do
//! valor TODO seleccionado; escrever substitui a selecção; ↑/↓ um passo (Shift ×10, Alt ×0,1 — a lei do
//! `card_step_mul` do cartão, pelas MESMAS constantes `SCRUB_SHIFT_MUL`/`SCRUB_ALT_MUL`); Enter aplica, Esc
//! devolve o valor do foco, Tab aplica e segue. Fora de `0..=1` o valor fica no limite e o [`Clamp`] diz porquê.
//!
//! O texto mora com quem o usa (`Typing::buffer` na A); aqui moram o caret, a âncora e o valor do foco.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::doc_surface::{Key as DocKey, KeyInput};
use ph2d_editor_core::widget::card_field::{paint_field_caret, paint_field_frame};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;

/// Um passo de ↑/↓ sobre um número de `0..=1`.
pub const STEP: f32 = 0.01; // LITERAL-PX-OK: zone step

/// O valor escrito saiu de `0..=1` e ficou no limite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clamp {
    Max,
    Min,
}

impl Clamp {
    /// A frase (Warn) ao lado do campo.
    #[must_use]
    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Max => "input_map_list.kept_max",
            Self::Min => "input_map_list.kept_min",
        }
    }
}

/// `v` dentro de `0..=1`, e se foi preciso prender.
#[must_use]
pub fn clamp01(v: f32) -> (f32, Option<Clamp>) {
    if v > 1.0 {
        (1.0, Some(Clamp::Max))
    } else if v < 0.0 {
        (0.0, Some(Clamp::Min))
    } else {
        (v, None)
    }
}

/// O texto de um valor: duas casas; três só quando o passo fino (Alt) as pôs lá.
#[must_use]
pub fn format(v: f32) -> String {
    let two = (v * 100.0).round() / 100.0;
    if (two - v).abs() < 1e-5 {
        format!("{v:.2}")
    } else {
        format!("{v:.3}")
    }
}

/// O caret, a âncora da selecção e o valor no momento do foco.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Caret {
    pub caret: usize,
    pub anchor: Option<usize>,
    pub at_focus: f32,
}

impl Caret {
    /// O foco num campo de valor `v`: o texto formatado, TODO seleccionado.
    #[must_use]
    pub fn open(v: f32) -> (String, Self) {
        let buffer = format(v);
        let c = Self {
            caret: buffer.len(),
            anchor: Some(0),
            at_focus: v,
        };
        (buffer, c)
    }

    /// A selecção `(início, fim)`, se não vazia.
    #[must_use]
    pub fn selection(&self) -> Option<(usize, usize)> {
        let a = self.anchor?;
        (a != self.caret).then(|| (a.min(self.caret), a.max(self.caret)))
    }

    fn select_all(&mut self, len: usize) {
        self.anchor = Some(0);
        self.caret = len;
    }

    fn take_selection(&mut self, buffer: &mut String) -> bool {
        let Some((s, e)) = self.selection() else {
            self.anchor = None;
            return false;
        };
        buffer.replace_range(s..e, "");
        self.caret = s;
        self.anchor = None;
        true
    }
}

/// O que uma tecla fez ao campo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    /// Não era para o campo.
    Ignored,
    /// O texto ou o caret mudaram.
    Edited,
    /// ↑/↓: o texto passou a este valor (ainda não aplicado).
    Stepped(Option<Clamp>),
    /// Enter: aplica `v`.
    Commit(f32, Option<Clamp>),
    /// Tab (`back` = Shift+Tab): aplica `v` e o foco segue.
    Move {
        v: f32,
        clamp: Option<Clamp>,
        back: bool,
    },
    /// Esc: o valor do foco volta e o campo larga.
    Revert(f32),
}

/// O multiplicador do passo pelos modificadores (o do `card_step_mul` do cartão).
#[must_use]
pub fn step_mul(mods: ph2d_host::Modifiers) -> f32 {
    if mods.shift {
        m::SCRUB_SHIFT_MUL as f32
    } else if mods.alt {
        m::SCRUB_ALT_MUL as f32
    } else {
        1.0
    }
}

/// O valor do texto, preso a `0..=1`; texto que não é número = o valor do foco.
#[must_use]
pub fn value_of(buffer: &str, c: &Caret) -> (f32, Option<Clamp>) {
    clamp01(buffer.trim().parse::<f32>().unwrap_or(c.at_focus))
}

/// Uma tecla para o campo aberto.
pub fn key(buffer: &mut String, c: &mut Caret, input: &KeyInput<'_>) -> Outcome {
    if !input.pressed {
        return Outcome::Ignored;
    }
    let len = buffer.len();
    match input.key {
        DocKey::Enter => {
            let (v, clamp) = value_of(buffer, c);
            Outcome::Commit(v, clamp)
        }
        DocKey::Tab => {
            let (v, clamp) = value_of(buffer, c);
            Outcome::Move {
                v,
                clamp,
                back: input.mods.shift,
            }
        }
        DocKey::Escape => Outcome::Revert(c.at_focus),
        DocKey::Up | DocKey::Down => {
            let sign = if input.key == DocKey::Up { 1.0 } else { -1.0 };
            let (now, _) = value_of(buffer, c);
            let (v, clamp) = clamp01(now + sign * STEP * step_mul(input.mods));
            *buffer = format(v);
            c.select_all(buffer.len());
            Outcome::Stepped(clamp)
        }
        DocKey::Char('a' | 'A') if input.mods.ctrl => {
            c.select_all(len);
            Outcome::Edited
        }
        DocKey::Backspace => {
            if !c.take_selection(buffer) && c.caret > 0 {
                c.caret -= 1;
                buffer.remove(c.caret);
            }
            Outcome::Edited
        }
        DocKey::Delete => {
            if !c.take_selection(buffer) && c.caret < len {
                buffer.remove(c.caret);
            }
            Outcome::Edited
        }
        DocKey::Left | DocKey::Right | DocKey::Home | DocKey::End => {
            c.caret = match (input.key, c.selection()) {
                (DocKey::Home, _) => 0,
                (DocKey::End, _) => len,
                (DocKey::Left, Some((s, _))) => s,
                (DocKey::Right, Some((_, e))) => e,
                (DocKey::Left, None) => c.caret.saturating_sub(1),
                _ => (c.caret + 1).min(len),
            };
            c.anchor = None;
            Outcome::Edited
        }
        _ => {
            let typed: String = (input.text.unwrap_or(""))
                .chars()
                .filter(|ch| ch.is_ascii_digit() || *ch == '.' || *ch == '-')
                .collect();
            if typed.is_empty() {
                return Outcome::Ignored;
            }
            c.take_selection(buffer);
            buffer.insert_str(c.caret, &typed);
            c.caret += typed.len();
            Outcome::Edited
        }
    }
}

/// A caixa do campo pelos pintores do cartão: o fundo (hover/foco) e o anel do foco.
pub fn paint_frame(scene: &mut VectorScene, ink: CardInk, r: Rect, hover: bool, focused: bool) {
    paint_field_frame(scene, ink, r, hover, focused);
}

/// A selecção (o `AccentSoft` do cartão) e o caret piscante sobre o texto pintado em `text_x`, centrado em `r`.
#[allow(clippy::too_many_arguments)]
pub fn paint_edit(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
    r: Rect,
    text_x: f32,
    text: &str,
    caret: usize,
    anchor: Option<usize>,
    caret_on: bool,
) {
    let px = m::VALUE_PX;
    let y = r.y + ph2d_editor_core::paint::folga_do_texto(r.h, px);
    let at =
        |ts: &mut TextSystem, i: usize| text_x + ts.prefix_width(&text[..i.min(text.len())], px);
    if let Some(a) = anchor.filter(|a| *a != caret) {
        let (s, e) = (a.min(caret), a.max(caret));
        let (x0, x1) = (at(ts, s), at(ts, e));
        let sel = Rect::new(x0, y, (x1 - x0).max(0.0), px);
        ph2d_editor_core::paint::fill_rounded_rect(
            scene,
            sel,
            m::SEG_PAD,
            ink.color(ColorToken::AccentSoft),
        );
    }
    if caret_on {
        let x = at(ts, caret);
        paint_field_caret(scene, ink, x, y, px);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(k: DocKey, text: Option<&str>, mods: ph2d_host::Modifiers) -> KeyInput<'_> {
        KeyInput {
            key: k,
            mods,
            pressed: true,
            text,
            paste: None,
        }
    }

    #[test]
    fn foco_seleciona_tudo_e_escrever_substitui() {
        let (mut b, mut c) = Caret::open(0.2);
        assert_eq!((b.as_str(), c.selection()), ("0.20", Some((0, 4))));
        let none = ph2d_host::Modifiers::default();
        key(&mut b, &mut c, &press(DocKey::Char('7'), Some("7"), none));
        assert_eq!(b, "7");
        key(&mut b, &mut c, &press(DocKey::Home, None, none));
        key(&mut b, &mut c, &press(DocKey::Char('.'), Some("."), none));
        assert_eq!(b, ".7");
        assert_eq!(value_of(&b, &c), (0.7, None));
    }

    #[test]
    fn passos_e_limites() {
        let (mut b, mut c) = Caret::open(0.995);
        let shift = ph2d_host::Modifiers {
            shift: true,
            ..Default::default()
        };
        let out = key(&mut b, &mut c, &press(DocKey::Up, None, shift));
        assert_eq!(
            (b.as_str(), out),
            ("1.00", Outcome::Stepped(Some(Clamp::Max)))
        );
        b = "-3".into();
        assert_eq!(value_of(&b, &c), (0.0, Some(Clamp::Min)));
    }
}
