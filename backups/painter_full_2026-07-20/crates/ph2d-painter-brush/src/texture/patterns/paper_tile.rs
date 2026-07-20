//! Pre-baked **paper-tooth tiles** (doc 12 GRAN-2, rota 2 — padrão-ouro): each Paper preset bakes a
//! seamless 256² grayscale height-field ONCE (lazy, deterministic — periodic integer-lattice value
//! noise, fixed seeds, HR-5 transcendental-free) and the hot loop becomes a single bilinear
//! wrap-fetch — CHEAPER than the old 4-octave fBm + Worley per pixel, and with the spectrum the old
//! generator only claimed to have.
//!
//! **Why (measured, doc 12 §5):** FFT of the old generator put ~90-98% of the spectral energy at
//! λ > 32 px (a low-frequency BLOTCH — its "high-pass" subtracted *uncorrelated* noise, which adds
//! low-band energy instead of removing it). Multiplying the wash density by that field is exactly
//! the reported "digital / mottled" look. Here the energy is confined to the tooth band **by
//! construction**: the coarsest octave cell is 32 px, so there is nothing above λ ≈ 32 px to leak.
//! Gold-standard shape per Bousseau 2006 (multi-scale sums) / Rebelle (pre-baked paper images).
//!
//! Tiles are PERIODIC by construction (every octave cell size divides the tile), so the canvas-
//! anchored wrap is seamless; each tile is normalised to mean 0.5 with a preset-scaled std (little
//! to no clamping — the old field clamped 4.2% of texels at 0).

use std::sync::OnceLock;

/// Tile side in texels. At Paper Size 1 one texel maps to one canvas pixel
/// (`rel × TILE` with `rel = px·size/TEX_TILE_BASE_PX` and `TEX_TILE_BASE_PX == TILE`).
pub(super) const TILE: usize = 256;

/// One octave of the bake: periodic cell size (px, MUST divide [`TILE`]) + its weight.
struct Octave {
    cell: usize,
    weight: f32,
}

/// A preset's bake recipe: tooth octaves + laid-line ridge mix + felt (Worley) mix + tooth depth
/// (the normalised std of the finished tile).
struct Recipe {
    octaves: &'static [Octave],
    /// Ridged laid-lines mix (`0` = none): long in x, tight in y (cells 64 × 8).
    ridge_mix: f32,
    /// Periodic Worley felt-cell mix (`0` = none): the hot-press mottle (period-16 lattice).
    felt_mix: f32,
    /// Target std of the normalised tile (the tooth depth; UI Contrast fine-tunes on top).
    depth: f32,
}

/// **Cold Press** — medium tooth, subtle fibre. Mid-band (8-32 px) dominant.
const COLD: Recipe = Recipe {
    octaves: &[
        Octave {
            cell: 16,
            weight: 0.40,
        },
        Octave {
            cell: 8,
            weight: 0.30,
        },
        Octave {
            cell: 4,
            weight: 0.20,
        },
        Octave {
            cell: 2,
            weight: 0.10,
        },
    ],
    ridge_mix: 0.12,
    felt_mix: 0.0,
    depth: 0.16,
};
/// **Rough** — deep coarse tooth + pronounced laid creases.
const ROUGH: Recipe = Recipe {
    octaves: &[
        Octave {
            cell: 32,
            weight: 0.35,
        },
        Octave {
            cell: 16,
            weight: 0.35,
        },
        Octave {
            cell: 8,
            weight: 0.20,
        },
        Octave {
            cell: 4,
            weight: 0.10,
        },
    ],
    ridge_mix: 0.40,
    felt_mix: 0.0,
    depth: 0.20,
};
/// **Hot Press** — fine smooth grain + soft felt mottle, minimal depth.
const HOT: Recipe = Recipe {
    octaves: &[
        Octave {
            cell: 8,
            weight: 0.50,
        },
        Octave {
            cell: 4,
            weight: 0.30,
        },
        Octave {
            cell: 2,
            weight: 0.20,
        },
    ],
    ridge_mix: 0.0,
    felt_mix: 0.35,
    depth: 0.11,
};

/// Integer-lattice hash → `[0, 1)` (same avalanche family as the pattern noise; period-safe because
/// the caller reduces the lattice coords modulo the octave period BEFORE hashing).
fn hash2p(ix: i64, iy: i64, seed: u32) -> f32 {
    let mut h = (ix as u32).wrapping_mul(0x8DA6_B343)
        ^ (iy as u32).wrapping_mul(0xD816_3841)
        ^ seed.wrapping_mul(0x1B56_C4E9);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// Cubic fade (transcendental-free).
fn fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// PERIODIC value noise at tile texel `(x, y)` with integer `cell` size: the lattice wraps at
/// `TILE / cell`, so the tile edge is seamless by construction.
fn periodic_noise(x: f32, y: f32, cell: usize, seed: u32) -> f32 {
    let period = (TILE / cell) as i64;
    let fx = x / cell as f32;
    let fy = y / cell as f32;
    let x0 = fx.floor();
    let y0 = fy.floor();
    let (ix, iy) = (x0 as i64, y0 as i64);
    let sx = fade(fx - x0);
    let sy = fade(fy - y0);
    let g = |dx: i64, dy: i64| {
        hash2p(
            (ix + dx).rem_euclid(period),
            (iy + dy).rem_euclid(period),
            seed,
        )
    };
    let (a, b, c, d) = (g(0, 0), g(1, 0), g(0, 1), g(1, 1));
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

/// Periodic Worley F1 (felt cells): jittered point per period-16 lattice cell, wrapped neighbours.
fn periodic_felt(x: f32, y: f32, seed: u32) -> f32 {
    const CELL: usize = 16;
    let period = (TILE / CELL) as i64;
    let (cx, cy) = (
        (x / CELL as f32).floor() as i64,
        (y / CELL as f32).floor() as i64,
    );
    let mut f1 = f32::MAX;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (gx, gy) = (cx + dx, cy + dy);
            let (wx, wy) = (gx.rem_euclid(period), gy.rem_euclid(period));
            let jx = gx as f32 + hash2p(wx, wy, seed);
            let jy = gy as f32 + hash2p(wy + 31, wx + 17, seed);
            let (ex, ey) = (jx * CELL as f32 - x, jy * CELL as f32 - y);
            let d = (ex * ex + ey * ey).sqrt() / CELL as f32;
            f1 = f1.min(d);
        }
    }
    f1.min(1.0)
}

/// Bake one preset's tile: octave sum + ridge/felt character, then normalise to mean 0.5 /
/// std `depth` (clamped to `[0, 1]` — the target std keeps clamping negligible).
fn bake(recipe: &Recipe, seed: u32) -> Vec<f32> {
    let mut t = vec![0.0f32; TILE * TILE];
    let wsum: f32 = recipe.octaves.iter().map(|o| o.weight).sum();
    for y in 0..TILE {
        for x in 0..TILE {
            let (fx, fy) = (x as f32, y as f32);
            let mut v = 0.0;
            for (i, o) in recipe.octaves.iter().enumerate() {
                v += o.weight * periodic_noise(fx, fy, o.cell, seed ^ (i as u32 * 0x9E37));
            }
            let mut h = v / wsum;
            if recipe.ridge_mix > 0.0 {
                // Laid lines: ridged noise, long in x (cell 64) / tight in y (cell 8) — both divide TILE.
                let nx = periodic_noise(fx, fy, 64, seed ^ 0x5249);
                let ny = periodic_noise(fy, fx, 8, seed ^ 0x4447);
                let ridge = 1.0 - (2.0 * (0.5 * nx + 0.5 * ny) - 1.0).abs();
                h = h * (1.0 - recipe.ridge_mix) + h * ridge * recipe.ridge_mix;
            }
            if recipe.felt_mix > 0.0 {
                let felt = periodic_felt(fx, fy, seed ^ 0x4645);
                h = h * (1.0 - recipe.felt_mix) + h * (0.55 + 0.45 * felt) * recipe.felt_mix;
            }
            t[y * TILE + x] = h;
        }
    }
    // TRUE high-pass (the audit's prescription — the old generator subtracted UNCORRELATED noise,
    // which *adds* low-band energy): subtract the tile's own periodic 32-px box blur, removing every
    // λ > ~32 px residue (octave leakage, the ridge's slow x-variation) by construction. One-time.
    let blurred = periodic_box_blur(&t, 16);
    for (v, b) in t.iter_mut().zip(&blurred) {
        *v -= b;
    }
    // Normalise: mean 0.5, std = depth (little/no clamping — the tooth stays inside [0, 1]).
    let n = t.len() as f32;
    let mean = t.iter().sum::<f32>() / n;
    let std = (t.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n).sqrt();
    let scale = if std > 1e-6 { recipe.depth / std } else { 0.0 };
    for v in &mut t {
        *v = (0.5 + (*v - mean) * scale).clamp(0.0, 1.0);
    }
    t
}

/// Periodic (wrapping) separable box blur of the square tile — bake-time only.
fn periodic_box_blur(src: &[f32], radius: i64) -> Vec<f32> {
    let n = TILE as i64;
    let win = (2 * radius + 1) as f32;
    let mut tmp = vec![0.0f32; src.len()];
    for y in 0..n {
        for x in 0..n {
            let mut acc = 0.0;
            for d in -radius..=radius {
                acc += src[(y * n + (x + d).rem_euclid(n)) as usize];
            }
            tmp[(y * n + x) as usize] = acc / win;
        }
    }
    let mut out = vec![0.0f32; src.len()];
    for y in 0..n {
        for x in 0..n {
            let mut acc = 0.0;
            for d in -radius..=radius {
                acc += tmp[((y + d).rem_euclid(n) * n + x) as usize];
            }
            out[(y * n + x) as usize] = acc / win;
        }
    }
    out
}

/// The three baked tiles (Cold, Rough, Hot) — lazy, deterministic, ~768 KB total.
fn tiles() -> &'static [Vec<f32>; 3] {
    static TILES: OnceLock<[Vec<f32>; 3]> = OnceLock::new();
    TILES.get_or_init(|| {
        [
            bake(&COLD, 0xC01D_0001),
            bake(&ROUGH, 0x0F0F_0002),
            bake(&HOT, 0x0707_0003),
        ]
    })
}

/// Sample a paper preset's tile at pattern coords `(u, v)` (1 unit = one tile = [`TILE`] texels, so
/// one texel ≈ one canvas px at Paper Size 1) — bilinear with seamless wrap. `which`: 0 Cold ·
/// 1 Rough · 2 Hot.
pub(super) fn sample(which: usize, u: f32, v: f32) -> f32 {
    let tile = &tiles()[which.min(2)];
    let (x, y) = (u * TILE as f32, v * TILE as f32);
    let x0 = x.floor();
    let y0 = y.floor();
    let (tx, ty) = (x - x0, y - y0);
    let (ix, iy) = (x0 as i64, y0 as i64);
    let w = |dx: i64, dy: i64| -> f32 {
        let px = (ix + dx).rem_euclid(TILE as i64) as usize;
        let py = (iy + dy).rem_euclid(TILE as i64) as usize;
        tile[py * TILE + px]
    };
    let (a, b, c, d) = (w(0, 0), w(1, 0), w(0, 1), w(1, 1));
    let top = a + (b - a) * tx;
    let bot = c + (d - c) * tx;
    top + (bot - top) * ty
}
