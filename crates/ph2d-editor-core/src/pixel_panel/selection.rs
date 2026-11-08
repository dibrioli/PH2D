//! O vocabulário da SELECÇÃO e da TRANSFORMAÇÃO do Pixel (P6): o modo de juntar, os menus Select,
//! Edit e Sprite do Aseprite, a moldura solta e os diálogos (Color Range, Modify, Canvas Size,
//! Sprite Size). A família (`ph2d-app-pixel`) traduz para o núcleo.

/// Como uma selecção nova se junta à que há (os quatro botões da barra do Aseprite; no rato:
/// Shift = somar, Alt+Shift = tirar, Ctrl+Shift = cruzar).
use crate::zones::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelSelectMode {
    #[default]
    Replace,
    Add,
    Subtract,
    Intersect,
}

impl PixelSelectMode {
    pub const ALL: [Self; 4] = [Self::Replace, Self::Add, Self::Subtract, Self::Intersect];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Replace => "panel.pixel.select.replace",
            Self::Add => "panel.pixel.select.add",
            Self::Subtract => "panel.pixel.select.subtract",
            Self::Intersect => "panel.pixel.select.intersect",
        }
    }
}

/// A amostragem de uma rotação/escala solta (a do combo da barra do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelResample {
    #[default]
    Fast,
    RotSprite,
}

impl PixelResample {
    pub const ALL: [Self; 2] = [Self::Fast, Self::RotSprite];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Fast => "panel.pixel.resample.fast",
            Self::RotSprite => "panel.pixel.resample.rotsprite",
        }
    }
}

/// O que Select ▸ Modify faz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelModify {
    Expand,
    Contract,
    Border,
}

/// O pincel de Select ▸ Modify.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelModifyBrush {
    #[default]
    Square,
    Circle,
}

/// O menu Select.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelSelectAction {
    /// `Ctrl+A`.
    All,
    /// `Esc` · `Ctrl+D`.
    Deselect,
    /// `Ctrl+Shift+D`.
    Reselect,
    /// `Ctrl+Shift+I`.
    Inverse,
    /// Abre o diálogo Color Range (a cor é a de frente).
    ColorRange,
    /// Abre o diálogo Modify ▸ Expand/Contract/Border.
    Modify(PixelModify),
    /// *Load from MSK file* / *Save to MSK file* (o `.msk` do Aseprite).
    LoadMsk,
    SaveMsk,
}

/// O menu Edit (o que mexe nas células seleccionadas) e a moldura solta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelEditAction {
    /// `Shift+H` · `Shift+V`.
    FlipH,
    FlipV,
    Rotate180,
    Rotate90Cw,
    Rotate90Ccw,
    /// Edit ▸ Shift ▸ Left/Right/Up/Down (um píxel, com volta dentro da caixa da selecção).
    ShiftLeft,
    ShiftRight,
    ShiftUp,
    ShiftDown,
    /// `Delete`: limpa as células seleccionadas (Background: a cor de fundo).
    Clear,
    /// `Ctrl+C` · `Ctrl+X` · `Ctrl+Shift+C` · `Ctrl+V`.
    Copy,
    Cut,
    CopyMerged,
    Paste,
    /// `Ctrl+T`: levanta as células seleccionadas numa moldura solta.
    Transform,
    /// `Enter`: assenta a moldura (UM passo). `Esc`: larga-a sem mudar nada.
    Commit,
    Cancel,
    /// `Ctrl+B` (Edit ▸ New Brush): a selecção vira o pincel de padrão; sem selecção, arrastar define-o.
    NewBrush,
    /// P8: `F` pinta a selecção com a cor de frente; `S` a borda interna dela (sem diálogo).
    Fill,
    Stroke,
    /// P8: os diálogos de filtro (Preview · OK · Cancel). Outline = `Shift+O`.
    Invert,
    Despeckle,
    Outline,
    /// P8: Edit ▸ Adjustments ▸ Brightness/Contrast…, Hue/Saturation… (`Ctrl+U`), Color Curve… (`Ctrl+M`).
    BrightnessContrast,
    HueSaturation,
    ColorCurve,
    /// P8: Edit ▸ FX ▸ Convolution Matrix… (`F9`).
    ConvolutionMatrix,
    /// P8: Edit ▸ Insert Text: abre a caixa do Text no centro da vista.
    InsertText,
    /// P12a: Edit ▸ Undo History (a janela `ph2d-panel-pixel-history`).
    UndoHistory,
    /// P12a: Edit ▸ Keyboard Shortcuts… (a janela `ph2d-panel-pixel-keys`).
    KeyboardShortcuts,
    /// P12 P: Edit ▸ Grid & Background… (a janela `ph2d-panel-pixel-prefs`).
    GridSettings,
}

/// O menu Sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelSpriteAction {
    /// Abrem os diálogos (`C` = Canvas Size).
    CanvasSize,
    SpriteSize,
    RotateCanvas180,
    RotateCanvas90Cw,
    RotateCanvas90Ccw,
    FlipCanvasH,
    FlipCanvasV,
    /// Corta à caixa da selecção.
    Crop,
    /// Tira a borda transparente (ou de uma cor só).
    Trim,
    /// P12 U: Sprite ▸ Properties… (abre/fecha a secção no painel PIXEL; sem passo).
    Properties,
}

/// Um campo numérico da moldura solta (a barra do Aseprite com os píxeis levantados).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelTransformField {
    X,
    Y,
    W,
    H,
    /// Graus, positivo = ANTI-horário (Aseprite).
    Angle,
    Skew,
}

impl PixelTransformField {
    pub const ALL: [Self; 6] = [Self::X, Self::Y, Self::W, Self::H, Self::Angle, Self::Skew];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::X => "panel.pixel.xform.x",
            Self::Y => "panel.pixel.xform.y",
            Self::W => "panel.pixel.xform.w",
            Self::H => "panel.pixel.xform.h",
            Self::Angle => "panel.pixel.xform.angle",
            Self::Skew => "panel.pixel.xform.skew",
        }
    }
}

/// Os números da moldura solta, como o painel os mostra (inteiros: células e graus, como os campos
/// do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelTransformInfo {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// Anti-horário, graus.
    pub angle: i32,
    pub skew: i32,
}

impl PixelTransformInfo {
    /// O valor do campo `f`.
    #[must_use]
    pub const fn get(&self, f: PixelTransformField) -> i32 {
        match f {
            PixelTransformField::X => self.x,
            PixelTransformField::Y => self.y,
            PixelTransformField::W => self.w,
            PixelTransformField::H => self.h,
            PixelTransformField::Angle => self.angle,
            PixelTransformField::Skew => self.skew,
        }
    }
}

/// O método de Sprite Size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelSizeMethod {
    #[default]
    Nearest,
    Bilinear,
    RotSprite,
}

/// Qual diálogo do Pixel está aberto (um só `ContextMenuKind::PixelEditDialog` para os quatro).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelDialogKind {
    #[default]
    ColorRange,
    Modify(PixelModify),
    CanvasSize,
    SpriteSize,
    /// P8: os filtros (o canvas pré-visualiza enquanto abertos).
    Invert,
    Despeckle,
    Outline,
    BrightnessContrast,
    HueSaturation,
    ColorCurve,
    ConvolutionMatrix,
}

impl PixelDialogKind {
    /// Um diálogo de filtro (Preview, Target, OK = UM passo)?
    #[must_use]
    pub const fn is_filter(self) -> bool {
        matches!(
            self,
            Self::Invert
                | Self::Despeckle
                | Self::Outline
                | Self::BrightnessContrast
                | Self::HueSaturation
                | Self::ColorCurve
                | Self::ConvolutionMatrix
        )
    }
}

/// O OK de um diálogo do Pixel — o pedido que a família drena.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelDialogChoice {
    ColorRange {
        tolerance: u8,
    },
    Modify {
        kind: PixelModify,
        n: u32,
        brush: PixelModifyBrush,
    },
    /// As bordas a acrescentar (negativas cortam).
    CanvasSize {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    },
    SpriteSize {
        width: u32,
        height: u32,
        method: PixelSizeMethod,
    },
    Invert {
        channels: super::PixelChannels,
        /// O alvo «All» (todos os quadros e camadas); `false` = o cel activo.
        all: bool,
    },
    Despeckle {
        width: u32,
        height: u32,
        tiled: bool,
        channels: super::PixelChannels,
        /// O alvo «All» (todos os quadros e camadas); `false` = o cel activo.
        all: bool,
    },
    /// A cor do contorno é a de frente (a família sabe-a).
    Outline {
        place: super::PixelOutlinePlace,
        matrix: super::PixelOutlineMatrix,
        background: super::PixelOutlineBg,
        tiled: bool,
        channels: super::PixelChannels,
        /// O alvo «All» (todos os quadros e camadas); `false` = o cel activo.
        all: bool,
    },
    /// Brilho e contraste em `-100..=100`.
    BrightnessContrast {
        brightness: i32,
        contrast: i32,
        channels: super::PixelChannels,
        /// O alvo «All» (todos os quadros e camadas); `false` = o cel activo.
        all: bool,
    },
    /// Matiz `-180..=180`, saturação, luz/valor e alfa `-100..=100`.
    HueSaturation {
        mode: super::PixelHueMode,
        hue: i32,
        saturation: i32,
        lightness: i32,
        /// `-100..=100`: o alfa `× (1 + v/100)`.
        alpha: i32,
        channels: super::PixelChannels,
        /// O alvo «All» (todos os quadros e camadas); `false` = o cel activo.
        all: bool,
    },
    ColorCurve {
        curve: super::PixelCurve,
        channels: super::PixelChannels,
        /// O alvo «All» (todos os quadros e camadas); `false` = o cel activo.
        all: bool,
    },
    /// `kernel` = índice na lista que a família publicou (`PixelFilterDialog::kernels`).
    Convolution {
        kernel: u8,
        tiled: bool,
        channels: super::PixelChannels,
        /// O alvo «All» (todos os quadros e camadas); `false` = o cel activo.
        all: bool,
    },
}

impl PixelDialogChoice {
    /// Um filtro com o alvo «All» (todos os quadros × todas as camadas que se pintam)?
    #[must_use]
    pub const fn targets_all(self) -> bool {
        match self {
            Self::Invert { all, .. }
            | Self::Despeckle { all, .. }
            | Self::Outline { all, .. }
            | Self::BrightnessContrast { all, .. }
            | Self::HueSaturation { all, .. }
            | Self::ColorCurve { all, .. }
            | Self::Convolution { all, .. } => all,
            _ => false,
        }
    }
}

/// O estado do diálogo do Pixel aberto no `WidgetStore`: qual é, o tamanho do sprite ao abrir, as
/// escolhas que não são campos numéricos e o pedido do OK. Os campos numéricos vivem nos widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelEditDialog {
    pub kind: PixelDialogKind,
    pub size: (u32, u32),
    /// Canvas Size: a âncora `(coluna, linha)` em `0..=2` (`(1, 1)` = centro).
    pub anchor: (u8, u8),
    pub brush: PixelModifyBrush,
    pub method: PixelSizeMethod,
    /// Sprite Size: mudar um lado muda o outro na mesma proporção.
    pub lock_ratio: bool,
    /// Color Range: a última tolerância da sessão.
    pub tolerance: u8,
    /// Modify: o último `n` (píxeis) da sessão.
    pub modify_n: u32,
    /// P8: as escolhas dos diálogos de filtro.
    pub filter: super::PixelFilterDialog,
    pub(crate) request: Option<PixelDialogChoice>,
    /// O retângulo do sprite no ecrã `(x, y, w, h)` em px, que a família publica a cada quadro: os
    /// diálogos de filtro encostam-se onde o tapam menos ([`place_dialog`]).
    pub avoid: Option<PixelOnScreen>,
}

/// Um retângulo de ecrã `(x, y, w, h)` em px.
pub type ScreenBox = (i32, i32, u32, u32);
/// A área livre do desenho e o sprite dentro dela, no ecrã.
pub type PixelOnScreen = (ScreenBox, ScreenBox);

/// **Onde a caixa `w × h` assenta na vista `view`**: dos oito encostos (os quatro lados ao meio e
/// os quatro cantos, a `margin` da borda, acima da faixa `hint_band` do fundo onde vive a dica), o
/// de MENOR área sobreposta ao sprite `avoid`; empate = a ordem (direita, esquerda, cima, baixo,
/// cantos). Sem sprite, ao centro.
#[must_use]
pub fn place_dialog(
    view: Rect,
    (w, h): (f32, f32),
    avoid: Option<ScreenBox>,
    margin: f32,
    hint_band: f32,
) -> Rect {
    let centro = Rect::new(
        (view.x + (view.w - w) * 0.5).max(view.x),
        (view.y + (view.h - h) * 0.5).max(view.y),
        w,
        h,
    );
    let Some((sx, sy, sw, sh)) = avoid else {
        return centro;
    };
    #[expect(clippy::cast_precision_loss, reason = "px de ecrã")]
    let s = Rect::new(sx as f32, sy as f32, sw as f32, sh as f32);
    let fundo = (view.y + view.h - hint_band).max(view.y + h);
    let (x0, x1) = (view.x + margin, (view.x + view.w - margin - w).max(view.x));
    let (y0, y1) = (view.y + margin, (fundo - margin - h).max(view.y));
    let (xm, ym) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
    let sobra = |r: Rect| {
        let ix = (r.x + r.w).min(s.x + s.w) - r.x.max(s.x);
        let iy = (r.y + r.h).min(s.y + s.h) - r.y.max(s.y);
        ix.max(0.0) * iy.max(0.0)
    };
    [
        (x1, ym),
        (x0, ym),
        (xm, y0),
        (xm, y1),
        (x1, y0),
        (x0, y0),
        (x1, y1),
        (x0, y1),
    ]
    .into_iter()
    .map(|(x, y)| Rect::new(x, y, w, h))
    .fold(None, |melhor: Option<Rect>, r| match melhor {
        Some(m) if sobra(m) <= sobra(r) => Some(m),
        _ => Some(r),
    })
    .unwrap_or(centro)
}

impl Default for PixelEditDialog {
    /// Os do Aseprite ao abrir pela 1.ª vez: âncora ao centro, Lock Ratio ligado, tolerância 0,
    /// `n = 1`, pincel quadrado, vizinho mais próximo.
    fn default() -> Self {
        Self {
            kind: PixelDialogKind::default(),
            size: (1, 1),
            anchor: (1, 1),
            brush: PixelModifyBrush::default(),
            method: PixelSizeMethod::default(),
            lock_ratio: true,
            tolerance: 0,
            modify_n: 1,
            filter: super::PixelFilterDialog::default(),
            request: None,
            avoid: None,
        }
    }
}

/// As bordas `[left, top, right, bottom]` que levam um canvas `w×h` a `nw×nh` com a âncora
/// `(coluna, linha)` em `0..=2` — a MESMA aritmética de `ph2d_pixel_core::canvas::anchor_borders`
/// (o Aseprite: ao centro `left = trunc(dw/2)`, `top = dh − trunc(dh/2)`), reescrita aqui porque o
/// chrome não depende do núcleo; a família prova a igualdade.
#[must_use]
pub fn pixel_anchor_borders(w: u32, h: u32, nw: u32, nh: u32, anchor: (u8, u8)) -> [i32; 4] {
    let dw = i64::from(nw) - i64::from(w);
    let dh = i64::from(nh) - i64::from(h);
    let left = match anchor.0 {
        0 => 0,
        1 => dw / 2,
        _ => dw,
    };
    let top = match anchor.1 {
        0 => 0,
        1 => dh - dh / 2,
        _ => dh,
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "lados em u32 de no máximo 65535: as diferenças cabem em i32"
    )]
    [
        left as i32,
        top as i32,
        (dw - left) as i32,
        (dh - top) as i32,
    ]
}
