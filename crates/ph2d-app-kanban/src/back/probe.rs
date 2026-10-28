//! ⚠️ As SONDAS do verso: o que o último quadro pintou e o estado da vista, para os testes da costura.

use crate::docs::KanbanDocs;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::CardId;

impl KanbanDocs {
    fn back_of(&self, id: u64) -> Option<&super::Back> {
        self.tabs.iter().find(|t| t.id() == id)?.back.as_deref()
    }

    /// ⚠️ SONDA: o cartão com o verso aberto na aba.
    #[must_use]
    pub fn probe_back(&self, id: u64) -> Option<CardId> {
        self.back_of(id).map(|b| b.card)
    }

    /// ⚠️ SONDA: os alvos que o último quadro pintou no verso (`Side(Labels)`, `Pop(Save)`…), com o
    /// rect de cada um, pela ordem (o último por cima).
    #[must_use]
    pub fn probe_back_hits(&self, id: u64) -> Vec<(String, Rect)> {
        self.back_of(id)
            .map(|b| b.hits.iter().map(|(r, h)| (format!("{h:?}"), *r)).collect())
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: a atividade que o último quadro pintou (`comment:<autor>:<texto>`,
    /// `activity:<frase>`), da mais nova para a mais antiga.
    #[must_use]
    pub fn probe_back_feed(&self, id: u64) -> Vec<String> {
        self.back_of(id).map(|b| b.feed.clone()).unwrap_or_default()
    }

    /// ⚠️ SONDA: o popover aberto (o nome da variante) e o texto aberto: `(popover, campo, descrição
    /// em edição, comentário em edição)`.
    #[must_use]
    pub fn probe_back_state(&self, id: u64) -> (Option<String>, Option<String>, bool, bool) {
        let Some(b) = self.back_of(id) else {
            return (None, None, false, false);
        };
        let pop = b.pop.as_ref().map(|p| {
            let s = format!("{p:?}");
            s.split(['(', ' ']).next().unwrap_or_default().to_owned()
        });
        (
            pop,
            b.edit.as_ref().map(|e| e.field.text.clone()),
            b.desc.editing,
            b.comment.editing,
        )
    }

    /// Os links que o artista pediu para abrir no verso desde a última chamada (a shell abre-os).
    pub fn take_opened_links(&mut self, id: u64) -> Vec<String> {
        self.tab_mut(id)
            .and_then(|t| t.back.as_deref_mut())
            .map(|b| std::mem::take(&mut b.opened))
            .unwrap_or_default()
    }
}
