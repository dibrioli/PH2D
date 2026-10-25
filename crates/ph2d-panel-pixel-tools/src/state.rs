//! O estado do painel PIXEL: nenhum gesto guarda estado próprio (a `State` tipada é vazia); o do
//! DOCUMENTO chega por thread-local, publicado pela shell antes de o painel pintar.

use ph2d_editor_core::pixel_panel::PixelPanelInfo;

#[derive(Clone, Debug, Default)]
pub struct PixelToolsPanelState {
    /// O renomear aberto: `(camada, o campo já foi semeado)`.
    pub(crate) rename: Option<(u32, bool)>,
}

thread_local! {
    static CURRENT: std::cell::RefCell<PixelPanelInfo> =
        std::cell::RefCell::new(PixelPanelInfo::default());
}

/// **A porta da shell** — uma vez por quadro.
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
