//! W11 do Kanban — os scripts que um cartão pediu para abrir (`kanban_links::open`): o editor de
//! scripts drena-os em [`crate::frame::frame`] e abre a aba de cada um.

use ph2d_editor_core::kanban_links::{Family, Target, take};
use ph2d_script::ScriptId;

/// Os scripts pedidos desde o último quadro, pela ordem (o último fica com o foco).
#[must_use]
pub(crate) fn scripts_asked() -> Vec<ScriptId> {
    take(Family::Script)
        .into_iter()
        .filter_map(|t| match t {
            Target::Script(s) => Some(ScriptId(s)),
            _ => None,
        })
        .collect()
}
