//! **Toda caixa numérica está LIGADA ao slider dela.**
//!
//! Um par slider+chip é **um** controlo: arrastar e digitar têm de dizer a mesma coisa. Quem os
//! liga é o `store.link_slider_number_mapped*` do `populate` — e sem ele o slider funciona
//! perfeitamente e **a caixa fica muda**, que foi exatamente o que o Enio viu no Steps do Blend
//! (2026-07-16).
//!
//! # Este gate não é um proxy: ele afirma a CONDIÇÃO do bug
//!
//! O espelho vive no `commit_number_buffer` do editor-core, e o galho é literal:
//!
//! ```ignore
//! if let Some(slider_id) = store.linked_slider(id) {
//!     events.push(WidgetEvent::ValueChanged(slider_id));
//! }
//! ```
//!
//! Sem o link, `linked_slider` devolve `None`, **nenhum `ValueChanged` do slider é emitido**, o
//! `event.rs` do painel nunca vê nada, nada chega ao barramento, e o blend nunca é re-afinado. A
//! caixa fica registrada, pintada, focável e editável — e inerte. É a classe de bug que a DIRETIVA
//! chama de "faltar uma ponta = clique dropado em silêncio", e nenhum gate de contagem de símbolo
//! a pega: os dois widgets EXISTEM.
//!
//! (Que o espelho funciona **quando ligado** é provado no editor-core, em
//! `tests/number_input_mapped_link.rs`. Que o `ValueChanged(slider)` chega ao barramento é provado
//! no `seam.rs` deste painel. Este gate fecha o elo que faltava entre os dois.)

use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

/// Os pares `(slider, chip)` do painel. **Todo id `*_NUM` daqui é o chip de um slider** — os campos
/// numéricos avulsos do painel (Transform X/Y/W/H) não usam o sufixo, então a regra é mecânica e a
/// lista não tem como escolher errado.
const PAIRS: &[(&str, ph2d_a11y::NodeId, ph2d_a11y::NodeId)] = &[
    ("Width", ids::VECTOR_WIDTH, ids::VECTOR_WIDTH_NUM),
    ("Dash", ids::VECTOR_DASH, ids::VECTOR_DASH_NUM),
    ("Gap", ids::VECTOR_GAP, ids::VECTOR_GAP_NUM),
    (
        "Stroke Opacity",
        ids::VECTOR_STROKE_OPACITY,
        ids::VECTOR_STROKE_OPACITY_NUM,
    ),
    (
        "Fill Opacity",
        ids::VECTOR_FILL_OPACITY,
        ids::VECTOR_FILL_OPACITY_NUM,
    ),
    (
        "Grad Angle",
        ids::VECTOR_GRAD_ANGLE,
        ids::VECTOR_GRAD_ANGLE_NUM,
    ),
    (
        "Grad Influence",
        ids::VECTOR_GRAD_INFLUENCE,
        ids::VECTOR_GRAD_INFLUENCE_NUM,
    ),
    (
        "Grad Jitter",
        ids::VECTOR_GRAD_JITTER,
        ids::VECTOR_GRAD_JITTER_NUM,
    ),
    (
        "Text Size",
        ids::VECTOR_TEXT_SIZE,
        ids::VECTOR_TEXT_SIZE_NUM,
    ),
    (
        "Text Weight",
        ids::VECTOR_TEXT_WEIGHT,
        ids::VECTOR_TEXT_WEIGHT_NUM,
    ),
    (
        "Text Tracking",
        ids::VECTOR_TEXT_TRACKING,
        ids::VECTOR_TEXT_TRACKING_NUM,
    ),
    (
        "Text Line Height",
        ids::VECTOR_TEXT_LINE_HEIGHT,
        ids::VECTOR_TEXT_LINE_HEIGHT_NUM,
    ),
    (
        "Blend Steps",
        ids::VECTOR_BLEND_STEPS,
        ids::VECTOR_BLEND_STEPS_NUM,
    ),
    ("Morph t", ids::VECTOR_MORPH_T, ids::VECTOR_MORPH_T_NUM),
];

#[test]
fn every_number_chip_is_linked_to_its_slider_in_both_directions() {
    let host = MockPanelHost::with_panel::<VectorPanel>();
    let store = host.store();

    let mut dead: Vec<&str> = Vec::new();
    for (name, slider, chip) in PAIRS {
        // Os dois sentidos: o chip acha o slider (é o que emite o `ValueChanged` no commit) **e** o
        // slider acha o chip (é o que atualiza o número enquanto se arrasta). Um link de mão única
        // dá metade do controlo.
        if store.linked_slider(*chip) != Some(*slider)
            || store.linked_number(*slider) != Some(*chip)
        {
            dead.push(name);
        }
    }

    // ⚠️ **Os pares por-LINHA do Filters não cabem numa lista `const`** — os ids são derivados do
    // índice da linha —, e é por isso que a seção inteira estava FORA deste gate desde que ela
    // existe. Uma lista escrita à mão só protege o que alguém lembrou de listar; o laço protege o
    // par que a próxima wave acrescentar.
    for r in 0..ids::MAX_FILTER_ROWS {
        for (name, slider, chip) in [
            (
                "Filter Radius",
                ids::filter_radius_id(r),
                ids::filter_radius_num_id(r),
            ),
            (
                "Filter Off X",
                ids::filter_offx_id(r),
                ids::filter_offx_num_id(r),
            ),
            (
                "Filter Off Y",
                ids::filter_offy_id(r),
                ids::filter_offy_num_id(r),
            ),
            (
                "Filter Opacity",
                ids::filter_opacity_id(r),
                ids::filter_opacity_num_id(r),
            ),
            (
                "Filter Size",
                ids::filter_scale_id(r),
                ids::filter_scale_num_id(r),
            ),
            (
                "Filter Detail",
                ids::filter_detail_id(r),
                ids::filter_detail_num_id(r),
            ),
            (
                "Filter Seed",
                ids::filter_seed_id(r),
                ids::filter_seed_num_id(r),
            ),
        ] {
            if store.linked_slider(chip) != Some(slider)
                || store.linked_number(slider) != Some(chip)
            {
                dead.push(name);
            }
        }
    }

    assert!(
        dead.is_empty(),
        "a caixa numérica destes controlos NÃO está ligada ao slider: {dead:?}\n\
         O slider funciona e a caixa fica MUDA — digitar nela não emite `ValueChanged` do slider \
         (é o `if let Some(slider_id) = store.linked_slider(id)` do `commit_number_buffer`), então \
         nada chega ao barramento e a feature não é re-afinada.\n\
         Conserto: registre o par com o helper `slider_chip` do `populate.rs` (que liga), ou chame \
         `store.link_slider_number_mapped{{,_integer}}(slider, chip, scale, offset)` à mão."
    );
}
