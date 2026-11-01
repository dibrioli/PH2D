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
    /// O xadrez de transparência por baixo do quadro (o mesmo do canvas).
    pub(crate) xadrez: ph2d_pixel_render::Checker,
    /// O que o xadrez pintou na última vez: `(largura, altura, escala, x, y)` do sprite no ecrã.
    pub(crate) xadrez_pintado: Option<(u32, u32, f64, f64, f64)>,
}

impl PixelPreviewPanelState {
    /// O xadrez como foi pintado: `(w, h, escala, x, y)` do sprite no ecrã (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn xadrez(&self) -> Option<(u32, u32, f64, f64, f64)> {
        self.xadrez_pintado
    }

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

/// Pinta o xadrez por baixo do quadro `w × h` e regista onde (para os gates).
pub(crate) fn desenha_xadrez(
    xadrez: &mut ph2d_pixel_render::Checker,
    registo: &mut Option<(u32, u32, f64, f64, f64)>,
    scene: &mut ph2d_vector::VectorScene,
    (w, h): (u32, u32),
    to_screen: ph2d_vector::Affine,
) {
    let [sx, _, _, _, tx, ty] = to_screen.as_coeffs();
    *registo = Some((w, h, sx, tx, ty));
    xadrez.draw_under(scene, w, h, to_screen);
}
