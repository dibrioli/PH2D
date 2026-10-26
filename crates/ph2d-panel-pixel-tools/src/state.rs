//! O estado do painel PIXEL: o da VISTA (o renomear aberto, o seletor de cor); o do DOCUMENTO
//! chega por thread-local, publicado pela shell antes de o painel pintar.

use std::sync::Arc;

use ph2d_editor_core::pixel_panel::{PixelPanelInfo, PixelSelector};
use ph2d_editor_core::zones::Rect;

#[derive(Clone, Debug, Default)]
pub struct PixelToolsPanelState {
    /// O renomear aberto: `(camada, o campo já foi semeado)`.
    pub(crate) rename: Option<(u32, bool)>,
    pub(crate) seletor: Seletor,
}

/// A vista do seletor de cor: os parâmetros do modo (o que a cor não diz — a matiz de um cinza —
/// fica aqui), a cor de frente a que eles correspondem, as imagens em cache e onde se pintaram.
#[derive(Clone, Debug, Default)]
pub(crate) struct Seletor {
    pub(crate) params: crate::seletor::Params,
    /// `(modo, cor)` dos parâmetros: outra cor de frente ou outro modo reconstrói-os.
    pub(crate) de: Option<(PixelSelector, [u8; 4])>,
    /// As imagens (quadro, barra, alfa) e os parâmetros com que se geraram.
    pub(crate) cache: Option<(PixelSelector, crate::seletor::Params, [u8; 4], Imagens)>,
    /// Os rects pintados neste quadro: quadro, barra, alfa.
    pub(crate) rects: [Option<Rect>; 3],
    /// O último ponto do arrasto em curso (premido sobre o seletor): a cor segue o ponteiro.
    pub(crate) arrasto: Option<[f32; 2]>,
}

#[derive(Clone, Debug)]
pub(crate) struct Imagens {
    pub(crate) quadro: Arc<Vec<u8>>,
    pub(crate) barra: Arc<Vec<u8>>,
    pub(crate) alfa: Arc<Vec<u8>>,
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
