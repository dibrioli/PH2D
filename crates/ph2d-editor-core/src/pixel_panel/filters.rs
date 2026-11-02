//! O vocabulário dos filtros do Pixel (P8, `docs/PixelArt/P8_plano.md` §4): Edit ▸ Invert e
//! Edit ▸ FX ▸ Despeckle/Outline, na janelinha com Preview · OK · Cancel. A família
//! (`ph2d-app-pixel`) traduz para `ph2d_pixel_core::filter`.

/// O *Target* do diálogo: os canais que o filtro muda `[R, G, B, A]`.
pub type PixelChannels = [bool; 4];

/// Outline: o contorno cresce para fora da forma ou come a borda de dentro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelOutlinePlace {
    #[default]
    Outside,
    Inside,
}

/// Outline: os quatro presets da matriz de vizinhos do Aseprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelOutlineMatrix {
    #[default]
    Circle,
    Square,
    Horizontal,
    Vertical,
}

/// Outline: o que conta como fundo — o transparente ou a cor de fundo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelOutlineBg {
    #[default]
    Transparent,
    BackgroundColor,
}

/// As escolhas de um diálogo de filtro que não são campos numéricos (lembradas na sessão).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelFilterDialog {
    /// O canvas mostra o resultado enquanto o diálogo está aberto (omissão: ligado).
    pub preview: bool,
    pub channels: PixelChannels,
    /// Despeckle e Outline: a vizinhança dá a volta à imagem.
    pub tiled: bool,
    pub place: PixelOutlinePlace,
    pub matrix: PixelOutlineMatrix,
    pub background: PixelOutlineBg,
    /// Despeckle: a última janela da sessão.
    pub median: (u32, u32),
}

impl Default for PixelFilterDialog {
    /// Os do Aseprite ao abrir pela 1.ª vez: Preview ligado, os quatro canais, 3×3, sem Tiled.
    fn default() -> Self {
        Self {
            preview: true,
            channels: [true; 4],
            tiled: false,
            place: PixelOutlinePlace::default(),
            matrix: PixelOutlineMatrix::default(),
            background: PixelOutlineBg::default(),
            median: (3, 3),
        }
    }
}
