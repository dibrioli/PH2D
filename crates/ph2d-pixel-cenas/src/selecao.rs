//! Os números e a arte da cena 9.

use ph2d_pixel_core::Rect;
use ph2d_pixel_core::transform::Image;

/// O tamanho da cena 9.
pub const SIZE: (u32, u32) = (128, 64);
/// A marquee retangular e o traço que a atravessa (linha `STROKE_Y`, de `0` a `STROKE_X1`).
pub const RECT: Rect = Rect::new(4, 4, 16, 12);
pub const STROKE_Y: i32 = 10;
pub const STROKE_X1: i32 = 24;
pub const STROKE: [u8; 4] = [235, 60, 60, 255];
/// A caixa da elipse (cantos inclusivos).
pub const ELLIPSE: ((i32, i32), (i32, i32)) = ((27, 4), (42, 19));
/// A estrela de laço (vértices em células).
pub const STAR: [(i32, i32); 10] = [
    (56, 2),
    (59, 9),
    (66, 9),
    (60, 13),
    (63, 20),
    (56, 16),
    (49, 20),
    (52, 13),
    (46, 9),
    (53, 9),
];
/// O losango da varinha: centro e raio (|dx| + |dy| ≤ r), e a cor dele.
pub const WAND: ((i32, i32), i32) = ((78, 12), 8);
pub const WAND_COLOR: [u8; 4] = [60, 180, 90, 255];
/// A imagem 3 × 3 aumentada: onde ficam a Nearest e a Bilinear (12 × 12).
pub const NEAREST_AT: (i32, i32) = (92, 4);
pub const BILINEAR_AT: (i32, i32) = (110, 4);
pub const SCALED: u32 = 12;
/// A figura (14 × 14) e onde ficam a original, as duas giradas, a espelhada e o ladrilho.
pub const FIGURE: u32 = 14;
pub const ORIGINAL_AT: (i32, i32) = (4, 42);
pub const FAST_AT: (i32, i32) = (26, 42);
pub const ROTSPRITE_AT: (i32, i32) = (50, 42);
pub const FLIP_AT: (i32, i32) = (74, 42);
pub const TILE_AT: (i32, i32) = (96, 42);
/// O giro das cópias (anti-horário, o campo do Aseprite) e o deslocamento do ladrilho.
pub const ANGLE: f64 = 30.0;
pub const SHIFT: (i32, i32) = (5, 4);

/// A imagem 3 × 3 de cores (RGBA) que a cena aumenta.
#[must_use]
pub fn small() -> Image {
    let c = [
        [230, 70, 70, 255],
        [240, 200, 60, 255],
        [70, 170, 90, 255],
        [70, 120, 230, 255],
        [250, 250, 250, 255],
        [150, 80, 200, 255],
        [40, 40, 40, 255],
        [200, 120, 60, 255],
        [90, 210, 220, 255],
    ];
    Image::map(3, 3, 4, |x, y| u32::from_le_bytes(c[(y * 3 + x) as usize]))
}

/// A figura: um «F» grosso de duas cores (sem simetria — o giro e o espelho leem-se nele).
#[must_use]
pub fn figure() -> Image {
    Image::map(FIGURE, FIGURE, 4, |x, y| {
        let haste = (2..5).contains(&x) && (1..13).contains(&y);
        let topo = (2..12).contains(&x) && (1..4).contains(&y);
        let meio = (2..9).contains(&x) && (6..8).contains(&y);
        let c: [u8; 4] = if topo {
            [240, 140, 40, 255]
        } else if haste || meio {
            [60, 90, 200, 255]
        } else {
            [0; 4]
        };
        u32::from_le_bytes(c)
    })
}

/// O ladrilho antes do Shift: um quadrante de cada cor (a volta lê-se nas costuras).
#[must_use]
pub fn tile() -> Image {
    let h = FIGURE / 2;
    Image::map(FIGURE, FIGURE, 4, |x, y| {
        let c: [u8; 4] = match (x < h, y < h) {
            (true, true) => [220, 60, 60, 255],
            (false, true) => [240, 210, 70, 255],
            (true, false) => [60, 170, 90, 255],
            (false, false) => [70, 110, 220, 255],
        };
        u32::from_le_bytes(c)
    })
}
