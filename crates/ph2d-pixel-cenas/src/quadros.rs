//! Os números da cena 8.

use ph2d_pixel_core::{Direction, Rect};

/// O tamanho da cena 8 e o número de frames.
pub const SIZE: (u32, u32) = (64, 32);
pub const FRAMES: usize = 16;
/// O pilar e o chão.
pub const PILLAR: Rect = Rect::new(22, 6, 5, 20);
pub const GROUND: Rect = Rect::new(0, 26, 64, 6);
/// O raio da bola e o centro dela na posição `k` (0..4) de cada trecho.
pub const BALL_R: i32 = 4;
#[must_use]
pub const fn ball_center(k: usize) -> (i32, i32) {
    (8 + k as i32 * 16, 17)
}
/// A cor de cada trecho (a da tag e a da bola).
pub const COLORS: [[u8; 4]; 4] = [
    [220, 50, 50, 255],
    [40, 170, 70, 255],
    [50, 110, 230, 255],
    [235, 200, 40, 255],
];
/// As tags: (nome, direcção, passadas).
pub const TAGS: [(&str, Direction, u16); 4] = [
    ("Forward", Direction::Forward, 2),
    ("Reverse", Direction::Reverse, 2),
    ("Ping-pong", Direction::PingPong, 2),
    ("Ping-pong Reverse", Direction::PingPongReverse, 0),
];
/// A duração de cada frame e a do frame que segura a bola (o 4.º).
pub const FRAME_MS: u32 = 120;
pub const HOLD_MS: u32 = 480;
pub const HOLD_INDEX: usize = 3;

pub fn bola(c: [u8; 4]) -> impl Fn(i32, i32) -> [u8; 4] {
    move |x, y| {
        let (dx, dy) = (x - BALL_R, y - BALL_R);
        if dx * dx + dy * dy <= BALL_R * BALL_R {
            c
        } else {
            [0; 4]
        }
    }
}
