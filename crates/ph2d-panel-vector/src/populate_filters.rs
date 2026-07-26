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

use super::{button, slider_chip};
use crate::ids;
use crate::state::filters::{FILTER_OFFSET_MAX, FILTER_RADIUS_MAX};
use ph2d_editor_core::interaction::WidgetStore;

/// Passo dos campos numéricos, no domínio do documento.
const RADIUS_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento (mundo)
const OFFSET_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento (mundo)
const OPACITY_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento (0..1)

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
        for id in [
            ids::filter_remove_id(row),
            ids::filter_up_id(row),
            ids::filter_down_id(row),
            ids::filter_hide_id(row),
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
        // Opacity: track == valor (`0..1`).
        let (opacity, opacity_num) = (ids::filter_opacity_id(row), ids::filter_opacity_num_id(row));
        slider_chip(store, opacity, opacity_num, 1.0, 1.0, 1.0, 0.0);
        store.set_number_range(opacity_num, 0.0, 1.0, OPACITY_STEP);
    }
}
