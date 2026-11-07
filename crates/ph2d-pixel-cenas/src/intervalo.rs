//! Os números da cena 16.

use ph2d_pixel_core::Rect;

pub const SIZE: (u32, u32) = (16, 16);
/// As durações (ms) e as cores dos 6 quadros (vermelho, verde, amarelo, azul, laranja, roxo).
pub const DURATIONS: [u32; 6] = [100, 200, 300, 400, 500, 600];
pub const COLORS: [[u8; 4]; 6] = [
    [220, 50, 50, 255],
    [40, 170, 70, 255],
    [235, 200, 40, 255],
    [50, 110, 230, 255],
    [240, 140, 40, 255],
    [150, 70, 200, 255],
];
/// O quadrado de cada quadro na camada A e o ponto da camada B.
pub const SQUARE: Rect = Rect::new(2, 2, 12, 12);
pub const DOT: Rect = Rect::new(6, 6, 4, 4);
/// A tag (posições, base 0) e o intervalo de abertura (posições).
pub const TAG: (u32, u32) = (1, 3);
pub const RANGE: [usize; 2] = [1, 2];
