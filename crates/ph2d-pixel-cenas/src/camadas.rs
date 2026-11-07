//! Os números e a tinta da cena 7.

use ph2d_pixel_core::Rect;

/// O tamanho da cena 7.
pub const SIZE: (u32, u32) = (190, 96);
/// A largura de cada faixa de modo e a altura da banda de cima.
pub const STRIPE_W: u32 = 10;
pub const BAND_H: u32 = 48;

/// A tinta das faixas na linha `y` da banda: laranja → cinza → azul, opaca.
#[must_use]
pub fn stripe_rgba(y: u32) -> [u8; 4] {
    let t = y as f32 / (BAND_H - 1) as f32;
    let (a, m, b) = (
        [255.0, 170.0, 60.0],
        [128.0, 128.0, 128.0],
        [40.0, 90.0, 230.0],
    );
    let mix = |p: [f32; 3], q: [f32; 3], u: f32| [0, 1, 2].map(|k| p[k] + (q[k] - p[k]) * u);
    let c = if t < 0.5 {
        mix(a, m, t * 2.0)
    } else {
        mix(m, b, t * 2.0 - 1.0)
    };
    [
        c[0].round() as u8,
        c[1].round() as u8,
        c[2].round() as u8,
        255,
    ]
}

/// O retângulo da faixa do modo `i` (0 = Normal … 18 = Divide).
#[must_use]
pub fn stripe_rect(i: usize) -> Rect {
    Rect::new(i as i32 * STRIPE_W as i32, 0, STRIPE_W, BAND_H)
}

/// As peças de baixo: (nome, retângulo).
pub const OPACITY_BOX: Rect = Rect::new(4, 54, 30, 36);
pub const HIDDEN_STAR: Rect = Rect::new(44, 54, 30, 36);
pub const LOCKED_BOX: Rect = Rect::new(84, 54, 30, 36);
pub const REFERENCE_BOX: Rect = Rect::new(124, 54, 28, 36);
pub const GROUP_A: Rect = Rect::new(158, 54, 14, 20);
pub const GROUP_B: Rect = Rect::new(166, 66, 20, 24);

/// A estrela de cinco pontas inscrita em `r` (dentro = amarela).
pub fn estrela(r: Rect, x: i32, y: i32) -> [u8; 4] {
    let (cx, cy) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0);
    let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
    let raio = (dx * dx + dy * dy).sqrt();
    let a = dy.atan2(dx) + std::f32::consts::FRAC_PI_2;
    let k = (a * 5.0 / std::f32::consts::TAU).rem_euclid(1.0);
    let borda = r.w.min(r.h) as f32 / 2.0 * (0.45 + 0.55 * (1.0 - (k - 0.5).abs() * 2.0));
    if raio <= borda {
        [250, 220, 40, 255]
    } else {
        [0; 4]
    }
}
