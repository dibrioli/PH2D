//! O registro dos widgets do **FILTERS** (a pilha de FX raster, plano 24) — irmão do [`super`]
//! pelo teto de 600 LOC, e par natural do `paint_filters` (que os PINTA).
//!
//! Registrar é o que os torna clicáveis: pintar + hit-rect não basta.
//!
//! ⚠️ O registro é pelo **TETO de linhas**, não pelo tamanho da pilha corrente — o `populate` corre
//! antes de a shell publicar o estado do frame, e um slot registado a menos é um controle pintado
//! e morto sob o mouse. É o padrão da pilha de Effects e da matriz de camadas da física.
//!
//! ⚠️ Ids derivados em LAÇO são invisíveis ao `architecture_panel_wiring_parity` (ele só coleta
//! `.register(ids::LITERAL`), então quem cobre esta seção é o seam que CLICA cada controle.

use super::{button, slider_chip, slider_chip_int};
use crate::ids;
use crate::state::filters::{
    FILTER_ADJUST_MAX, FILTER_DETAIL_MAX, FILTER_GROW_MAX, FILTER_HUE_MAX, FILTER_OFFSET_MAX,
    FILTER_RADIUS_MAX, FILTER_SCALE_MAX, FILTER_SEED_MAX,
};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::DropdownState;

/// Passo dos campos numéricos, no domínio do documento.
const RADIUS_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento (mundo)
const HUE_STEP: f64 = 5.0; // LITERAL-PX-OK: passo em GRAUS de matiz
const ADJUST_STEP: f64 = 0.05; // LITERAL-PX-OK: passo da saturação/brilho, cujo domínio é -1..1
const OFFSET_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento (mundo)
const OPACITY_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento (0..1)
const SCALE_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento (mundo)
/// Detail e Seed são CONTAGENS: o passo é a unidade, não uma fração dela.
const COUNT_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento (contagem)

/// Os botões "Add" (um por tipo) + o bloco de controles de cada linha do teto.
pub(super) fn populate_filters(store: &mut WidgetStore) {
    for kind in 0..ids::MAX_FILTER_KINDS {
        button(store, ids::filter_add_id(kind));
    }
    for row in 0..ids::MAX_FILTER_ROWS {
        // O cabeçalho do card: reordenar, desarmar, apagar.
        //
        // ⚠️ **A swatch de cor NÃO entra aqui.** Ela é alvo de PICKER (`register_picker_swatch`,
        // no passe de sementes), e um id só pode ter UM tipo de widget no store — registá-la como
        // botão faz o Down abrir o picker e nenhum `Click` sair, que foi exatamente o que o seam
        // gate pegou. É a mesma lição do `vector_fx_toggle_id`.
        for m in 0..ids::MAX_FILTER_MODES {
            button(store, ids::filter_mode_id(row, m));
        }
        for id in [
            ids::filter_remove_id(row),
            ids::filter_up_id(row),
            ids::filter_down_id(row),
            ids::filter_hide_id(row),
            // O `+` / `−` do trilho da rampa. ⚠️ Os PUNHOS não entram: eles são `CurvePoint`, e um
            // id só pode ter UM tipo de widget no store — registá-los como botão faria o Down
            // consumir o arrasto e nenhum stop se moveria.
            ids::filter_stop_add_id(row),
            ids::filter_stop_remove_id(row),
        ] {
            button(store, id);
        }
        // Radius: track `0..1` → `0..FILTER_RADIUS_MAX`. O `scale`/`offset` do chip é o MESMO mapa
        // que o `event` desfaz na fronteira, senão slider e campo divergiriam.
        let (radius, radius_num) = (ids::filter_radius_id(row), ids::filter_radius_num_id(row));
        slider_chip(
            store,
            radius,
            radius_num,
            0.0,
            0.0,
            FILTER_RADIUS_MAX as f32,
            0.0,
        );
        store.set_number_range(radius_num, 0.0, FILTER_RADIUS_MAX, RADIUS_STEP);
        // Offset X/Y: BIPOLAR `−MAX..MAX`, `0.5` = zero.
        for (slider, chip) in [
            (ids::filter_offx_id(row), ids::filter_offx_num_id(row)),
            (ids::filter_offy_id(row), ids::filter_offy_num_id(row)),
        ] {
            slider_chip(
                store,
                slider,
                chip,
                0.5,
                0.0,
                (2.0 * FILTER_OFFSET_MAX) as f32,
                -FILTER_OFFSET_MAX as f32,
            );
            store.set_number_range(chip, -FILTER_OFFSET_MAX, FILTER_OFFSET_MAX, OFFSET_STEP);
        }
        // A LEI DE MISTURA: o chip é um `Dropdown` (abrir/fechar/roda vêm de graça do dispatch
        // genérico) + uma opção por lei. Registradas por ÍNDICE, como as pontas do traço: uma lei
        // nova entra na tabela publicada e já nasce clicável, sem tocar aqui.
        //
        // ⚠️ Registram-se as VINTE em TODA linha, mesmo nas que o tipo não oferece o controle — o
        // `populate` corre antes de a shell publicar o estado, e o `paint` é quem decide o que
        // registra hit. Um slot a menos é uma opção pintada e morta sob o mouse.
        store.register_if_absent(
            ids::filter_blend_id(row),
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
        for i in 0..ids::MAX_FILTER_BLENDS {
            button(store, ids::filter_blend_option_id(row, i));
        }
        // Os três knobs do RUÍDO. Registrados em TODA linha, como as leis de mistura: o
        // `populate` corre antes de a shell publicar o estado, e um slot a menos é um controle
        // pintado e morto sob o mouse.
        let (scale, scale_num) = (ids::filter_scale_id(row), ids::filter_scale_num_id(row));
        slider_chip(
            store,
            scale,
            scale_num,
            0.0,
            0.0,
            FILTER_SCALE_MAX as f32,
            0.0,
        );
        store.set_number_range(scale_num, 0.0, FILTER_SCALE_MAX, SCALE_STEP);
        let (detail, detail_num) = (ids::filter_detail_id(row), ids::filter_detail_num_id(row));
        slider_chip_int(
            store,
            detail,
            detail_num,
            0.0,
            1.0,
            (FILTER_DETAIL_MAX - 1.0) as f32,
            1.0,
        );
        store.set_number_range(detail_num, 1.0, FILTER_DETAIL_MAX, COUNT_STEP);
        let (seed, seed_num) = (ids::filter_seed_id(row), ids::filter_seed_num_id(row));
        slider_chip_int(store, seed, seed_num, 0.0, 0.0, FILTER_SEED_MAX as f32, 0.0);
        store.set_number_range(seed_num, 0.0, FILTER_SEED_MAX, COUNT_STEP);
        // **Amount** do Grow / Shrink: BIPOLAR `−MAX..MAX`, `0.5` = zero — a mesma régua do par de
        // offset, e pela mesma razão: o número tem SINAL, e o meio do curso é o neutro.
        let (grow, grow_num) = (ids::filter_grow_id(row), ids::filter_grow_num_id(row));
        slider_chip(
            store,
            grow,
            grow_num,
            0.5,
            0.0,
            (2.0 * FILTER_GROW_MAX) as f32,
            -FILTER_GROW_MAX as f32,
        );
        store.set_number_range(grow_num, -FILTER_GROW_MAX, FILTER_GROW_MAX, RADIUS_STEP);
        // **Os três do Color Adjust**, todos BIPOLARES (`0.5` = neutro) — a mesma régua do Amount
        // e do par de offset, e pela mesma razão: os três têm SINAL e o neutro é o meio do curso.
        let (hue, hue_num) = (ids::filter_hue_id(row), ids::filter_hue_num_id(row));
        slider_chip(
            store,
            hue,
            hue_num,
            0.5,
            0.0,
            (2.0 * FILTER_HUE_MAX) as f32,
            -FILTER_HUE_MAX as f32,
        );
        store.set_number_range(hue_num, -FILTER_HUE_MAX, FILTER_HUE_MAX, HUE_STEP);
        for (slider, chip) in [
            (ids::filter_sat_id(row), ids::filter_sat_num_id(row)),
            (ids::filter_bright_id(row), ids::filter_bright_num_id(row)),
        ] {
            slider_chip(
                store,
                slider,
                chip,
                0.5,
                0.0,
                (2.0 * FILTER_ADJUST_MAX) as f32,
                -FILTER_ADJUST_MAX as f32,
            );
            store.set_number_range(chip, -FILTER_ADJUST_MAX, FILTER_ADJUST_MAX, ADJUST_STEP);
        }
        // Opacity: track == valor (`0..1`).
        let (opacity, opacity_num) = (ids::filter_opacity_id(row), ids::filter_opacity_num_id(row));
        slider_chip(store, opacity, opacity_num, 1.0, 1.0, 1.0, 0.0);
        store.set_number_range(opacity_num, 0.0, 1.0, OPACITY_STEP);
    }
}
