//! ⭐⭐ **O arrasto que SAI do painel** — o estado dele, no `WidgetStore` (plano
//! `docs/Components/07`, etapa B).
//!
//! # Por que ele mora aqui, e não no painel
//!
//! Porque **o alvo não é um painel**. Todo arrasto deste editor guarda o estado no store e resolve
//! o alvo no próprio painel que o começou (reparentar, barra, redimensionar). Este começa no
//! navegador de assets e acaba **na tela** — que nenhum painel possui. ⇒ o estado tem de estar num
//! sítio que o shell alcança, e o `WidgetStore` já é esse sítio para todos os outros.
//!
//! ⛔ **E o fim dele NÃO é drenado pelo `dispatch_up`**, ao contrário dos irmãos. O `pointer_up`
//! do editor-core fecha os arrastos cujo alvo ele conhece; este só se resolve com a câmara e o
//! pick, que vivem no shell. Se o dispatch o fechasse, a resposta chegaria ao shell já perdida.

use super::WidgetStore;
use crate::interaction::drag_payload::{DragPayload, InFlightDrag};
use ph2d_a11y::NodeId;

impl WidgetStore {
    /// **Os cartões que o navegador de assets está a mostrar agora.**
    ///
    /// ⚠️ Republicado por quadro pelo painter do painel — é o mesmo idioma do
    /// [`WidgetStore::set_hierarchy_row_ids`], e pela mesma razão: quantos cartões existem só se
    /// sabe em runtime, e o `pointer_down` precisa de responder *«este id é um cartão?»* sem
    /// conhecer o painel.
    pub fn set_asset_cell_ids(&mut self, ids: std::collections::BTreeSet<NodeId>) {
        self.asset_cell_ids = ids;
    }

    /// `id` é um cartão do navegador de assets neste quadro?
    #[must_use]
    pub fn is_asset_cell(&self, id: NodeId) -> bool {
        self.asset_cell_ids.contains(&id)
    }

    /// Começa um arrasto de asset — **ainda não armado**, porque isto ainda pode ser um clique.
    pub fn begin_asset_drag(&mut self, payload: DragPayload, x: f32, y: f32) {
        self.asset_drag_origin = (x, y);
        self.asset_drag = Some(InFlightDrag::started(payload, (x, y)));
    }

    /// Move o cursor do arrasto, armando-o se passou o limiar.
    pub fn update_asset_drag(&mut self, x: f32, y: f32) {
        let origin = self.asset_drag_origin;
        if let Some(d) = self.asset_drag.as_mut() {
            d.moved(origin, (x, y));
        }
    }

    /// O arrasto em curso, para quem o pinta.
    #[must_use]
    pub fn asset_drag(&self) -> Option<InFlightDrag> {
        self.asset_drag
    }

    /// Termina o arrasto e devolve o que ele era.
    ///
    /// ⚠️ **Devolve mesmo quando não estava armado** — quem chama tem de saber distinguir *«foi um
    /// clique»* de *«não havia arrasto nenhum»*, e um `None` para os dois casos apagaria a
    /// diferença.
    pub fn end_asset_drag(&mut self) -> Option<InFlightDrag> {
        self.asset_drag.take()
    }
}
