//! Os números da cena do tocar.

use ph2d_pixel_core::Direction;

pub const SIZE: (u32, u32) = (16, 16);
pub const COLORS: [[u8; 4]; 8] = [
    [220, 50, 50, 255],
    [240, 140, 40, 255],
    [235, 200, 40, 255],
    [40, 170, 70, 255],
    [40, 180, 190, 255],
    [50, 110, 230, 255],
    [150, 70, 200, 255],
    [230, 90, 170, 255],
];
/// As tags (posições, base 0): o nome, de, até, a direcção e as passadas.
pub const TAGS: [(&str, u32, u32, Direction, u16); 2] = [
    ("out", 1, 6, Direction::Forward, 0),
    ("in", 2, 4, Direction::PingPong, 2),
];
