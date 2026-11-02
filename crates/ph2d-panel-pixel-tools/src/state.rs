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
    pub(crate) paleta: PaletaVista,
    pub(crate) pincel: PincelVista,
}

/// A vista do pincel: a secção Dynamics aberta, as trilhas de duas pegas pintadas neste quadro
/// (pressão, velocidade) e a pega agarrada `(trilha, pega)`.
#[derive(Clone, Debug, Default)]
pub(crate) struct PincelVista {
    pub(crate) dinamica: bool,
    pub(crate) trilhas: [Option<Rect>; 2],
    pub(crate) pega: Option<(usize, ph2d_editor_core::widget::RangeHandle)>,
}

/// A paleta como se pintou neste quadro: a ordem (prevista, a meio de um arrasto) e o bloco em voo.
#[derive(Clone, Debug, Default)]
pub(crate) struct PaletaVista {
    /// Célula `j` → a amostra que lá se pintou; `None` = o vão onde o bloco cai.
    pub(crate) pintada: Vec<Option<usize>>,
    pub(crate) voo: Option<VooPaleta>,
    /// A barra que marca onde o bloco cai (à esquerda da primeira célula do vão).
    pub(crate) marca: Option<Rect>,
    /// Onde o arrasto vivo largaria — a MESMA conta que se pintou; a queda lê-a (`seam::larga`).
    pub(crate) queda: Option<crate::paint::Queda>,
}

/// O bloco de amostras agarrado: a caixa (antes da inclinação), as cores e quantas por linha.
#[derive(Clone, Debug)]
pub(crate) struct VooPaleta {
    pub(crate) caixa: Rect,
    pub(crate) cores: Vec<[u8; 4]>,
    pub(crate) por_linha: usize,
}

impl PixelToolsPanelState {
    /// A ordem que a grelha da paleta pintou (`None` = o vão) — para os gates.
    #[doc(hidden)]
    #[must_use]
    pub fn palette_painted(&self) -> &[Option<usize>] {
        &self.paleta.pintada
    }

    /// A caixa do bloco de amostras em voo, se há arrasto — para os gates.
    #[doc(hidden)]
    #[must_use]
    pub fn palette_flight(&self) -> Option<Rect> {
        self.paleta.voo.as_ref().map(|v| v.caixa)
    }

    /// A barra que marca onde o bloco de amostras cai, se há arrasto — para os gates.
    #[doc(hidden)]
    #[must_use]
    pub fn palette_drop_marker(&self) -> Option<Rect> {
        self.paleta.marca
    }

    /// A secção Dynamics está aberta — para os gates.
    #[doc(hidden)]
    #[must_use]
    pub fn dynamics_open(&self) -> bool {
        self.pincel.dinamica
    }

    /// Os parâmetros do seletor de cor — de onde o anel se desenha (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn selector_params(&self) -> crate::seletor::Params {
        self.seletor.params
    }
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
    /// As últimas cores que o próprio seletor mandou: o instantâneo chega um quadro atrasado, e
    /// reconstruir os parâmetros de uma delas fazia o anel tremer (smoke do dono 25/10).
    pub(crate) mandadas: Vec<[u8; 4]>,
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
