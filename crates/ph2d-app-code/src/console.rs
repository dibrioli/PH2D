//! ⭐ **E10 — o console de UM script**: a faixa de baixo da aba. O `Store` (as linhas, o separador e os
//! problemas do Play por `ScriptId`) vive em `ph2d_panel_code::console_store`; aqui, as sondas da família.

use ph2d_editor_core::zones::Rect;
use ph2d_script::ScriptId;

pub(crate) use ph2d_panel_code::console_store::{PlayKind, PlayProblem, Store};

impl crate::docs::CodeDocs {
    /// As linhas do console do script `id`, como a aba as mostra.
    #[must_use]
    pub fn console_texts(&self, id: ScriptId) -> Vec<String> {
        self.consoles.get(&id).map(Store::texts).unwrap_or_default()
    }

    /// O «quem · quando» de cada linha do console do script `id`.
    #[must_use]
    pub fn console_tags(&self, id: ScriptId) -> Vec<Option<String>> {
        self.consoles.get(&id).map(Store::tags).unwrap_or_default()
    }

    #[must_use]
    pub fn console_open(&self, id: ScriptId) -> bool {
        self.consoles.get(&id).is_some_and(|s| s.open)
    }

    /// Onde o console pintou a linha `i` no último quadro.
    #[must_use]
    pub fn console_row_rect(&self, id: ScriptId, i: usize) -> Option<Rect> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .and_then(|d| d.console_view.row_rect(i))
    }
}
