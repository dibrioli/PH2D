//! Os números e o boneco da cena 19.

use ph2d_pixel_core::Rect;

/// O tamanho da cena 19.
pub const SIZE: (u32, u32) = (64, 48);
/// A caixa do boneco na camada de cima (fora dela a camada é transparente).
pub const FIGURE: Rect = Rect::new(6, 6, 16, 35);
/// A pele, a camisa e as calças do boneco.
pub const SKIN: [u8; 4] = [240, 190, 140, 255];
pub const SHIRT: [u8; 4] = [50, 100, 210, 255];
pub const PANTS: [u8; 4] = [60, 50, 70, 255];
/// A rampa (roxo → amarelo), acrescentada depois das cores de fábrica da paleta.
pub const RAMP: [[u8; 4]; 6] = [
    [40, 20, 80, 255],
    [100, 30, 120, 255],
    [180, 50, 110, 255],
    [235, 100, 80, 255],
    [250, 170, 60, 255],
    [255, 235, 120, 255],
];

/// A cor do boneco na célula `(x, y)`: cabeça, braços e tronco, pernas.
#[must_use]
pub fn boneco(x: i32, y: i32) -> Option<[u8; 4]> {
    match (x, y) {
        (10..=17, 6..=13) => Some(SKIN),
        (6..=7 | 20..=21, 15..=24) => Some(SKIN),
        (8..=19, 15..=28) => Some(SHIRT),
        (9..=12 | 15..=18, 29..=40) => Some(PANTS),
        _ => None,
    }
}
