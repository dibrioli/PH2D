//! O vocabulário da SELECÇÃO e da TRANSFORMAÇÃO do Pixel (P6): o modo de juntar, os menus Select,
//! Edit e Sprite do Aseprite, a moldura solta e os diálogos (Color Range, Modify, Canvas Size,
//! Sprite Size). A família (`ph2d-app-pixel`) traduz para o núcleo.

/// Como uma selecção nova se junta à que há (os quatro botões da barra do Aseprite; no rato:
/// Shift = somar, Alt+Shift = tirar, Ctrl+Shift = cruzar).
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
    pub(crate) request: Option<PixelDialogChoice>,
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
            request: None,
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
