//! ⭐ **As folhas de um Animated Sprite como o cartão Image as mostra** (plano GEF 26 §3.3). O shell publica-as por
//! quadro, como a [`crate::slot_image`]; o painel não conhece o `SpriteSheets`.

use crate::slot_image::SlotImage;
use std::cell::RefCell;

/// Uma folha: a miniatura da imagem INTEIRA (com o nome), os píxeis dela e o corte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetFace {
    pub image: SlotImage,
    /// `(largura, altura)` da imagem em px.
    pub px: (u32, u32),
    /// `[colunas, linhas]`.
    pub grid: [u32; 2],
}

/// As folhas do objeto seleccionado em Animated: `sheets[0]` é a folha `0`, nunca vazio.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetCard {
    pub entity_bits: u64,
    /// A folha visível no painel (`ph2d_ecs::shown_sheet`), `< sheets.len()`.
    pub shown: u8,
    pub sheets: Vec<SheetFace>,
}

impl SheetCard {
    /// A folha visível.
    #[must_use]
    pub fn visible(&self) -> Option<&SheetFace> {
        self.sheets.get(usize::from(self.shown))
    }
}

thread_local! {
    static CURRENT: RefCell<Option<SheetCard>> = const { RefCell::new(None) };
}

/// O shell publica as folhas da seleção (`None` = não é um Animated com imagem).
pub fn set_current_sheets(card: Option<SheetCard>) {
    CURRENT.with(|c| *c.borrow_mut() = card);
}

/// As folhas publicadas para `entity_bits`.
#[must_use]
pub fn current_sheets(entity_bits: u64) -> Option<SheetCard> {
    CURRENT.with(|c| {
        c.borrow()
            .as_ref()
            .filter(|s| s.entity_bits == entity_bits)
            .cloned()
    })
}
