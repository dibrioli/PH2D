//! ⭐ **DE QUE SLOT É ESTE ID?** — irmão de `state.rs` pelo teto de 600 LOC do painel.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE, e aqui ele é nítido: nada disto é ESTADO.** Cada função
//! inverte uma fábrica de id do painel contra o espaço **fixo** de slots — *"este id é o campo do
//! parâmetro 2?"*, *"é a 5.ª opção do slot 1 das pontas?"* — enquanto o resto do `state.rs` responde
//! *"o que a shell publicou neste quadro?"*. Duas perguntas, dois ficheiros.
//!
//! ⚠️ **Casar contra o espaço FIXO (e não contra o que existe hoje) é deliberado**, e está escrito
//! em cada doc: a resolução não pode depender de quantas formas ou pontas o catálogo tem, senão um
//! id que já foi pintado deixa de se resolver quando a lista encolhe.

use super::with_font_previews;
use ph2d_a11y::NodeId;

/// Índice do parâmetro de forma cujo id de campo é `id` (`None` se não for um).
pub(crate) fn shape_field_index(id: NodeId) -> Option<usize> {
    (0..crate::ids::MAX_SHAPE_FIELD_SLOTS).find(|&i| crate::ids::vector_shape_field_id(i) == id)
}

/// Índice do parâmetro cujo **botão de escolha** é `id` (o gêmeo clicável do slot numérico).
pub(crate) fn shape_choice_index(id: NodeId) -> Option<usize> {
    (0..crate::ids::MAX_SHAPE_FIELD_SLOTS).find(|&i| crate::ids::vector_shape_choice_id(i) == id)
}

/// Índice da forma no catálogo cujo id de botão é `id`.
pub(crate) fn shape_index(id: NodeId) -> Option<usize> {
    (0..ph2d_tool_vector::shapes::SHAPES.len()).find(|&i| crate::ids::vector_shape_id(i) == id)
}

/// Índice da família cujo id de aba é `id`.
pub(crate) fn shape_group_index(id: NodeId) -> Option<usize> {
    (0..ph2d_tool_vector::shapes::ALL_GROUPS.len())
        .find(|&i| crate::ids::vector_shape_group_id(i) == id)
}

/// Índice do eixo de variação cujo id de campo é `id` (`None` se não for um). Casa
/// contra o espaço fixo de slots (`MAX_TEXT_VARIATION_AXES`).
pub(crate) fn text_axis_index(id: NodeId) -> Option<usize> {
    (0..crate::ids::MAX_TEXT_VARIATION_AXES).find(|&i| crate::ids::vector_text_axis_id(i) == id)
}

/// Índice da família cujo id de opção do dropdown é `id` (`None` se não for uma
/// opção de fonte). Casa contra as previews publicadas na ordem selecionável.
pub(crate) fn font_option_index(id: NodeId) -> Option<usize> {
    with_font_previews(|p| (0..p.len()).find(|&i| crate::ids::vector_text_font_option_id(i) == id))
}

/// `(slot, índice)` da opção de ponta cujo id é `id` (`None` se não for uma). Casa contra
/// o espaço FIXO de slots, como as outras fábricas de id do painel — a resolução não
/// depende de quantas pontas existem hoje.
pub(crate) fn marker_option(id: NodeId) -> Option<(usize, usize)> {
    (0..crate::ids::MARKER_SLOTS).find_map(|slot| {
        (0..crate::ids::MAX_MARKER_OPTIONS)
            .find(|&i| crate::ids::vector_marker_option_id(slot, i) == id)
            .map(|i| (slot, i))
    })
}
