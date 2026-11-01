//! O estado da VISTA da Preview: onde está, o zoom e a imagem da GPU da última publicação.

use ph2d_editor_core::zones::Rect;

#[derive(Clone, Default)]
pub struct PixelPreviewPanelState {
    /// O rect da janela (nasce no primeiro quadro visível; o arrasto soma-lhe o deslocamento).
    pub(crate) rect: Option<Rect>,
    /// O degrau de zoom ([`crate::paint::ZOOMS`]); `None` = caber na janela.
    pub(crate) zoom: Option<usize>,
    /// A imagem da GPU e o `seq` da publicação de que veio.
    pub(crate) imagem: Option<(u64, ph2d_vector::StableImage)>,
    /// A escala da última pintura (o zoom da roda parte dela).
    pub(crate) escala: f32,
    /// Os cantos `(x0, y0, x1, y1)` passados ao `draw_stable_image` na última pintura.
    pub(crate) destino: Option<(f64, f64, f64, f64)>,
}

impl PixelPreviewPanelState {
    /// O rect da janela como foi pintado (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn rect(&self) -> Option<Rect> {
        self.rect
    }

    /// Píxeis de ecrã por célula na última pintura (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn escala(&self) -> f32 {
        self.escala
    }

    /// Os cantos da imagem como foram passados ao `draw_stable_image` (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn destino(&self) -> Option<(f64, f64, f64, f64)> {
        self.destino
    }
}
