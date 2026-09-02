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
    /// ⚠️ **É um MAPA `id → índice`, e não um conjunto.** Quem sabe o índice de um cartão é quem o
    /// pintou; a 1.ª versão publicava só os ids e o despachante re-derivava o índice varrendo 512
    /// hashes **por cada clique do rato em qualquer sítio do app**.
    pub fn set_asset_cells(&mut self, cells: std::collections::BTreeMap<NodeId, usize>) {
        self.asset_cells = cells;
    }

    /// `id` é um cartão do navegador de assets neste quadro?
    #[must_use]
    pub fn is_asset_cell(&self, id: NodeId) -> bool {
        self.asset_cells.contains_key(&id)
    }

    /// O índice do cartão `id` na grade — `None` se ele não é um cartão pintado agora.
    #[must_use]
    pub fn asset_cell_index(&self, id: NodeId) -> Option<usize> {
        self.asset_cells.get(&id).copied()
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

    /// ⭐⭐ **Escreve o veredito** — *o que aconteceria se a mão largasse agora* (wave B4).
    ///
    /// ⚠️ **Quem o calcula é o shell**, que é quem conhece a lei da queda (`asset_drop::resolve`) e
    /// o alvo debaixo do cursor. Esta camada só o guarda, como já guarda o cursor: um `interaction`
    /// que soubesse decidir quedas de asset passaria a conhecer o modelo de assets.
    pub fn set_asset_drag_verdict(
        &mut self,
        verdict: crate::interaction::drag_payload::DragVerdict,
    ) {
        if let Some(d) = self.asset_drag.as_mut() {
            d.verdict = verdict;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::drag_payload::ASSET_DRAG_THRESHOLD_PX;
    use ph2d_a11y::NodeId;

    fn store() -> WidgetStore {
        WidgetStore::default()
    }

    /// ⭐ **O ciclo inteiro**: semear, andar, armar, terminar.
    #[test]
    fn the_drag_arms_after_the_threshold_and_ends_once() {
        let mut s = store();
        assert!(s.asset_drag().is_none(), "nasce sem arrasto");
        s.begin_asset_drag(DragPayload::Prefab { stable_id: 3 }, 10.0, 10.0);
        assert!(
            !s.asset_drag().expect("semeado").armed,
            "ainda e' um clique"
        );
        s.update_asset_drag(10.0 + ASSET_DRAG_THRESHOLD_PX + 1.0, 10.0);
        let d = s.asset_drag().expect("em curso");
        assert!(d.armed, "passou o limiar e nao armou");
        assert_eq!(d.payload, DragPayload::Prefab { stable_id: 3 });
        let ended = s.end_asset_drag().expect("terminar devolve o arrasto");
        assert!(ended.armed);
        assert!(
            s.end_asset_drag().is_none(),
            "terminar duas vezes nao pode devolver duas quedas"
        );
    }

    /// ⛔ **Um gesto parado termina NÃO ARMADO** — e o chamador tem de o distinguir de *«não havia
    /// arrasto»*, senão o clique do cartão desaparece.
    #[test]
    fn a_still_gesture_ends_unarmed_and_is_not_the_same_as_no_drag() {
        let mut s = store();
        s.begin_asset_drag(DragPayload::Image { asset: [1; 32] }, 5.0, 5.0);
        s.update_asset_drag(5.0, 5.0);
        let ended = s.end_asset_drag().expect("houve um gesto");
        assert!(!ended.armed, "parado nao arma");
    }

    /// ⚠️ **Andar sem ter semeado é um no-op** — um `Move` sem `Down` não pode inventar um arrasto.
    #[test]
    fn moving_without_a_down_invents_nothing() {
        let mut s = store();
        s.update_asset_drag(100.0, 100.0);
        assert!(s.asset_drag().is_none());
    }

    /// ⭐ Os cartões são publicados por quadro, e **esvaziar a lista fecha a porta**: um `Down` no
    /// sítio onde o painel ESTAVA não pode arrancar um arrasto.
    #[test]
    fn the_cell_census_is_republished_and_closes_when_the_panel_hides() {
        let mut s = store();
        let a = NodeId(4242);
        assert!(
            !s.is_asset_cell(a),
            "nada e' cartao antes de alguem o dizer"
        );
        s.set_asset_cells([(a, 7)].into_iter().collect());
        assert!(s.is_asset_cell(a));
        assert_eq!(
            s.asset_cell_index(a),
            Some(7),
            "o indice tem de vir do painel"
        );
        s.set_asset_cells(std::collections::BTreeMap::new());
        assert!(
            !s.is_asset_cell(a),
            "o painel fechou e o cartao continua a existir"
        );
    }
}
