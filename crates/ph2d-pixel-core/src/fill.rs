//! **O balde** (Aseprite *Paint Bucket*): a região a pintar a partir de uma semente.
//!
//! Scanline com pilha EXPLÍCITA (nunca recursão: 4096² enche sem pilha do SO). 4-conectado por
//! omissão — o 8 vaza pela diagonal de um contorno de pixel art, que é como um pixel artist desenha
//! quinas (MVP).

use crate::compose::compose_rgba;
use crate::raster::Cell;
use crate::sprite::{CelKey, ColorMode, Sprite};

/// O que o balde compara.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BucketSample {
    /// Só a camada activa (*Active Layer*).
    #[default]
    Layer,
    /// O que se vê: todas as camadas visíveis compostas (*All Layers*).
    Visible,
}

/// As opções do balde (a barra de contexto do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bucket {
    /// `0..=255`: a maior diferença por canal RGBA que ainda é «a mesma cor». Em Indexed com 0 a
    /// pergunta é o ÍNDICE (dois índices da mesma cor são cores diferentes para o artista).
    pub tolerance: u8,
    /// `false` = troca TODAS as células da cor, ligadas ou não.
    pub contiguous: bool,
    /// 8-conectado (atravessa diagonais).
    pub eight: bool,
    pub sample: BucketSample,
}

impl Default for Bucket {
    fn default() -> Self {
        Self {
            tolerance: 0,
            contiguous: true,
            eight: false,
            sample: BucketSample::Layer,
        }
    }
}

/// **A varredura**: as células `(x, y)` de `w × h` ligadas à semente onde `same(x, y)` — `cb` em
/// qualquer ordem, cada célula uma vez. Semente fora = nada.
pub fn flood(
    w: u32,
    h: u32,
    seed: Cell,
    eight: bool,
    same: impl Fn(i32, i32) -> bool,
    mut cb: impl FnMut(i32, i32),
) {
    let (wi, hi) = (w as i32, h as i32);
    let (sx, sy) = seed;
    if sx < 0 || sy < 0 || sx >= wi || sy >= hi || !same(sx, sy) {
        return;
    }
    let mut seen = vec![false; w as usize * h as usize];
    let at = |x: i32, y: i32| y as usize * w as usize + x as usize;
    let mut stack = vec![seed];
    let k = i32::from(eight);
    while let Some((x0, y)) = stack.pop() {
        if seen[at(x0, y)] {
            continue;
        }
        let mut l = x0;
        while l > 0 && !seen[at(l - 1, y)] && same(l - 1, y) {
            l -= 1;
        }
        let mut r = x0;
        while r + 1 < wi && !seen[at(r + 1, y)] && same(r + 1, y) {
            r += 1;
        }
        for x in l..=r {
            seen[at(x, y)] = true;
            cb(x, y);
        }
        for ny in [y - 1, y + 1] {
            if ny < 0 || ny >= hi {
                continue;
            }
            let mut run = false;
            for x in (l - k).max(0)..=(r + k).min(wi - 1) {
                let ok = !seen[at(x, ny)] && same(x, ny);
                if ok && !run {
                    stack.push((x, ny));
                }
                run = ok;
            }
        }
    }
}

/// As células que o balde pinta no cel `key` a partir de `seed`.
#[must_use]
pub fn bucket_cells(sprite: &Sprite, key: CelKey, seed: Cell, opt: Bucket) -> Vec<Cell> {
    let (w, h) = (sprite.width(), sprite.height());
    if !sprite.bounds().contains(seed.0, seed.1) {
        return Vec::new();
    }
    let by_index = sprite.mode() == ColorMode::Indexed
        && opt.tolerance == 0
        && opt.sample == BucketSample::Layer;
    let mut composed = Vec::new();
    if opt.sample == BucketSample::Visible {
        compose_rgba(sprite, key.1, None, &mut composed);
    }
    let rgba = |x: i32, y: i32| -> [u8; 4] {
        let c = match opt.sample {
            BucketSample::Layer => sprite.rgba_of_value(sprite.value_at(key, x, y)),
            BucketSample::Visible => {
                let i = (y as usize * w as usize + x as usize) * 4;
                [
                    composed[i],
                    composed[i + 1],
                    composed[i + 2],
                    composed[i + 3],
                ]
            }
        };
        // Todo transparente é O transparente, seja qual for o RGB por baixo.
        if c[3] == 0 { [0; 4] } else { c }
    };
    let target_value = sprite.value_at(key, seed.0, seed.1);
    let target = rgba(seed.0, seed.1);
    let tol = opt.tolerance;
    let same = |x: i32, y: i32| {
        if by_index {
            return sprite.value_at(key, x, y) == target_value;
        }
        let c = rgba(x, y);
        (0..4).all(|i| c[i].abs_diff(target[i]) <= tol)
    };
    let mut out = Vec::new();
    if opt.contiguous {
        flood(w, h, seed, opt.eight, same, |x, y| out.push((x, y)));
    } else {
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                if same(x, y) {
                    out.push((x, y));
                }
            }
        }
    }
    out
}
