//! ⚠️ As SONDAS da W6: o que o último quadro pintou (os alvos dos painéis) e o estado da vista
//! (filtro, linhas mostradas, espelhos, idades, edição rápida, diálogo), para os testes da costura.

use crate::docs::KanbanDocs;
use crate::tab::Tab;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{CardId, CardRef, ListId};

impl KanbanDocs {
    fn w6_tab(&self, id: u64) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id() == id)
    }

    /// ⚠️ SONDA: os alvos da W6 pintados no último quadro (`FilterButton`, `Label(LabelId(3))`,
    /// `Dlg(Confirm)`…), com o rect de cada um, pela ordem (o último por cima).
    #[must_use]
    pub fn probe_w6_hits(&self, id: u64) -> Vec<(String, Rect)> {
        self.w6_tab(id)
            .map(|t| {
                t.w6.hits
                    .borrow()
                    .iter()
                    .map(|(r, a)| (format!("{a:?}"), *r))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: `(painel aberto, diálogo aberto, linha da edição rápida, escolhas do filtro)`.
    #[must_use]
    pub fn probe_w6(&self, id: u64) -> (Option<String>, Option<String>, Option<CardId>, usize) {
        let Some(t) = self.w6_tab(id) else {
            return (None, None, None, 0);
        };
        (
            t.w6.over.map(|o| format!("{o:?}")),
            t.w6.dialog.as_ref().map(|d| format!("{:?}", d.kind)),
            t.w6.quick,
            t.w6.filter.selected(),
        )
    }

    /// ⚠️ SONDA: as linhas que cada lista mostrou (com o filtro e os espelhos), pela ordem.
    #[must_use]
    pub fn probe_rows(&self, id: u64, list: ListId) -> Vec<CardId> {
        self.w6_tab(id)
            .and_then(|t| t.rows.lists.get(&list).cloned())
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: os espelhos mostrados — o id na disposição e o original.
    #[must_use]
    pub fn probe_mirrors(&self, id: u64) -> Vec<(CardId, CardRef)> {
        self.w6_tab(id)
            .map(|t| {
                t.rows
                    .mirrors
                    .iter()
                    .map(|(k, (_, of))| (*k, *of))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: o nível de envelhecimento de cada linha parada.
    #[must_use]
    pub fn probe_ages(&self, id: u64) -> Vec<(CardId, u8)> {
        self.w6_tab(id)
            .map(|t| t.rows.ages.iter().map(|(k, v)| (*k, *v)).collect())
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: os cartões que a pesquisa encontrou, pela relevância.
    #[must_use]
    pub fn probe_found(&self, id: u64) -> Vec<CardRef> {
        self.w6_tab(id)
            .map(|t| t.w6.found.iter().map(|h| h.card).collect())
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: os distintivos da frente da linha `row` (o texto de cada um), como o pintor os vê.
    #[must_use]
    pub fn probe_badges(&self, id: u64, row: CardId) -> Vec<String> {
        let Some(t) = self.w6_tab(id) else {
            return Vec::new();
        };
        let set = self.set.borrow();
        let Some(k) = set.get(t.view.kanban) else {
            return Vec::new();
        };
        let counts = crate::front::Counts::of(k);
        let others = crate::rows::sources(&set, k, &t.rows, &self.me);
        crate::rows::front_of(&set, k, &counts, &others, &t.rows, row)
            .map(|f| {
                crate::front::badges::items(&f)
                    .into_iter()
                    .filter_map(|b| b.text)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// ⚠️ SONDA: o ícone «Create card from template» no pé da lista `list`.
    #[must_use]
    pub fn probe_template_button(&self, id: u64, list: ListId) -> Option<Rect> {
        self.w6_tab(id)?.layout.list(list)?.template
    }

    /// ⚠️ SONDA: o cartão do `Ctrl+C`.
    #[must_use]
    pub fn probe_clip(&self) -> Option<CardRef> {
        self.clip
    }
}
