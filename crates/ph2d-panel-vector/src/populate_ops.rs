//! **O registro das OPERAÇÕES** do painel vetorial — irmão de `populate.rs` pelo teto de 600 LOC
//! dos painéis (ele estava em 613 quando este nasceu).
//!
//! O corte é por RESPONSABILIDADE: aqui ficam os comandos que AGEM sobre a seleção (vértice,
//! topologia, booleana, regra de preenchimento, o ímã, o tipo de fill, alinhamento); no irmão
//! ficam a forma e a moldura dela (catálogo, arrange, transform, seções).

use super::{button, slider_chip};
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_tool_vector::params::{
    OFFSET_DEFAULT_FRAC, OFFSET_SLIDER_OFFSET, OFFSET_SLIDER_SCALE, WPROFILE_DEFAULT_END,
    WPROFILE_DEFAULT_MID, WPROFILE_DEFAULT_POS, WPROFILE_DEFAULT_START, WPROFILE_SLIDER_OFFSET,
    WPROFILE_SLIDER_SCALE, offset_frac_to_slider, wprofile_to_slider,
};

/// Passo do campo numérico do Offset, em fração do comprimento do caminho.
const TEXTPATH_OFFSET_STEP: f64 = 0.01; // LITERAL-PX-OK: passo no domínio do documento

/// Vértice, topologia (as três da W4 + o corte), Boolean + Compound, regra de preenchimento,
/// o ímã, o tipo de Fill e Align/Distribute.
pub(super) fn populate_ops(store: &mut WidgetStore) {
    // Vertex-type buttons (retype the selected vertex; shown only when a vertex
    // is selected, but registered unconditionally — the store is mode-agnostic)
    // + the Delete-node button.
    button(store, ids::VECTOR_VERT_CORNER);
    button(store, ids::VECTOR_VERT_SMOOTH);
    button(store, ids::VECTOR_VERT_SYMMETRIC);
    button(store, ids::VECTOR_VERT_DELETE);
    // As três da W4 (Join · Reverse · Average).
    button(store, ids::VECTOR_VERT_AVERAGE);
    button(store, ids::VECTOR_PATH_JOIN);
    button(store, ids::VECTOR_PATH_REVERSE);
    // Os dois da LINHA DE CORTE. Registrados INCONDICIONALMENTE (a store é agnóstica de estado),
    // embora só sejam PINTADOS com lâmina desenhada — as duas perguntas são diferentes, e
    // confundi-las é o que deixa um botão vivo na tela e morto sob o mouse.
    button(store, ids::VECTOR_CUT_APPLY);
    button(store, ids::VECTOR_CUT_DISCARD);
    button(store, ids::VECTOR_VERT_SEL_SUBPATH);
    button(store, ids::VECTOR_VERT_SEL_SAME);

    // Boolean op buttons (N-ary over the SELECTED closed regions) + compound row.
    super::blend::populate_blend(store);
    super::envelope::populate_envelope(store);
    // Text on Path (plano 22): os quatro controles + o slider de offset. Registrados
    // INCONDICIONALMENTE, como todos os irmãos — o store é agnóstico de modo, e quem decide se
    // o clique é possível é a PINTURA (sem hit-rect não há Click).
    button(store, ids::VECTOR_TEXTPATH_LINK);
    button(store, ids::VECTOR_TEXTPATH_PICK);
    button(store, ids::VECTOR_TEXTPATH_DETACH);
    button(store, ids::VECTOR_TEXTPATH_FLIP);
    button(store, ids::VECTOR_TEXTPATH_FLIP_OFF);
    // O offset é uma FRAÇÃO do comprimento (o `startOffset` do SVG): track e valor coincidem,
    // então a escala é 1 e o deslocamento 0 — o único slider do painel em que isso é verdade,
    // e é o que torna o campo legível (0.50 é meio caminho, em qualquer curva).
    slider_chip(
        store,
        ids::VECTOR_TEXTPATH_OFFSET,
        ids::VECTOR_TEXTPATH_OFFSET_NUM,
        0.0,
        0.0,
        1.0,
        0.0,
    );
    store.set_number_range(
        ids::VECTOR_TEXTPATH_OFFSET_NUM,
        0.0,
        1.0,
        TEXTPATH_OFFSET_STEP,
    );
    // Pattern on Path (plano 23): os quatro botões + os dois sliders, num irmão pelo teto de LOC.
    super::patternpath::populate_patternpath(store);
    // Contour (pesquisa `20_*` #9): os três comandos, os dois pares exclusivos e os três sliders.
    super::contour::populate_contour(store);
    // Filters (FX raster, plano 24): os quatro chips de tipo + os quatro pares slider/campo.
    super::filters::populate_filters(store);
    super::effects::populate_effects(store);
    button(store, ids::VECTOR_BOOL_UNION);
    // A BOOLEANA VIVA (plano UI/UX W1): o par de modo + o commit. Sem estas linhas os tres
    // pintam e ficam MORTOS sob o mouse -- a falha exata que esta lista existe para prevenir.
    button(store, ids::VECTOR_BOOL_LIVE_OFF);
    button(store, ids::VECTOR_BOOL_LIVE_ON);
    button(store, ids::VECTOR_BOOL_APPLY);
    // As quatro da W5 — sem estas linhas os botoes pintam e ficam MORTOS sob o mouse.
    button(store, ids::VECTOR_BOOL_MINUS_BACK);
    button(store, ids::VECTOR_BOOL_TRIM);
    button(store, ids::VECTOR_BOOL_CROP);
    button(store, ids::VECTOR_BOOL_MERGE);
    button(store, ids::VECTOR_BOOL_SUBTRACT);
    button(store, ids::VECTOR_BOOL_INTERSECT);
    button(store, ids::VECTOR_BOOL_EXCLUDE);
    button(store, ids::VECTOR_COMPOUND_MAKE);
    button(store, ids::VECTOR_COMPOUND_RELEASE);
    // Expand — Outline Stroke + Offset Path (a seção irmã da Boolean).
    button(store, ids::VECTOR_EXPAND_SIDE_OUTER);
    button(store, ids::VECTOR_EXPAND_SIDE_INNER);
    button(store, ids::VECTOR_EXPAND_SIDE_BOTH);
    button(store, ids::VECTOR_EXPAND_JOIN_MITER);
    button(store, ids::VECTOR_EXPAND_JOIN_ROUND);
    button(store, ids::VECTOR_EXPAND_JOIN_BEVEL);
    button(store, ids::VECTOR_EXPAND_OFFSET_PATH);
    button(store, ids::VECTOR_EXPAND_OUTLINE_STROKE);
    button(store, ids::VECTOR_EXPAND_POWER_STROKE);
    // Os perfis nomeados (W2b): o TETO de botões, sempre. O `paint` desenha só os que a tabela
    // publica, então registrar de menos deixaria um perfil novo clicável-e-MORTO sob o mouse, e
    // registrar de mais é inerte. Espelho do laço dos presets de gaiola.
    for i in 0..ids::MAX_WIDTH_PRESETS {
        button(store, ids::vector_width_preset_id(i));
    }
    for (slider, chip, default) in [
        (
            ids::VECTOR_EXPAND_W_START,
            ids::VECTOR_EXPAND_W_START_NUM,
            WPROFILE_DEFAULT_START,
        ),
        (
            ids::VECTOR_EXPAND_W_MID,
            ids::VECTOR_EXPAND_W_MID_NUM,
            WPROFILE_DEFAULT_MID,
        ),
        (
            ids::VECTOR_EXPAND_W_END,
            ids::VECTOR_EXPAND_W_END_NUM,
            WPROFILE_DEFAULT_END,
        ),
    ] {
        slider_chip(
            store,
            slider,
            chip,
            wprofile_to_slider(default),
            default,
            WPROFILE_SLIDER_SCALE,
            WPROFILE_SLIDER_OFFSET,
        );
    }
    // A posição já É uma fração `0..1`: mapa identidade.
    slider_chip(
        store,
        ids::VECTOR_EXPAND_W_POS,
        ids::VECTOR_EXPAND_W_POS_NUM,
        WPROFILE_DEFAULT_POS as f32,
        WPROFILE_DEFAULT_POS,
        1.0,
        0.0,
    );
    // O chip numérico do Offset é PERCENTUAL do tamanho da seleção (−100..+100) — o mapa
    // fica estático e o rótulo nunca mente; o mundo-d resolve na shell (`offset_scale`).
    slider_chip(
        store,
        ids::VECTOR_EXPAND_OFFSET,
        ids::VECTOR_EXPAND_OFFSET_NUM,
        offset_frac_to_slider(OFFSET_DEFAULT_FRAC),
        OFFSET_DEFAULT_FRAC * 100.0, // LITERAL-PX-OK: unit conversion (fraction -> percent readout), not a design measure.
        OFFSET_SLIDER_SCALE,
        OFFSET_SLIDER_OFFSET,
    );
    // Fill rule (compound paths only — the row hides for a single contour).
    button(store, ids::VECTOR_FILL_RULE_NONZERO);
    button(store, ids::VECTOR_FILL_RULE_EVENODD);
    // Shape snapping (tool setting, always visible). The grid toggle is in the
    // editor's universal Grid Snap panel.
    button(store, ids::VECTOR_SNAP_OFF);
    button(store, ids::VECTOR_SNAP_ON);
    button(store, ids::VECTOR_SNAP_PATH_OFF);
    button(store, ids::VECTOR_SNAP_PATH_ON);
    button(store, ids::VECTOR_SNAP_CROSS_OFF);
    button(store, ids::VECTOR_SNAP_CROSS_ON);
    button(store, ids::VECTOR_SNAP_GUIDES_OFF);
    button(store, ids::VECTOR_SNAP_GUIDES_ON);
    button(store, ids::VECTOR_RULERS_OFF);
    button(store, ids::VECTOR_RULERS_ON);

    // Fill-type selector (Solid / Linear / Radial) — act on the selected path.
    button(store, ids::VECTOR_FILL_KIND_SOLID);
    button(store, ids::VECTOR_FILL_KIND_LINEAR);
    button(store, ids::VECTOR_FILL_KIND_RADIAL);
    button(store, ids::VECTOR_FILL_KIND_MULTI);
    button(store, ids::VECTOR_GRAD_ADD_POINT);
    button(store, ids::VECTOR_GRAD_REMOVE_POINT);
    button(store, ids::VECTOR_GRAD_ADD_STOP);
    button(store, ids::VECTOR_GRAD_REMOVE_STOP);
    // Align + Distribute (multi-path object selection).
    button(store, ids::VECTOR_PIVOT_EDIT);
    button(store, ids::VECTOR_ALIGN_LEFT);
    button(store, ids::VECTOR_ALIGN_HCENTER);
    button(store, ids::VECTOR_ALIGN_RIGHT);
    button(store, ids::VECTOR_ALIGN_TOP);
    button(store, ids::VECTOR_ALIGN_VCENTER);
    button(store, ids::VECTOR_ALIGN_BOTTOM);
    button(store, ids::VECTOR_DISTRIBUTE_H);
    button(store, ids::VECTOR_DISTRIBUTE_V);
}
