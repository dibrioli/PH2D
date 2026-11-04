//! ⭐ **A identidade de um ITEM de uma lista de componente** (F30 §A1) — um timer, um tween, um
//! estado, uma seta, uma regra, uma acção, uma fonte de abanão, uma resistência, uma âncora.
//!
//! A chave de animação de um campo de item é `(lista, campo, ItemId)`, nunca a posição: apagar ou
//! reordenar um item não desloca a chave de outro. O id sai do `StableIdCounter` do projecto
//! (`ph2d_ecs::item_id`), que é monotónico e fora do undo: um id nunca se reusa, logo uma chave
//! órfã nunca acorda noutro item. Desenho: `docs/UI_New_and_Simple/F30_plano.md` §A1.

use serde::{Deserialize, Serialize};

/// O id durável de um item. `0` = por atribuir (o passe de atribuição dá-lho no quadro).
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct ItemId(pub u32);

impl ItemId {
    /// Por atribuir.
    pub const NONE: ItemId = ItemId(0);

    /// ⭐ **A tinta de BASE, como endereço de item** (estilos de cor, CartoesUI D30): o preenchimento
    /// de um caminho não é um item da pilha, mas liga-se a um estilo como um. Só vale ao lado do
    /// caminho, em `(path, ItemId)`; nenhuma `PaintEntry` o tem e o alocador nunca o entrega
    /// (`StableIdCounter::aloca_item`, gate `o_alocador_nunca_entrega_um_id_reservado`).
    pub const BASE_FILL: ItemId = ItemId(u32::MAX);
    /// O traço de base (ver [`Self::BASE_FILL`]).
    pub const BASE_STROKE: ItemId = ItemId(u32::MAX - 1);

    #[must_use]
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }

    /// `true` ⇒ uma das tintas de base: um id que o alocador nunca entrega.
    #[must_use]
    pub const fn reservado(self) -> bool {
        self.0 >= Self::BASE_STROKE.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_id_grava_se_como_o_numero() {
        let b = postcard::to_allocvec(&ItemId(300)).expect("grava");
        assert_eq!(b, postcard::to_allocvec(&300u32).expect("grava"));
        assert!(ItemId::default().is_none());
    }
}
