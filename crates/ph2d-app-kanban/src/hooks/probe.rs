//! ⚠️ SONDA da W6 no elemento do Quadro (a vista vive no canvas, não entre as abas).

use crate::docs::KanbanDocs;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::CardId;

/// O que a W6 da vista de um elemento mostra.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasW6 {
    /// O painel por cima (`Filter`, `Search`…).
    pub over: Option<String>,
    /// O cartão da edição rápida.
    pub quick: Option<CardId>,
    /// Os alvos pintados no último quadro (`Quick(Archive)`…), em px da vista.
    pub hits: Vec<(String, Rect)>,
}

impl KanbanDocs {
    /// ⚠️ SONDA: a W6 da vista do elemento `el` do Quadro.
    #[must_use]
    pub fn probe_canvas_w6(&self, el: u64) -> Option<CanvasW6> {
        let c = self.canvas.borrow();
        let t = c.tabs.get(&ph2d_board_model::ElementId(el))?;
        Some(CanvasW6 {
            over: t.w6.over.map(|o| format!("{o:?}")),
            quick: t.w6.quick,
            hits: t
                .w6
                .hits
                .borrow()
                .iter()
                .map(|(r, a)| (format!("{a:?}"), *r))
                .collect(),
        })
    }
}
