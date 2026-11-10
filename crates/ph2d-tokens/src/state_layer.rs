//! **O que um estado pinta sobre a superfície de um controlo**, nos dois temas (estado da arte):
//! o disco do interruptor ligado (macOS/iOS) e a camada do hover (Material 3, «state layers»).
//! Porquê: `docs/CodeEditor/handoffs/HANDOFF_line_ScriptEditor_2026-11-10_ACABAMENTO.md` §A.

use crate::color::{Color, ColorToken};
use crate::color_space::srgb_to_oklch;
use crate::theme::Theme;

/// A opacidade da camada de HOVER: a tinta do texto da superfície, a 8 % por cima dela (o
/// `state-layer` de hover do Material 3). Num tema escuro a linha CLAREIA; num claro escurece (a
/// superfície de um tema claro já é a mais clara).
pub const HOVER_LAYER_ALPHA: f32 = 0.08;

/// O piso da luminosidade OKLCH do disco: o quase-branco do macOS (≈ `#ebebeb`). Num tema pastel
/// (Candy) a ponta mais clara fica a meio (luminância WCAG `0,465` medida, `L` ≈ 0,77) e o disco
/// leria cinzento sobre o acento.
const KNOB_L: f64 = 0.93;
/// O croma máximo do disco: quase neutro, um nada do matiz do tema.
const KNOB_C: f64 = 0.02;

/// O disco de um interruptor LIGADO: claro nos dois temas (macOS/iOS: o disco é claro sobre o
/// acento, em claro e em escuro). A mais clara das duas pontas do tema (o texto num escuro, a
/// superfície elevada num claro), levada ao piso [`KNOB_L`] e quase neutra.
#[must_use]
pub fn switch_knob(theme: Theme) -> Color {
    let text = ColorToken::Text1.resolve(theme);
    let surface = ColorToken::BgElev.resolve(theme);
    let lighter = if text.relative_luminance() >= surface.relative_luminance() {
        text
    } else {
        surface
    };
    let (l, c, h) = srgb_to_oklch(lighter.r, lighter.g, lighter.b);
    Color::from_oklch(l.max(KNOB_L), c.min(KNOB_C), h)
}
