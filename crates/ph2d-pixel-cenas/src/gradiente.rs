//! A arte da cena 4: uma esfera, um céu e uma faixa arco-íris.

/// O tamanho da cena 4.
pub const GRADIENT_SIZE: (u32, u32) = (96, 64);
/// As linhas da faixa arco-íris (as últimas da imagem).
const RAINBOW_ROWS: u32 = 10;
/// O centro e o raio da esfera.
const SPHERE: ((f32, f32), f32) = ((48.0, 26.0), 20.0);

/// A cor da célula `(x, y)` da cena 4.
#[must_use]
pub fn gradient_rgba(x: u32, y: u32) -> [u8; 4] {
    let (w, h) = GRADIENT_SIZE;
    let byte = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    if y >= h - RAINBOW_ROWS {
        let [r, g, b] = hue(x as f32 / w as f32);
        return [byte(r * 255.0), byte(g * 255.0), byte(b * 255.0), 255];
    }
    let ((cx, cy), r) = SPHERE;
    let (dx, dy) = ((x as f32 + 0.5 - cx) / r, (y as f32 + 0.5 - cy) / r);
    let d2 = dx * dx + dy * dy;
    if d2 <= 1.0 {
        // Lambert + um brilho, a luz de cima à esquerda e à frente.
        let nz = (1.0 - d2).sqrt();
        let (lx, ly, lz) = (-0.48_f32, -0.6_f32, 0.64_f32);
        let lambert = (dx * lx + dy * ly + nz * lz).max(0.0);
        let spec = lambert.powi(24);
        let k = 0.12 + 0.88 * lambert;
        let base = [90.0_f32, 170.0, 235.0];
        return [
            byte(base[0] * k + 255.0 * spec),
            byte(base[1] * k + 255.0 * spec),
            byte(base[2] * k + 255.0 * spec),
            255,
        ];
    }
    let t = y as f32 / (h - RAINBOW_ROWS - 1) as f32;
    let top = [18.0_f32, 28.0, 96.0];
    let bottom = [252.0_f32, 150.0, 58.0];
    let mix = |i: usize| byte(top[i] + (bottom[i] - top[i]) * t + x as f32 * 0.25);
    [mix(0), mix(1), mix(2), 255]
}

/// O arco-íris: matiz `t ∈ [0, 1)` com saturação e valor cheios.
fn hue(t: f32) -> [f32; 3] {
    let h = (t * 6.0).rem_euclid(6.0);
    let f = h - h.floor();
    match h as u32 {
        0 => [1.0, f, 0.0],
        1 => [1.0 - f, 1.0, 0.0],
        2 => [0.0, 1.0, f],
        3 => [0.0, 1.0 - f, 1.0],
        4 => [f, 0.0, 1.0],
        _ => [1.0, 0.0, 1.0 - f],
    }
}
