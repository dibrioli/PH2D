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
    Checkbox, CheckboxState, CheckboxValue, ColorSwatch, DEFAULT_LABEL_W, SwatchSize,
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
            ParamRow::Scalar(row) if row.driven_by.is_some() => {
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
            driven_by: None,
            display: RowDisplay::default(),
        })
    }

    /// **Quanta folga o dock ainda tem** — a sonda que decide quando o painel precisa ROLAR.
    ///
    /// O gate irmão afirma que `MAX_PARAM_ROWS` linhas CABEM; ele é binário e não diz *por
    /// quanto*. Esta imprime o número, porque a decisão que vem a seguir (a varredura PRO do
    /// doc 88 §B3, que dá a cada nó o conjunto completo de params) é exatamente a que consome
    /// essa folga — e escolher entre *"cabe mais uma família"* e *"o painel tem de rolar"*
    /// precisa do px, não do booleano.
    #[test]
    #[ignore = "sonda de medição, não gate"]
    fn measure_the_docks_headroom() {
        let mut hit = HitIndex::default();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let store = WidgetStore::default();
        let head = 64.0; // LITERAL-PX-OK: a mesma folga de titulo que o gate irmao usa
        let mut per_row = 0.0f32;
        for n in [1usize, 8, MAX_PARAM_ROWS] {
            let rows: Vec<ParamRow> = (0..n).map(scalar).collect();
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
            per_row = used / n as f32;
            println!(
                "{n:3} linhas: {used:7.1} px  (+{head} de titulo = {:7.1})  sobra {:7.1} de {INSPECTOR_MAX_H}",
                used + head,
                INSPECTOR_MAX_H - used - head
            );
        }
        println!("por linha: {per_row:.1} px");
        println!(
            "cabem {:.0} linhas escalares no dock inteiro",
            (INSPECTOR_MAX_H - head) / per_row
        );
    }

    /// **A altura reportada e a altura VERDADEIRA das linhas.**
    ///
    /// Este gate se chamava `a_full_panel_of_rows_fits_the_inspector` e afirmava que
    /// `MAX_PARAM_ROWS` linhas cabiam no dock — a defesa contra *"conserte o corte subindo o
    /// teto ate a ultima linha ser cortada pela borda da tela em vez do `.take()`"*. O
    /// doc-comment dele terminava dizendo: *"um dia em que ela deixe de caber e um dia em que o
    /// painel precisa ROLAR"*. Esse dia chegou (doc 88 §B3) e a rolagem existe, entao **caber
    /// deixou de ser requisito** — mante-la afirmada faria a proxima linha que nao coubesse
    /// pedir um teto MENOR, que e o oposto da cura.
    ///
    /// O que sobrevive, e agora e load-bearing, e a HONESTIDADE do numero: o scrollbar deriva
    /// `max_scroll` de `content_h - visible_h`, entao um `used` que saturasse na altura do dock
    /// convenceria o painel de que tudo cabe e o artista **perderia a cauda em silencio** — com
    /// as linhas desenhadas, o thumb ausente e a roda inerte. Por isso a afirmacao e sobre
    /// CRESCIMENTO: dobrar as linhas dobra a altura, e o dock nao entra na conta.
    #[test]
    fn the_reported_height_is_the_true_height_of_the_rows() {
        // ⚠️ O teto mora DENTRO do `paint_rows` (`.take(MAX_PARAM_ROWS)`), entao pedir mais
        // linhas nao produz mais altura — 40 e 20 medem os MESMOS 544 px. A consequencia, que
        // vale mais que este gate: com o teto em `MAX_PARAM_ROWS` o corpo mede no maximo ~544 px
        // contra um dock de 880, entao **o painel nao transborda hoje** e a rolagem esta INERTE.
        // Ela nao e enfeite: e o que remove o dock da lista de razoes para o teto nao subir.
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
        // E a metade oposta: um painel que nao desenha nada tambem "cresce" trivialmente.
        assert!(used > 0.0, "as linhas nao ocuparam altura nenhuma");

        // Metade das linhas tem de custar (aproximadamente) metade da altura. A tolerancia e de
        // UMA linha porque o passo entre secoes nao e exatamente o passo entre linhas.
        let half: Vec<ParamRow> = (0..MAX_PARAM_ROWS / 2).map(scalar).collect();
        let mut hit2 = HitIndex::default();
        let mut scene2 = VectorScene::new();
        let mut text2 = TextSystem::without_system_fonts();
        let (_, _, half_used) = paint_rows(
            &half,
            0.0,
            ph2d_editor_core::screens::layout::INSPECTOR_W,
            64.0,
            Spacing::Sm.px(),
            0.0,
            TypeToken::Base.px(),
            &store,
            &mut hit2,
            &mut scene2,
            &mut text2,
            Theme::default(),
            &Default::default(),
            &[],
        );
        let per_row = used / MAX_PARAM_ROWS as f32;
        assert!(
            (used - 2.0 * half_used).abs() < per_row,
            "{MAX_PARAM_ROWS} linhas medem {used} px e {} medem {half_used}: a altura reportada \
             nao esta crescendo com as linhas, entao ela foi clampada em algum lugar — e um \
             painel que reporta menos conteudo do que desenha rola de menos e PERDE a cauda",
            MAX_PARAM_ROWS / 2
        );
        // E o fato que decide a wave, afirmado em vez de suposto: no teto de HOJE o corpo cabe
        // no dock com folga, entao a rolagem nunca dispara. O dia em que esta assercao virar
        // vermelha e o dia em que ela passa a trabalhar — e nao ha nada a consertar nele.
        assert!(
            used < INSPECTOR_MAX_H,
            "{MAX_PARAM_ROWS} linhas medem {used} px num dock de {INSPECTOR_MAX_H}: o painel \
             passou a TRANSBORDAR, e a rolagem (doc 88 §B3) deixou de ser inerte"
        );
    }
}
