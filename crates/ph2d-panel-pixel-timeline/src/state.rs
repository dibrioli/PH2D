//! O estado do painel: o da VISTA (os renomear abertos, a primeira coluna à vista, o menu da
//! direcção aberto); o do DOCUMENTO chega por thread-local, publicado pela shell antes de pintar.

use ph2d_editor_core::pixel_panel::{PixelDirection, PixelPanelInfo};
use ph2d_editor_core::zones::Rect;

#[derive(Clone, Debug, Default)]
pub struct PixelTimelinePanelState {
    /// O renomear de camada aberto: `(camada, o campo já foi semeado)`.
    pub(crate) rename_layer: Option<(u32, bool)>,
    /// O renomear de tag aberto: `(índice da tag, semeado)`.
    pub(crate) rename_tag: Option<(usize, bool)>,
    /// A primeira coluna à vista (a grelha segue o frame activo).
    pub(crate) first_col: usize,
    /// O menu da direcção aberto: o chip e a direcção actual (pinta-se por cima, no fim).
    pub(crate) direction_open: Option<(Rect, PixelDirection)>,
}

impl PixelTimelinePanelState {
    /// A primeira coluna à vista (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn first_col(&self) -> usize {
        self.first_col
    }
}

thread_local! {
    static CURRENT: std::cell::RefCell<PixelPanelInfo> =
        std::cell::RefCell::new(PixelPanelInfo::default());
}

/// **A porta da shell** — uma vez por quadro (o mesmo instantâneo do painel PIXEL).
pub fn set_current_pixel(info: PixelPanelInfo) {
    CURRENT.with(|c| *c.borrow_mut() = info);
}

/// O instantâneo como o painel o leu.
#[must_use]
pub fn current_pixel() -> PixelPanelInfo {
    CURRENT.with(|c| c.borrow().clone())
}

pub(crate) fn with_current<R>(f: impl FnOnce(&PixelPanelInfo) -> R) -> R {
    CURRENT.with(|c| f(&c.borrow()))
}
