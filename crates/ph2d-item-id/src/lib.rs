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

    #[must_use]
    pub const fn is_none(self) -> bool {
        self.0 == 0
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
