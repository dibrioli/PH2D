//! ⭐ **O boneco de fábrica** — a folha do *Character* e do *Animated Sprite* do menu Add
//! (`docs/GameEngineFuncional/05_N1_plano.md` §2): 4 linhas (parado · correr · saltar · cair) × 6 células.
//!
//! Procedimental e determinística: uma função pura desenha cada célula por SDF com anti-alias de um texel,
//! e cada célula tem margem transparente (o filtro bilinear não sangra da vizinha). Grava-se pela chave
//! reservada, como o quadrado branco; o boneco olha para a DIREITA (`facing = +1`).

use ph2d_color::SrgbRgba;
use std::f32::consts::TAU;

/// Chave reservada da folha — abaixo do disco, no topo do espaço de chaves.
pub const HERO_SHEET_KEY: u32 = u32::MAX - 2;
/// Lado de uma célula, em texels.
pub const HERO_SHEET_CELL_PX: u32 = 64;
/// Células por linha.
pub const HERO_SHEET_COLS: u32 = 6;
/// Linhas (uma por animação).
pub const HERO_SHEET_ROWS: u32 = 4;

const _: () = assert!(HERO_SHEET_KEY > super::FIRST_IMPORT_KEY && HERO_SHEET_KEY < super::DISC_TILE_KEY);

/// Do centro da célula à sola do boneco, em fracção do lado da célula (a perna acaba no texel 61,6 de
/// 64). Quem põe o boneco no mundo faz o personagem pairar a esta altura: os pés tocam o chão.
pub const HERO_SHEET_SOLE_BELOW_CENTRE: f32 = (SOLE_PX - 32.0) / 64.0;

/// A sola, em texels a partir do topo da célula (duas de margem transparente por baixo).
const SOLE_PX: f32 = 61.6;

/// As animações da folha, por linha: nome da tag, primeira e última célula (inclusivas), ms por célula.
pub const HERO_SHEET_TAGS: [(&str, u32, u32, u32); 4] = [
    ("idle", 0, 5, 140),
    ("run", 6, 11, 80),
    ("jump", 12, 17, 90),
    ("fall", 18, 23, 90),
];

/// A pose de uma célula: deslocamentos em texels, ângulos em radianos (0 = pendurado para baixo).
#[derive(Clone, Copy)]
struct Pose {
    bob: f32,
    stretch: f32,
    lean: f32,
    leg: [f32; 2],
    knee_up: f32,
    arm: [f32; 2],
}

fn pose(row: u32, i: u32) -> Pose {
    let t = i as f32 / HERO_SHEET_COLS as f32;
    let s = (t * TAU).sin();
    match row {
        0 => Pose {
            bob: s * 1.2,
            stretch: 1.0 + s * 0.03,
            lean: 0.0,
            leg: [0.0, 0.0],
            knee_up: 0.0,
            arm: [0.15 + s * 0.05, -0.15 - s * 0.05],
        },
        1 => Pose {
            bob: -(t * TAU).sin().abs() * 2.5,
            stretch: 1.0,
            lean: 0.12,
            leg: [s * 0.65, -s * 0.65],
            knee_up: 0.0,
            arm: [-s * 0.8, s * 0.8],
        },
        2 => Pose {
            bob: -2.0 - s * 0.6,
            stretch: 1.08,
            lean: 0.05,
            leg: [0.45, -0.25],
            knee_up: 5.0,
            arm: [2.6 + s * 0.15, -2.6 - s * 0.15],
        },
        _ => Pose {
            bob: 0.0,
            stretch: 0.95,
            lean: -0.05,
            leg: [0.35 + s * 0.08, -0.35 - s * 0.08],
            knee_up: 0.0,
            arm: [1.9 + s * 0.35, -1.9 - s * 0.35],
        },
    }
}

const BODY: [f32; 3] = [0.20, 0.66, 0.78];
const BELLY: [f32; 3] = [0.62, 0.88, 0.92];
const LIMB: [f32; 3] = [0.13, 0.20, 0.30];
const EYE: [f32; 3] = [1.0, 1.0, 1.0];
const PUPIL: [f32; 3] = [0.08, 0.10, 0.16];

/// Distância com sinal a um segmento de espessura `r` (cápsula).
fn capsule(p: [f32; 2], a: [f32; 2], b: [f32; 2], r: f32) -> f32 {
    let (pa, ba) = ([p[0] - a[0], p[1] - a[1]], [b[0] - a[0], b[1] - a[1]]);
    let h = ((pa[0] * ba[0] + pa[1] * ba[1]) / (ba[0] * ba[0] + ba[1] * ba[1]).max(1e-6)).clamp(0.0, 1.0);
    let (dx, dy) = (pa[0] - ba[0] * h, pa[1] - ba[1] * h);
    (dx * dx + dy * dy).sqrt() - r
}

/// Distância (aproximada) a uma elipse de semi-eixos `rx`, `ry`.
fn ellipse(p: [f32; 2], c: [f32; 2], rx: f32, ry: f32) -> f32 {
    let (x, y) = ((p[0] - c[0]) / rx, (p[1] - c[1]) / ry);
    ((x * x + y * y).sqrt() - 1.0) * rx.min(ry)
}

/// Cobertura de uma distância com sinal: rampa de um texel na borda.
fn cover(d: f32) -> f32 {
    (0.5 - d).clamp(0.0, 1.0)
}

/// Uma célula em sRGB pré-multiplicado `[r, g, b, a]` por texel (fundo transparente).
fn cell(row: u32, col: u32) -> Vec<[f32; 4]> {
    let n = HERO_SHEET_CELL_PX as usize;
    let p = pose(row, col);
    let cx = n as f32 * 0.5;
    // O boneco ocupa ~13..61 na vertical: os pés no fundo da célula (o personagem PAIRA — a cápsula
    // cobre corpo e cabeça e as pernas desenham-se por baixo dela, até ao chão).
    let leg_len = 13.0 - p.knee_up;
    // No chão (parado, correr) o pé mais baixo pousa SEMPRE na sola: a anca desce quando as pernas abrem.
    let hip_y = if row <= 1 {
        SOLE_PX - 2.6 - leg_len * p.leg[0].cos().max(p.leg[1].cos())
    } else {
        46.0 + p.bob * 0.4
    };
    let hip = [cx, hip_y];
    let body_c = [cx + p.lean * 8.0, hip_y - 16.0 + p.bob];
    let (rx, ry) = (14.0 / p.stretch.sqrt(), 17.0 * p.stretch);
    let shoulder = [body_c[0], body_c[1] + 3.0];
    let limb = |base: [f32; 2], ang: f32, len: f32| [base[0] + ang.sin() * len, base[1] + ang.cos() * len];
    // Ancas juntas quando as pernas balançam (abrem em tesoura em vez de cruzar).
    let hips = if p.leg[0] * p.leg[1] < 0.0 && row == 1 { [hip[0] - 1.5, hip[0] + 1.5] } else { [hip[0] - 4.0, hip[0] + 4.0] };
    let feet = [
        limb([hips[0], hip[1]], p.leg[0], leg_len),
        limb([hips[1], hip[1]], p.leg[1], leg_len),
    ];
    let hands = [
        limb([shoulder[0] - 11.0, shoulder[1]], -p.arm[0], 10.0),
        limb([shoulder[0] + 11.0, shoulder[1]], -p.arm[1], 10.0),
    ];
    let eye_c = [body_c[0] + 5.0, body_c[1] - 7.0];
    let mut out = vec![[0.0; 4]; n * n];
    for y in 0..n {
        for x in 0..n {
            let q = [x as f32 + 0.5, y as f32 + 0.5];
            let mut px = [0.0f32; 4];
            let mut over = |rgb: [f32; 3], a: f32| {
                for k in 0..3 {
                    px[k] = rgb[k] * a + px[k] * (1.0 - a);
                }
                px[3] = a + px[3] * (1.0 - a);
            };
            for (i, foot) in feet.iter().enumerate() {
                over(LIMB, cover(capsule(q, [hips[i], hip[1]], *foot, 2.6)));
            }
            over(LIMB, cover(capsule(q, [shoulder[0] - 11.0, shoulder[1]], hands[0], 2.2)));
            over(BODY, cover(ellipse(q, body_c, rx, ry)));
            over(BELLY, cover(ellipse(q, [body_c[0] + 1.5, body_c[1] + 5.0], rx * 0.62, ry * 0.5)));
            over(EYE, cover(ellipse(q, eye_c, 4.6, 5.5)));
            over(PUPIL, cover(ellipse(q, [eye_c[0] + 1.8, eye_c[1] + 0.5], 2.1, 2.8)));
            over(LIMB, cover(capsule(q, [shoulder[0] + 11.0, shoulder[1]], hands[1], 2.2)));
            out[y * n + x] = px;
        }
    }
    out
}

/// ⭐ Os texels da folha inteira (`COLS × CELL` por `ROWS × CELL`), sRGB em alfa reto, linha a linha.
#[must_use]
pub fn hero_sheet_pixels() -> Vec<SrgbRgba> {
    let n = HERO_SHEET_CELL_PX as usize;
    let w = n * HERO_SHEET_COLS as usize;
    let mut px = vec![SrgbRgba::new(0, 0, 0, 0); w * n * HERO_SHEET_ROWS as usize];
    for row in 0..HERO_SHEET_ROWS {
        for col in 0..HERO_SHEET_COLS {
            for (k, c) in cell(row, col).into_iter().enumerate() {
                let (x, y) = (k % n, k / n);
                let a = c[3];
                let rgb = |v: f32| {
                    let straight = if a > 0.0 { v / a } else { 0.0 };
                    (straight.clamp(0.0, 1.0) * 255.0).round() as u8
                };
                px[(row as usize * n + y) * w + col as usize * n + x] =
                    SrgbRgba::new(rgb(c[0]), rgb(c[1]), rgb(c[2]), (a * 255.0).round() as u8);
            }
        }
    }
    px
}

#[cfg(test)]
#[path = "hero_sheet_tests.rs"]
mod tests;
