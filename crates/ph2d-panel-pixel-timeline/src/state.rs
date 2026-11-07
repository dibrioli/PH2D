//! O estado do painel: o da VISTA (os renomear abertos, a primeira coluna à vista, o menu da
//! direcção aberto); o do DOCUMENTO chega por thread-local, publicado pela shell antes de pintar.

use ph2d_editor_core::pixel_panel::{PixelDirection, PixelPanelInfo, PixelRange};
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
    /// O menu da velocidade aberto: o chip e a velocidade de agora.
    pub(crate) speed_open: Option<(Rect, ph2d_editor_core::pixel_panel::PixelSpeed)>,
    /// *Constant Frame Rate* aberto: o activo e o intervalo de quando abriu (mudar um fecha-o).
    pub(crate) constante: Option<(Option<u32>, PixelRange)>,
    /// O arrasto em curso na grelha ([`crate::gestos`]).
    pub(crate) arrasto: Option<crate::gestos::Arrasto>,
    /// A geometria da grelha do último quadro pintado (as posições dos gestos lêem-na).
    pub(crate) geo: Option<crate::gestos::Geo>,
    /// A barra que marca onde o bloco arrastado cai (à esquerda da primeira coluna do vão).
    pub(crate) marca: Option<Rect>,
}

impl PixelTimelinePanelState {
    /// A barra da queda do arrasto em curso (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn drop_marker(&self) -> Option<Rect> {
        self.marca
    }

    /// Quantas vezes a barra da queda foi PINTADA nesta thread (para os gates: `drop_marker` só
    /// diz onde ela ficaria, não que alguém a desenhou).
    #[doc(hidden)]
    #[must_use]
    pub fn marcas_pintadas(&self) -> u32 {
        MARCAS.with(std::cell::Cell::get)
    }

    /// Quantas marcas de key de slice foram PINTADAS nesta thread (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn marcas_de_slice_pintadas(&self) -> u32 {
        MARCAS_DE_SLICE.with(std::cell::Cell::get)
    }

    /// A primeira coluna à vista (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn first_col(&self) -> usize {
        self.first_col
    }
}

thread_local! {
    static MARCAS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static MARCAS_DE_SLICE: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
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

pub(crate) fn conta_marca() {
    MARCAS.with(|c| c.set(c.get() + 1));
}

pub(crate) fn conta_marca_de_slice() {
    MARCAS_DE_SLICE.with(|c| c.set(c.get() + 1));
}

pub(crate) fn with_current<R>(f: impl FnOnce(&PixelPanelInfo) -> R) -> R {
    CURRENT.with(|c| f(&c.borrow()))
}
