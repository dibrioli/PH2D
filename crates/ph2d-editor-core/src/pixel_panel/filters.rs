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

/// Hue/Saturation: os quatro modelos do Aseprite (HSL por omissão; os «+» somam).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelHueMode {
    #[default]
    Hsl,
    Hsv,
    HslPlus,
    HsvPlus,
}

impl PixelHueMode {
    /// Os modelos HSV chamam «Value» à terceira barra.
    #[must_use]
    pub const fn is_hsv(self) -> bool {
        matches!(self, Self::Hsv | Self::HsvPlus)
    }
}

/// Quantos pontos interiores a curva guarda: os que a caixa do diálogo distingue ao agarrar
/// (`widget::curve_capacity` da caixa de `DIALOG_W`, ≤ este número — o gate do diálogo confere).
pub const PIXEL_CURVE_MAX: usize = 32;

/// Color Curve: os pontos INTERIORES por `x` crescente (as pontas `(0,0)`/`(255,255)` são fixas).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelCurve {
    n: u8,
    pts: [(u8, u8); PIXEL_CURVE_MAX],
}

impl Default for PixelCurve {
    fn default() -> Self {
        Self {
            n: 0,
            pts: [(0, 0); PIXEL_CURVE_MAX],
        }
    }
}

impl PixelCurve {
    #[must_use]
    pub fn points(&self) -> &[(u8, u8)] {
        &self.pts[..usize::from(self.n)]
    }

    /// Os primeiros [`PIXEL_CURVE_MAX`] pontos de `pts` (já por `x` crescente).
    #[must_use]
    pub fn from_points(pts: &[(u8, u8)]) -> Self {
        let mut c = Self::default();
        for (d, s) in c.pts.iter_mut().zip(pts) {
            *d = *s;
        }
        c.n = u8::try_from(pts.len().min(PIXEL_CURVE_MAX)).unwrap_or(0);
        c
    }
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
    /// Adjustments: `[brilho, contraste, matiz, saturação, luz, alfa]` da última vez (OK).
    pub adjust: [i32; 6],
    pub hue_mode: PixelHueMode,
    pub curve: PixelCurve,
    /// O ponto da curva agarrado (premido sobre a caixa).
    pub curve_grab: Option<usize>,
    /// Convolution Matrix: o kernel escolhido (índice em [`Self::kernels`]), lembrado na sessão.
    pub kernel: u8,
    /// Convolution Matrix: os nomes dos kernels, na ordem do núcleo — a família publica-os
    /// ([`crate::interaction::WidgetStore::set_pixel_filter_kernels`]); o chrome não vê o núcleo.
    pub kernels: &'static [&'static str],
    /// O alvo «All»: todos os quadros × todas as camadas que se pintam (desligado = «Selected», o
    /// cel activo). Lembrado na sessão.
    pub all: bool,
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
            adjust: [0; 6],
            hue_mode: PixelHueMode::default(),
            curve: PixelCurve::default(),
            curve_grab: None,
            kernel: 0,
            kernels: &[],
            all: false,
        }
    }
}
