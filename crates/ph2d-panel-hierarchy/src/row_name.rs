//! ⭐⭐ **O nome tem prioridade sobre os interruptores** (defeito do dono 11/11: «Collisio…», «Ground L…»
//! na largura padrão; o idioma das camadas do Figma). O olho, o cadeado e o grupo no valor NORMAL (à
//! vista, destravado) só se pintam na linha apontada ou escolhida, ou durante um arrasto que os pinta;
//! no valor de excepção (escondido, travado) ficam sempre. Os ícones de componente seguem a mesma regra
//! (smoke do dono 11/11: em repouso, nenhum ícone à direita); com o rato voltam todos juntos, nas
//! posições de sempre. Um nome que mesmo assim não cabe sai com «…», e o balão diz o nome inteiro.

use ph2d_a11y::NodeId;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use std::cell::RefCell;
use std::collections::BTreeSet;

thread_local! {
    static CORTADOS: RefCell<BTreeSet<NodeId>> = const { RefCell::new(BTreeSet::new()) };
}

/// Os interruptores no valor normal pintam-se nesta linha?
pub(crate) fn toggles_all_shown(e: &HierarchyEntity) -> bool {
    e.selected || e.hovered || ph2d_editor_core::interaction::toggle_paint::em_curso()
}

/// O rato está sobre a linha ou sobre um dos ícones dela? (`hover_live` diz 1 a quem o relógio nunca viu.)
/// ⚠️ Os ícones de componente contam: sem eles, o rato num deles apagava os ícones e a linha mexia
/// sob o dedo.
pub(crate) fn under_pointer(
    store: &ph2d_editor_core::interaction::WidgetStore,
    id: NodeId,
    e: &HierarchyEntity,
) -> bool {
    use ph2d_editor_core::ids as core;
    store.hot_id().is_some_and(|h| {
        h == id
            || core::hier_expand_companion_to_row(h) == Some(id)
            || core::hier_eye_companion_to_row(h) == Some(id)
            || core::hier_lock_companion_to_row(h) == Some(id)
            || core::hier_group_companion_to_row(h) == Some(id)
            || core::hier_icon_companion_to_row(h) == Some(id)
            || e.components
                .iter()
                .any(|c| crate::ids::hier_component_icon(id, *c) == h)
    })
}

/// O pintor regista que o nome da linha `id` saiu cortado neste quadro.
pub(crate) fn mark_cut(id: NodeId) {
    CORTADOS.with_borrow_mut(|c| {
        c.insert(id);
    });
}

/// As linhas cortadas no quadro anterior; esvazia o registo para o quadro que começa.
pub(crate) fn take_cut() -> BTreeSet<NodeId> {
    CORTADOS.with_borrow_mut(std::mem::take)
}
