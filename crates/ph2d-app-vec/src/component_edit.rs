//! **O QUE UM CLIQUE NA SECÇÃO *Prefab* PEDE** — a tabela `id → verbo`, e mais nada.
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `component_edit.rs`.

/// O que um clique num verbo de componente PEDE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentEdit {
    Create,
    /// ⭐⭐⭐ **Edit Prefab** — abre a RECEITA desta cópia (2026-09-07). Ver
    /// [`crate::instance_verbs::Verb::Edit`]: no modelo geral a receita está escondida do canvas,
    /// e até aqui só o cartão da biblioteca lá chegava.
    Edit,
    Place,
    /// ⭐⭐ **Instantiate Linked** (2026-09-07) — a cópia divide a ARTE da receita.
    PlaceLinked,
    Detach,
    Reset,
    /// **Apply to Prefab** — as diferenças desta cópia passam a ser a receita.
    UpdateMain,
    /// **Swap Prefab** — arma o conta-gotas; o clique seguinte no canvas escolhe o prefab.
    Swap,
}

/// Este id é um verbo de componente? Porta única do roteador.
#[must_use]
pub fn component_edit_for_id(id: ph2d_editor_core::NodeId) -> Option<ComponentEdit> {
    match id {
        _ if id == ph2d_panel_vector::ids::VECTOR_COMPONENT_CREATE => Some(ComponentEdit::Create),
        _ if id == ph2d_panel_vector::ids::VECTOR_COMPONENT_EDIT => Some(ComponentEdit::Edit),
        _ if id == ph2d_panel_vector::ids::VECTOR_COMPONENT_PLACE => Some(ComponentEdit::Place),
        _ if id == ph2d_panel_vector::ids::VECTOR_COMPONENT_PLACE_LINKED => {
            Some(ComponentEdit::PlaceLinked)
        }
        _ if id == ph2d_panel_vector::ids::VECTOR_COMPONENT_DETACH => Some(ComponentEdit::Detach),
        _ if id == ph2d_panel_vector::ids::VECTOR_COMPONENT_RESET => Some(ComponentEdit::Reset),
        _ if id == ph2d_panel_vector::ids::VECTOR_COMPONENT_UPDATE_MAIN => {
            Some(ComponentEdit::UpdateMain)
        }
        _ if id == ph2d_panel_vector::ids::VECTOR_COMPONENT_SWAP => Some(ComponentEdit::Swap),
        _ => None,
    }
}
