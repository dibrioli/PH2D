//! Os três braços de **EDITOR** do `paint_rows` — curva, paleta e gradiente.
//!
//! Eles são a família distinta do `match`: cada um desenha um editor de VÁRIAS linhas, devolve
//! **a própria altura** (que é função do conteúdo — quantos pontos, quantas cores, quantos stops)
//! e enche uma sacola de widgets que só a fase mutável do painel consegue registrar. Os outros
//! nove braços são uma linha de widget com altura de linha; misturar as duas famílias no mesmo
//! corpo é o que levou o `paint_rows` ao teto de 200 LOC.

use super::gradient_row::ColourRowWidgets;
use super::{ParamRow, curve_row, gradient_row};
use crate::curve_row::CurveWidgets;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Desenha a row se ela for um editor, devolvendo a ALTURA usada; `None` para as demais.
///
/// ⚠️ O `None` é o que mantém UMA porta: o `paint_rows` não enumera quais variantes são editores
/// (uma lista que apodrece quando a quarta chegar) — ele pergunta, e quem responde é o mesmo
/// `match` que desenha.
#[expect(
    clippy::too_many_arguments,
    reason = "espelha a porta de paint do paint_rows"
)]
pub(crate) fn paint_editor_row(
    row: &ParamRow,
    i: usize,
    inner_x: f32,
    inner_w: f32,
    y: f32,
    label_font: f32,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    curve_widgets: &mut CurveWidgets,
    gradient_widgets: &mut ColourRowWidgets,
) -> Option<f32> {
    let used = match row {
        ParamRow::Curve(row) => {
            // The interactive Curve editor — a graph with draggable handles. Its
            // `CurvePoint`/`Button` store states ride back in `curve_widgets` (this
            // pass has only an immutable store); the caller registers them (Phase C).
            curve_row::paint_curve_row(
                row,
                i,
                inner_x,
                inner_w,
                y,
                label_font,
                hit_index,
                scene,
                text_system,
                theme,
                curve_widgets,
            )
        }
        ParamRow::Palette(row) => {
            // The wrapping swatch strip. It returns the height it USED, because a
            // palette's row height is a function of how many colours there are.
            crate::palette_row::paint_palette_row(
                row,
                i,
                inner_x,
                inner_w,
                y,
                label_font,
                hit_index,
                scene,
                text_system,
                theme,
                gradient_widgets,
            )
        }
        ParamRow::Gradient(row) => {
            // The interactive Gradient editor — a bar with draggable position markers +
            // per-stop swatches. Its `CurvePoint`/`Button`/picker-swatch store states ride
            // back in `gradient_widgets` (this pass has only an immutable store); the
            // caller registers them (Phase B/C), the mirror of the Curve editor.
            gradient_row::paint_gradient_row(
                row,
                i,
                inner_x,
                inner_w,
                y,
                label_font,
                hit_index,
                scene,
                text_system,
                theme,
                gradient_widgets,
            )
        }
        _ => return None,
    };
    Some(used)
}
