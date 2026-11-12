//! ⭐ **O destaque da paleta** no modo «destaca, depois escolhe» (`PaletteModel::select`, Add Object, dono
//! 07/11) — filho do `chrome_ops` pelo tecto de LOC.

use super::super::WidgetStore;
use crate::widget::command_palette::{item_ids, scoped};
use ph2d_a11y::NodeId;

impl WidgetStore {
    /// A paleta aberta destaca antes de escolher?
    #[must_use]
    pub fn command_palette_selects(&self) -> bool {
        self.command_palette
            .as_ref()
            .is_some_and(|m| m.select.is_some())
    }

    /// ⭐ **O item destacado que se VÊ** — o escolhido, se a busca o deixa à vista; senão o melhor da busca;
    /// senão o primeiro visível. `None` fora do modo de seleção ou sem nada à vista.
    #[must_use]
    pub fn command_palette_highlight(&self) -> Option<NodeId> {
        let model = self
            .command_palette
            .as_ref()
            .filter(|m| m.select.is_some())?;
        let view = scoped(
            model,
            &self.command_palette_query,
            self.command_palette_scopes(),
        );
        let ids = item_ids(&view.model);
        // Em lista o destaque por omissão é a 1.ª linha que se pinta, não o melhor da busca.
        let top = view.top.filter(|_| !model.is_list());
        self.command_palette_scope
            .highlight
            .filter(|id| ids.contains(id))
            .or(top)
            .or_else(|| ids.first().copied())
    }

    /// ⭐ A busca de uma paleta em LISTA mudou: o destaque antigo sai no mesmo instante (o Enter do
    /// mesmo quadro corre a 1.ª linha filtrada, não a que estava destacada).
    pub(super) fn drop_list_highlight(&mut self) {
        if self.command_palette.as_ref().is_some_and(|m| m.is_list()) {
            self.command_palette_scope.highlight = None;
        }
    }

    /// Destaca `id` (um clique num item). O clique sai da busca, como num campo de verdade.
    pub fn highlight_command_palette(&mut self, id: NodeId) {
        if self.command_palette_selects() {
            self.command_palette_scope.highlight = Some(id);
            self.command_palette_scope.search_focus = false;
        }
    }

    /// ⭐ **A busca tem o foco?** Só depois de um clique nela ou de uma tecla (dono 08/11: «deveria ficar
    /// destacado se o usuário clicar nele ou começar a digitar»); ao abrir, não.
    #[must_use]
    pub fn command_palette_search_focused(&self) -> bool {
        self.command_palette.is_some() && self.command_palette_scope.search_focus
    }

    /// Um clique na caixa da busca.
    pub fn focus_command_palette_search(&mut self) {
        if self.command_palette.is_some() {
            self.command_palette_scope.search_focus = true;
        }
    }

    /// ↑ ↓: o destaque anda `delta` itens pelos visíveis, preso às pontas.
    pub fn step_command_palette_highlight(&mut self, delta: isize) {
        let Some(model) = self.command_palette.as_ref().filter(|m| m.select.is_some()) else {
            return;
        };
        let view = scoped(
            model,
            &self.command_palette_query,
            self.command_palette_scopes(),
        );
        let ids = item_ids(&view.model);
        let Some(last) = ids.len().checked_sub(1) else {
            return;
        };
        let at = self
            .command_palette_highlight()
            .and_then(|h| ids.iter().position(|id| *id == h))
            .unwrap_or(0);
        let next = at.saturating_add_signed(delta).min(last);
        self.command_palette_scope.highlight = Some(ids[next]);
    }

    /// O botão do ensino: limpa a busca e destaca a peça por onde se começa.
    pub fn restart_command_palette_highlight(&mut self) {
        let Some(model) = self.command_palette.as_ref() else {
            return;
        };
        let Some(t) = crate::widget::command_palette::teach_for(model, &self.command_palette_query)
        else {
            return;
        };
        self.command_palette_scope.highlight = Some(t.start);
        self.command_palette_query.clear();
    }

    /// ⭐ **O alvo do `Enter`:** no modo de seleção, o destacado; senão o melhor da busca.
    #[must_use]
    pub fn command_palette_enter_target(&self) -> Option<NodeId> {
        if self.command_palette_selects() {
            self.command_palette_highlight()
        } else {
            self.command_palette_top()
        }
    }
}

impl WidgetStore {
    /// Quantos tipos o *Recent* do Add Object guarda (as três linhas da maquete B).
    pub const ADD_RECENT_MAX: usize = 3;

    /// ⭐ **Os tipos que o Add Object criou por último**, o mais novo primeiro (até
    /// [`Self::ADD_RECENT_MAX`]). Gravam-se com as preferências das secções (`~/.ph2d/sections.txt`).
    #[must_use]
    pub fn add_recent(&self) -> &[NodeId] {
        &self.section_prefs.add_recent
    }

    /// O Add Object criou `id`: passa para o topo do *Recent*, sem repetir.
    pub fn push_add_recent(&mut self, id: NodeId) {
        let list = &mut self.section_prefs.add_recent;
        list.retain(|x| *x != id);
        list.insert(0, id);
        list.truncate(Self::ADD_RECENT_MAX);
        self.mark_section_prefs_dirty();
    }
}

/// A linha `add_recent=` do ficheiro de preferências.
pub(in crate::interaction::state) fn escreve_recentes(store: &WidgetStore, s: &mut String) {
    use std::fmt::Write as _;
    let list = store.add_recent();
    if !list.is_empty() {
        let ids: Vec<String> = list.iter().map(|n| n.0.to_string()).collect();
        let _ = writeln!(s, "add_recent={}", ids.join(","));
    }
}

/// Lê a linha de [`escreve_recentes`]; `false` se `k` não é dela.
pub(in crate::interaction::state) fn le_recentes(
    store: &mut WidgetStore,
    k: &str,
    v: &str,
) -> bool {
    if k != "add_recent" {
        return false;
    }
    let mut list: Vec<NodeId> = Vec::new();
    for id in v
        .split(',')
        .filter_map(|t| t.trim().parse().ok())
        .map(NodeId)
    {
        if !list.contains(&id) {
            list.push(id);
        }
    }
    list.truncate(WidgetStore::ADD_RECENT_MAX);
    store.section_prefs.add_recent = list;
    true
}
