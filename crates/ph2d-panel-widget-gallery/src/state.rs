//! Widget Gallery panel state — owned by `ErasedPanel<WidgetGalleryPanel>` (ADR-0029 Phase C.3).
//!
//! - `rect` — a janela flutuante de hoje (`PH2D_CARDS=0`), criada no primeiro quadro.
//! - `choices` — o que os cartões do vocabulário mostram e que NÃO é número do store (o segmento
//!   aceso, a caixa de marcar, a lista suspensa, a secção presente). O paint lê; o evento escreve.

use ph2d_editor_core::zones::Rect;

/// As escolhas vivas dos cartões do vocabulário. `Default` = os valores de fábrica (o ↺).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choices {
    /// Low · Mid · High (0..=2).
    pub level: u8,
    /// Butt · Round · Square (0..=2).
    pub cap: u8,
    pub visible: bool,
    pub locked: bool,
    pub snap: bool,
    /// Normal · Multiply · Screen (0..=2).
    pub blend: u8,
    pub fill_visible: bool,
    /// A secção Physics existe (o «+» do cartão ausente cria-a).
    pub physics: bool,
}

impl Default for Choices {
    fn default() -> Self {
        Self {
            level: 1,
            cap: 0,
            visible: true,
            locked: false,
            snap: false,
            blend: 0,
            fill_visible: true,
            physics: false,
        }
    }
}

/// Widget Gallery retained state.
#[derive(Clone, Debug, Default)]
pub struct WidgetGalleryState {
    /// Rect of the floating gallery (`PH2D_CARDS=0`), lazily set on first paint.
    pub rect: Option<Rect>,
    pub choices: Choices,
}
