//! ⭐ **O estado de VISTA das linhas novas da Hierarquia** ([`crate::hier_rows`]) — os grupos de
//! conteúdo abertos e o pedido de «mostra este cartão» que o Inspector atende.

use super::*;

/// Os dois factos, num campo só do [`WidgetStore`].
#[derive(Clone, Debug, Default)]
pub struct HierRowsState {
    /// Grupos de conteúdo ABERTOS. ⚠️ Conjunto dos abertos, e não dos fechados como o
    /// `hierarchy_collapsed`: o conteúdo nasce recolhido (doc 01 §6).
    content_open: std::collections::BTreeSet<NodeId>,
    reveal: Option<InspectorReveal>,
}

/// ⭐ **«Mostra o cartão desta família»** — o clique num ícone de componente da linha. O Inspector
/// atende-o no primeiro quadro em que pinta a entidade `entity` (`Entity::to_bits`).
///
/// ⚠️ Pela IDENTIDADE, nunca pelo nome: com dois objectos do mesmo nome e um deles já no
/// Inspector, o pedido do outro casava no quadro de ANTES da troca e o cartão rolava sobre o
/// objecto errado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InspectorReveal {
    pub category: ph2d_component_desc::ComponentCategory,
    pub entity: u64,
}

impl WidgetStore {
    /// O grupo de conteúdo `id` está aberto?
    #[must_use]
    pub fn is_hierarchy_content_open(&self, id: NodeId) -> bool {
        self.hier_rows.content_open.contains(&id)
    }

    /// Abre/fecha o grupo de conteúdo `id` (o galo dele).
    pub fn toggle_hierarchy_content_open(&mut self, id: NodeId) {
        if !self.hier_rows.content_open.insert(id) {
            self.hier_rows.content_open.remove(&id);
        }
    }

    /// Pede ao Inspector que traga à vista o primeiro cartão de `category` da entidade `entity`.
    pub fn request_inspector_reveal(
        &mut self,
        category: ph2d_component_desc::ComponentCategory,
        entity: u64,
    ) {
        self.hier_rows.reveal = Some(InspectorReveal { category, entity });
    }

    /// O pedido em curso (o Inspector lê-o ao pintar).
    #[must_use]
    pub fn inspector_reveal(&self) -> Option<&InspectorReveal> {
        self.hier_rows.reveal.as_ref()
    }

    /// Atende o pedido: tira-o do store.
    pub fn take_inspector_reveal(&mut self) -> Option<InspectorReveal> {
        self.hier_rows.reveal.take()
    }
}
