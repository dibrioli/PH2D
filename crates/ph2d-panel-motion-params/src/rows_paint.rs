//! The per-row paint loop (split out of `lib.rs::paint` for the HR-18 200-LOC fn cap +
//! the 600-LOC panel-file cap). One `match` arm per [`ParamRow`] kind, each delegating to
//! the SHARED source-of-truth painters (slider/chip, swatch, checkbox, segmented button,
//! number box, text field). `super` is the crate root, so the pooled-id helpers and the
//! `normalized_track`/`row_value` mappers are in scope.

use super::curve_row::{self, CurveWidgets};
use super::gradient_row::{self, ColourRowWidgets};
use super::{
    CHANNELS_EXTRA_BASE, MAX_ENUM_OPTIONS, MAX_PARAM_ROWS, ParamRow, normalized_track,
    paint_angle_row, paint_seed_row, paint_text_row, param_checkbox_id, param_chip_id,
    param_enum_id, param_number_id, param_reroll_id, param_slider_id, param_swatch_id,
    param_text_id, row_value,
};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::{
    ButtonState, Checkbox, CheckboxState, CheckboxValue, ColorSwatch, DEFAULT_LABEL_W, SwatchSize,
    paint_checkbox, paint_color_swatch, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[path = "rows_paint_editors.rs"]
mod editors;
#[path = "rows_paint_number.rs"]
mod number;
#[path = "rows_paint_reset.rs"]
mod reset;
#[path = "rows_paint_sections.rs"]
pub(crate) mod sections;
use reset::{RESET_GUTTER_W, paint_reset_button, row_is_modified};
#[path = "rows_paint_kinds.rs"]
mod kinds;
use kinds::{
    paint_channels_row, paint_color_row, paint_driven_row, paint_enum_row, paint_scalar_row,
    paint_source_row, paint_toggle_row,
};

/// Paint each param row from `body_top` down, registering hit rects as it goes.
///
/// Devolve, além dos widgets que a fase mutável registra, **a ALTURA que de fato usou**.
///
/// ⚠️ Ela existe porque o painel **não rola**: um teto de linhas alto o bastante para caber todo
/// param do registry (`MAX_PARAM_ROWS`, medido pelo censo na shell) só é honesto se essas linhas
/// couberem na altura do inspector — senão a linha 14 deixa de ser cortada pelo `.take()` e passa
/// a ser cortada pela borda da tela, que é a MESMA invisibilidade por outra porta. Quem responde
/// *"quanto isto ocupou?"* é o pintor, uma vez; um segundo cálculo de altura ao lado dele
/// divergiria no dia em que uma linha composta mudar de tamanho.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_rows(
    rows: &[ParamRow],
    inner_x: f32,
    inner_w: f32,
    chip_w: f32,
    row_gap: f32,
    body_top: f32,
    label_font: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    modified: &std::collections::BTreeSet<String>,
    section_at: &[(String, usize)],
) -> (CurveWidgets, ColourRowWidgets, f32) {
    let mut y = body_top;
    // Toda row cede a MESMA calha à direita, tenha ou não o que reverter — largura que depende
    // do estado é rótulo que se mexe quando você toca nele (`rows_paint_reset`).
    let inner_w = (inner_w - RESET_GUTTER_W).max(0.0);
    let mut curve_widgets = CurveWidgets::new();
    let mut gradient_widgets = ColourRowWidgets::new();
    // Dobrada até o próximo cabeçalho. Uma row DOBRADA é pulada no desenho e mantém o índice
    // `i` — os eventos enumeram a MESMA lista, então pular no pintor sem pular no roteador
    // desalinharia os slots (o bug clássico de lista filtrada).
    let mut collapsed = false;
    for (i, row) in rows.iter().enumerate().take(MAX_PARAM_ROWS) {
        if let Some((dy, folded)) = sections::header_at(
            section_at,
            i,
            inner_x,
            inner_w + RESET_GUTTER_W,
            y,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        ) {
            y += dy;
            collapsed = folded;
        }
        if collapsed {
            continue;
        }
        if row_is_modified(row, modified) {
            paint_reset_button(
                i,
                inner_x,
                inner_w + RESET_GUTTER_W,
                y,
                store,
                hit_index,
                scene,
                theme,
            );
        }
        match row {
            // A DRIVEN param (doc 58): the wire decides the number, so there is no widget —
            // just the label and the live value. Nothing is registered, so nothing can be
            // dragged; the artist unplugs the wire to take the knob back.
            ParamRow::Scalar(row) if row.driven => {
                paint_driven_row(
                    row,
                    inner_x,
                    inner_w,
                    y,
                    label_font,
                    scene,
                    text_system,
                    theme,
                );
                y += ROW_H_PX + row_gap;
            }
            ParamRow::Scalar(row) => {
                y = paint_scalar_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    chip_w,
                    row_gap,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Color(row) => {
                y = paint_color_row(
                    row,
                    inner_x,
                    inner_w,
                    row_gap,
                    y,
                    label_font,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Toggle(row) => {
                y = paint_toggle_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    row_gap,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Enum(row) => {
                y = paint_enum_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    row_gap,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Angle(_) | ParamRow::Seed(_) | ParamRow::Text(_) => {
                // As rows-CAIXA: uma altura de row, um `Rect` explícito.
                let used = number::paint_box_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                )
                .expect("o braço e a porta casam por construção");
                y += used + row_gap;
            }
            ParamRow::Channels(row) => {
                y = paint_channels_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    row_gap,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Source(row) => {
                y = paint_source_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    row_gap,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Curve(_) | ParamRow::Palette(_) | ParamRow::Gradient(_) => {
                // Os editores de várias linhas — cada um devolve a própria altura.
                let used = editors::paint_editor_row(
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
                    &mut curve_widgets,
                    &mut gradient_widgets,
                )
                .expect("o braço e a porta casam por construção");
                y += used + row_gap;
            }
        }
    }
    (curve_widgets, gradient_widgets, y - body_top)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{RowDisplay, ScalarRow};
    use ph2d_editor_core::screens::layout::INSPECTOR_MAX_H;

    fn scalar(i: usize) -> ParamRow {
        ParamRow::Scalar(ScalarRow {
            name: "p",
            label: format!("Param {i}"),
            value: 0.5,
            min: 0.0,
            max: 1.0,
            hard_min: 0.0,
            hard_max: 1.0,
            step: 0.01,
            integer: false,
            driven: false,
            display: RowDisplay::default(),
        })
    }

    /// **O teto de linhas cabe no dock.**
    ///
    /// O irmão deste gate mora na shell (`the_panel_shows_every_param_of_every_node`) e prende o
    /// teto por BAIXO — nenhum nó do registry pode ter mais linhas que ele. Este o prende por
    /// CIMA: as linhas que ele permite têm de caber na altura que o inspector tem. Sem os dois,
    /// "conserte o corte" tem a resposta trivial de subir o teto até a linha 14 passar a ser
    /// cortada pela borda da tela em vez do `.take()` — a MESMA invisibilidade por outra porta,
    /// e a que nenhum teste de contagem vê.
    ///
    /// A fixture é PESSIMISTA de propósito: `MAX_PARAM_ROWS` linhas escalares, que é mais linhas
    /// do que qualquer nó real declara. Um dia em que ela deixe de caber é um dia em que o painel
    /// precisa rolar, e é isso que a falha vai dizer.
    #[test]
    fn a_full_panel_of_rows_fits_the_inspector() {
        let rows: Vec<ParamRow> = (0..MAX_PARAM_ROWS).map(scalar).collect();
        let mut hit = HitIndex::default();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let store = WidgetStore::default();
        let (_, _, used) = paint_rows(
            &rows,
            0.0,
            ph2d_editor_core::screens::layout::INSPECTOR_W,
            64.0,
            Spacing::Sm.px(),
            0.0,
            TypeToken::Base.px(),
            &store,
            &mut hit,
            &mut scene,
            &mut text,
            Theme::default(),
            &Default::default(),
            &[],
        );
        // O corpo começa abaixo do título; a folga que sobra é o que ele custa.
        let head = 64.0; // LITERAL-PX-OK: folga de titulo+padding, generosa de proposito
        assert!(
            used + head <= INSPECTOR_MAX_H,
            "{MAX_PARAM_ROWS} linhas ocupam {used} px e o inspector tem {INSPECTOR_MAX_H}: o \
             teto de linhas passou da altura do dock, entao a ultima linha e cortada pela borda \
             da tela — o painel precisa ROLAR antes de o teto subir mais"
        );
        // E a metade oposta: um painel que nao desenha nada tambem "cabe".
        assert!(used > 0.0, "as linhas nao ocuparam altura nenhuma");
    }
}
