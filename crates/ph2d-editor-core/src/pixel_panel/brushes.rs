//! **O pincel na mão e os pincéis guardados** (P7b) no vocabulário do chrome: os espaços Alt+1…
//! com a miniatura, o padrão do Ctrl+B e os gestos do painel sobre eles. A família traduz para
//! `ph2d_pixel_core::pattern` e `ph2d_pixel_io::brush_file`.

use super::PixelBrushShape;

/// Virar ou girar o pincel de padrão (decisão do dono 02/11, §6.2 do handoff P7a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelPatternTurn {
    FlipH,
    FlipV,
    RotateRight,
    RotateLeft,
}

impl PixelPatternTurn {
    pub const ALL: [Self; 4] = [
        Self::FlipH,
        Self::FlipV,
        Self::RotateLeft,
        Self::RotateRight,
    ];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::FlipH => "panel.pixel.brush.flip_h",
            Self::FlipV => "panel.pixel.brush.flip_v",
            Self::RotateRight => "panel.pixel.brush.rotate_right",
            Self::RotateLeft => "panel.pixel.brush.rotate_left",
        }
    }
}

/// Um gesto do painel sobre o pincel (`PixelRequest::Brush`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelBrushAction {
    /// Uma das três formas: larga o padrão.
    Shape(PixelBrushShape),
    /// O espaço `n` (de 0) — o mesmo que Alt+(n+1).
    Recall(usize),
    /// *Save Brush Here*: o pincel actual inteiro num espaço novo, trancado.
    SaveHere,
    ToggleLock(usize),
    Delete(usize),
    Turn(PixelPatternTurn),
}

/// O maior lado da miniatura de um espaço, em células (a família amostra o padrão por vizinho).
pub const PIXEL_BRUSH_THUMB_MAX: u32 = 16;

/// A miniatura de um padrão: `w × h` células, linha a linha; `None` = fora do pincel ou transparente.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PixelBrushThumb {
    pub w: u32,
    pub h: u32,
    pub cells: Vec<Option<[u8; 4]>>,
}

/// O que um espaço guarda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PixelBrushSlotKind {
    Shape { shape: PixelBrushShape, size: u32 },
    Pattern(PixelBrushThumb),
}

/// Um espaço da lista, como o painel o mostra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelBrushSlotInfo {
    pub locked: bool,
    pub kind: PixelBrushSlotKind,
}
