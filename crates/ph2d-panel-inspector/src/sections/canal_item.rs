//! ⭐ **A fileira de um ITEM de lista regista o canal do item** (F30 §A1) — a chave é o `ItemId`
//! (`ph2d_component_desc::lista::chaves`), nunca a posição: o losango segue o timer, o estado, a
//! seta… para onde a lista os mover. `item == 0` (por atribuir) não tem canal ainda.

use ph2d_component_desc::lista;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::key_mark;

/// Regista, para cada `(field_id, campos)`, os campos da fileira sob o canal desse campo do item.
pub(crate) fn regista(codigo: u16, item: u32, linhas: &[(u16, &[NodeId])]) {
    if item == 0 {
        return;
    }
    for &(field_id, campos) in linhas {
        key_mark::regista_canal(campos, &lista::chaves(codigo, field_id, item));
    }
}
