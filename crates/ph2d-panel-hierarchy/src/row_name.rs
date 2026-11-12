//! ⭐⭐ **O nome tem prioridade sobre os interruptores** (defeito do dono 11/11: «Collisio…», «Ground L…»
//! na largura padrão; o idioma das camadas do Figma). O olho, o cadeado e o grupo no valor NORMAL (à
//! vista, destravado) só se pintam na linha apontada ou escolhida, ou durante um arrasto que os pinta;
//! no valor de excepção (escondido, travado) ficam sempre. Os ícones de componente ficam: dizem o que o
//! objecto É. Um nome que mesmo assim não cabe sai com «…», e o balão da linha diz o nome inteiro.

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
