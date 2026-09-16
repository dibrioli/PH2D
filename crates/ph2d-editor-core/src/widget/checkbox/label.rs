//! ⭐⭐ **O NOME de uma linha de marcar** — onde ele é pousado, e o pintor que o compõe com a marca.
//!
//! ⚠️ **Irmão por RESPONSABILIDADE do [`super::mark`], forçado pelo tecto de 500 LOC do widget**
//! (2026-09-15, quando a marca ganhou CAIXA por ordem do dono): aquele ficheiro responde *onde a
//! marca assenta e que tinta ela leva*; este responde *onde o texto do nome é pousado*. ⛔ Os dois
//! crescem por motivos diferentes — o primeiro por decisões de disposição, o segundo por defeitos
//! de medição de texto. **É a mesma partição que o `property_box` já tem** (`row.rs` / `label.rs`).
//!
//! ⛔ A re-exportação fica no [`super`], então nenhum chamador muda de caminho.

use super::mark::{BooleanMark, paint_boolean_mark};
use super::{Checkbox, CheckboxState};
use crate::paint::{paint_text, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Square box + label. Box fills with `Accent` when Checked, paints a check glyph; Indeterminate
/// paints a dash glyph.
///
/// ⚠️ **A marca vai à DIREITA e o rótulo à esquerda** desde o redesenho de 2026-09 — ver
/// [`paint_boolean_mark`], que é onde a geometria vive.
pub fn paint_checkbox(
    cb: &Checkbox,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let box_rect = paint_boolean_mark(
        rect,
        BooleanMark {
            value: cb.value,
            state: cb.state,
            hover_t: cb.hover_t,
            box_px: cb.box_px,
            decorator: cb.decorator,
            linha: cb.seccao,
        },
        scene,
        theme,
    );

    if !cb.label.is_empty() {
        let label_color = if cb.state == CheckboxState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        // ⚠️ **`Sm`, e não `Base`** — medido em 2026-09-03: a linha de propriedade escreve o
        // rótulo a `12 px` e esta escrevia a `13`. Num formulário as duas alternam, e **1 px de
        // corpo de letra entre linhas vizinhas lê-se como desalinho**, não como ênfase.
        // ⚠️ **No clássico volta a `Base`, à DIREITA da caixa** — é a linha de sempre.
        let redesign = crate::paint::ui_is_redesign();
        let font_size = if redesign {
            TypeToken::Sm.px()
        } else {
            TypeToken::Base.px()
        };
        let ly = rect.y + (rect.h - font_size) * 0.5;
        if let (true, Some(sec)) = (redesign, cb.seccao) {
            // ⭐⭐⭐ **O NOME VAI À COLUNA DO NOME DA SECÇÃO** — report do dono, 2026-09-14
            // (*«as labels alinhadas todas à direita»*) levado até à linha de MARCAR.
            //
            // ⛔⛔ Até 2026-09-15 ele era encostado à esquerda da faixa e corria até à marca. Num
            // formulário que alterna números e marcas isso dá **duas colunas de nome**, alternando
            // linha sim linha não — *a mesma doença do rótulo por cima do campo, meia volta
            // adiante*.
            //
            // ⚠️ **A marca NÃO se move**, e isso é uma lei com gate: ela partilha a
            // [`super::super::property_box::value_column`] com o número, e é isso que dá ao
            // formulário **uma** margem direita (`the_mark_is_anchored_to_the_right_edge`). ⇒ entre
            // o nome e a marca fica um vão, e ele é o mesmo em todas as linhas de marcar.
            let row =
                crate::widget::property_box::colunas_da_linha(rect.x, rect.w, rect.y, rect.h, sec);
            crate::widget::property_box::paint_property_label(
                text_system,
                scene,
                &cb.label,
                row.label.x,
                ly,
                font_size,
                row.label.w,
                resolve(label_color, theme),
            );
        } else if redesign {
            // ⛔ **Fora do formulário** (a pele de canvas): o nome fica onde o artista o pôs.
            let lx = rect.x + Spacing::Md.px();
            let budget = (box_rect.x - lx - Spacing::Md.px()).max(0.0);
            let cut =
                crate::widget::property_box::fit_label(text_system, &cb.label, font_size, budget);
            if !cut.is_empty() {
                paint_text(
                    text_system,
                    scene,
                    &cut,
                    lx,
                    ly,
                    font_size,
                    f32::INFINITY,
                    resolve(label_color, theme),
                );
            }
        } else if rect.w > box_rect.w + Spacing::Md.px() {
            let lx = rect.x + box_rect.w + Spacing::Md.px();
            paint_text(
                text_system,
                scene,
                &cb.label,
                lx,
                ly,
                font_size,
                (rect.x + rect.w - lx).max(0.0),
                resolve(label_color, theme),
            );
        }
    }
}
