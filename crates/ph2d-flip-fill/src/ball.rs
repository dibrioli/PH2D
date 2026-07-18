//! **Trapped-ball** — o flood que não passa por um vão estreito.
//!
//! Clean-room de **Zhang et al.**, *"Vectorizing Cartoon Animations"* (TVCG 15(4),
//! 2009), §3.1: uma bola de raio `r` não atravessa um vão mais estreito que `2r`, então
//! inundar *com a bola* em vez de com um pixel fecha por construção os buracos que o
//! line-art à mão sempre tem. É a alternativa ao Gap Closure para o caso comum — em vez
//! de o artista achar o vão e calibrar um alcance, ele diz "a tinta é grossa assim".
//!
//! ## A mecânica, e por que ela é uma pergunta de DISTÂNCIA
//!
//! O paper descreve erosão → flood → dilatação, por raio, num laço best-first de raios
//! decrescentes. Implementado assim, cada raio custa uma rodada nova de morfologia.
//!
//! Mas "cabe uma bola de raio `r` centrada aqui?" é *literalmente* `dist(p, tinta) ≥ r`.
//! Com a EDT exata ([`crate::sq_distance_to_set`]) a resposta sai **de uma vez para
//! TODO raio** — a varredura de raios decrescentes vira um re-threshold do MESMO buffer.
//! E a dilatação de volta é a segunda EDT, a partir da componente inundada.
//!
//! **A região resultante nunca cruza a tinta, e isso é geometria, não um clamp:** ela é
//! a união de bolas centradas em pontos onde a bola *cabe* — e uma bola que cabe está,
//! inteira, no papel. Não há passo de recorte para esquecer.
//!
//! ## O que ele NÃO muda
//!
//! Nada, quando desligado: `trap_px = 0` deixa o `fill_at` no `Grid::flood` de sempre,
//! **byte a byte**. A wave inteira é opt-in (`docs/Flip/09_colorize.md`).

use crate::edt::sq_distance_to_set;
use crate::raster::{BOUNDARY, Grid};

/// A estrutura de alcance da bola: a EDT da tinta, calculada **uma vez** por fill.
///
/// Guardá-la é o que torna o laço de raios decrescentes do paper barato — os raios são
/// thresholds sobre este mesmo buffer, não rodadas novas de morfologia.
pub struct TrapBall {
    /// Distância ao QUADRADO de cada pixel à fronteira mais próxima.
    sq_dist: Vec<u32>,
    w: usize,
    h: usize,
}

impl TrapBall {
    /// Constrói o alcance a partir das fronteiras já rasterizadas no `grid`.
    #[must_use]
    pub fn new(grid: &Grid) -> Self {
        let sq_dist = sq_distance_to_set(grid.w, grid.h, |i| grid.flags[i] & BOUNDARY != 0);
        Self {
            sq_dist,
            w: grid.w,
            h: grid.h,
        }
    }

    /// Cabe uma bola de raio `r_px` centrada no pixel `i`?
    ///
    /// A comparação é em `f64` porque `r_px` é fracionário e as distâncias são
    /// inteiras: `u32::MAX` (o "infinitamente longe" de um grid sem tinta) é exato em
    /// `f64`, e nenhum raio plausível chega perto da faixa onde ele deixaria de ser.
    #[must_use]
    pub fn fits(&self, i: usize, r_px: f32) -> bool {
        let r = f64::from(r_px.max(0.0));
        f64::from(self.sq_dist[i]) >= r * r
    }

    /// **A região que a bola de raio `r_px` alcança a partir de `seed`.**
    ///
    /// Devolve a máscara da região, ou `None` quando a bola **não cabe na semente** —
    /// que é a resposta honesta, não um erro: significa "aqui é mais estreito que a
    /// bola", e quem chama decide (o laço best-first do paper baixa o raio).
    ///
    /// O `escaped` de saída diz se a região tocou a borda da grade — o mesmo critério
    /// de vazamento do [`Grid::flood`], que o chamador vira `FillError::Leaked`.
    #[must_use]
    pub fn region_from(&self, seed: (usize, usize), r_px: f32) -> Option<TrapRegion> {
        let (core, escaped) = self.core_mask(seed, r_px)?;
        // 2. Dilata de volta: a região é a união das bolas centradas em `core` — ou
        //    seja, tudo a distância ≤ r do núcleo. Segunda EDT, e por construção o
        //    resultado está contido no papel (uma bola que cabe não invade a tinta).
        //
        //    ⚠️ **Só dentro da bbox do núcleo, folgada de `r`.** Um pixel a mais de `r`
        //    do núcleo está fora da região por definição, então varrer a grade inteira
        //    aqui é trabalho que a resposta não usa — e ele custava tanto quanto a EDT
        //    do campo de alcance, que essa sim precisa ser global (medição na régua
        //    `measure_the_product_grid_and_ball_cost`). O resultado é o MESMO, ao bit:
        //    é uma janela, não uma aproximação.
        let r = f64::from(r_px.max(0.0));
        let rr = r * r;
        let pad = r_px.max(0.0).ceil() as usize + 1;
        let Some((x0, y0, x1, y1)) = self.core_bounds(&core, pad) else {
            // Núcleo vazio não acontece (a semente está nele), mas se acontecesse a
            // resposta honesta é "região vazia", nunca uma máscara de grade inteira.
            return Some(TrapRegion {
                mask: vec![false; self.w * self.h],
                escaped,
            });
        };
        let (bw, bh) = (x1 - x0 + 1, y1 - y0 + 1);
        let local = sq_distance_to_set(bw, bh, |i| {
            let (lx, ly) = (i % bw, i / bw);
            core[(y0 + ly) * self.w + (x0 + lx)]
        });
        let mut mask = vec![false; self.w * self.h];
        for ly in 0..bh {
            for lx in 0..bw {
                if f64::from(local[ly * bw + lx]) <= rr {
                    mask[(y0 + ly) * self.w + (x0 + lx)] = true;
                }
            }
        }
        Some(TrapRegion { mask, escaped })
    }
}

impl TrapBall {
    /// **O núcleo `E_r`**: os centros válidos alcançáveis a partir de `seed`, e se essa
    /// inundação tocou a borda da grade.
    ///
    /// Inunda 4-conexo **onde a bola cabe**. Um vão mais estreito que `2r` simplesmente
    /// não tem centro válido, então a inundação não tem por onde passar — é aqui que o
    /// trapped-ball acontece.
    ///
    /// Porta ÚNICA: o `region_from` e o gate que confere a janela da dilatação leem
    /// deste mesmo lugar. Um núcleo reconstruído no teste seria uma 2ª resposta para
    /// "o que a bola alcança", e as duas divergiriam sem ninguém notar.
    fn core_mask(&self, seed: (usize, usize), r_px: f32) -> Option<(Vec<bool>, bool)> {
        let (sx, sy) = seed;
        if sx >= self.w || sy >= self.h {
            return None;
        }
        let start = sy * self.w + sx;
        if !self.fits(start, r_px) {
            return None;
        }
        let mut core = vec![false; self.w * self.h];
        let mut stack = vec![start];
        core[start] = true;
        let mut escaped = false;
        while let Some(i) = stack.pop() {
            let (x, y) = (i % self.w, i / self.w);
            if x == 0 || y == 0 || x + 1 == self.w || y + 1 == self.h {
                escaped = true;
            }
            let push = |nx: usize, ny: usize, stack: &mut Vec<usize>, core: &mut Vec<bool>| {
                let n = ny * self.w + nx;
                if !core[n] && self.fits(n, r_px) {
                    core[n] = true;
                    stack.push(n);
                }
            };
            if x > 0 {
                push(x - 1, y, &mut stack, &mut core);
            }
            if x + 1 < self.w {
                push(x + 1, y, &mut stack, &mut core);
            }
            if y > 0 {
                push(x, y - 1, &mut stack, &mut core);
            }
            if y + 1 < self.h {
                push(x, y + 1, &mut stack, &mut core);
            }
        }
        Some((core, escaped))
    }

    /// O mesmo núcleo, para o gate que confere que a **janela** da dilatação não muda a
    /// resposta. Existe só no build de teste — a produção não tem por que expor isto.
    #[cfg(test)]
    fn core_mask_for_reference(&self, seed: (usize, usize), r_px: f32) -> Option<Vec<bool>> {
        self.core_mask(seed, r_px).map(|(c, _)| c)
    }

    /// Bbox do núcleo, folgada de `pad` e recortada na grade. `None` se o núcleo é
    /// vazio.
    fn core_bounds(&self, core: &[bool], pad: usize) -> Option<(usize, usize, usize, usize)> {
        let (mut x0, mut y0) = (usize::MAX, usize::MAX);
        let (mut x1, mut y1) = (0usize, 0usize);
        for (i, &c) in core.iter().enumerate() {
            if !c {
                continue;
            }
            let (x, y) = (i % self.w, i / self.w);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        if x0 == usize::MAX {
            return None;
        }
        Some((
            x0.saturating_sub(pad),
            y0.saturating_sub(pad),
            (x1 + pad).min(self.w - 1),
            (y1 + pad).min(self.h - 1),
        ))
    }
}

/// O que a bola alcançou, e se isso tocou a borda da grade.
pub struct TrapRegion {
    /// Máscara da região, indexada como o `Grid` (`y·w + x`).
    pub mask: Vec<bool>,
    /// A região tocou a borda da grade — o mesmo "vazou" do [`Grid::flood`].
    pub escaped: bool,
}

#[cfg(test)]
#[path = "ball_tests.rs"]
mod tests;
