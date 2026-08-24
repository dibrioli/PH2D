//! **Bridson 2007** — *Fast Poisson Disk Sampling in Arbitrary Dimensions* (SIGGRAPH
//! sketch): dart-throwing with a background grid, `O(N)`.
//!
//! ```text
//! 1. cell = r/√2 — small enough that a cell can hold AT MOST ONE sample, which is what
//!    turns the "is anything within r?" question into a fixed 5×5 cell scan.
//! 2. Seed one point; it is the first ACTIVE point.
//! 3. While the active list is not empty: pick an active point at random, throw K darts
//!    into the annulus [r, 2r] around it, and keep the first that lands in bounds and no
//!    closer than r to any point already placed. If all K miss, that point is DONE
//!    (retire it from the active list — its neighbourhood is full).
//! ```
//!
//! Every iteration either places a point or retires an active one, and both are bounded
//! by the number of cells, so the loop always terminates.
//!
//! ## Two deliberate departures from the sketch
//!
//! **The dart's direction is rejection-sampled from the unit disc, not polar.** The
//! sketch says "pick a random angle", which means `sin`/`cos` — banned (HR-5). Drawing a
//! point in the square and normalising it would bias the direction toward the diagonals
//! (the corners are farther out), so the square sample is *rejected* until it lands
//! inside the unit disc: uniform direction, arithmetic and `sqrt` only.
//!
//! **The dart's radius is uniform by AREA, not uniform in `[r, 2r]`.** The sketch draws
//! the radius uniformly, which crowds the darts onto the inner ring (a thin annulus at
//! radius ρ has area ∝ ρ, so uniform-in-ρ over-samples small ρ). The area-uniform draw
//! `ρ = √(r² + u·3r²)` is the accepted correction and spends fewer darts on the region
//! that is most likely to be already occupied.

use crate::hash::Draws;
use ph2d_motion_region::Region;

/// Sorteios da SEMENTE numa forma recortada antes de o nó desistir. A aceitação é a
/// razão de áreas (um disco na sua caixa aceita `π/4 ≈ 79%`, um anel de buraco `0,98`
/// aceita `~3%`), e `64` leva a probabilidade de falhar todos abaixo de `1e-9` no pior
/// anel que o [`Region`] deixa construir.
const SEED_TRIES: u32 = 64;

/// Bridson's cell size is `r/√2` — see the module doc.
const SQRT2: f32 = std::f32::consts::SQRT_2;

/// Darts thrown at an active point before it is retired. Bridson's `k = 30`.
const K: u32 = 30;

/// The ceiling on the background grid — and therefore on everything else, because a
/// Bridson cell holds at most one point: **bounding the cells bounds the memory AND the
/// count.** This is what a node with no `count` param needs instead of one; a radius
/// typed as `0` must not ask for an infinite grid.
const MAX_CELLS: usize = 1 << 18;

/// An absolute floor under the radius, for a rectangle so small that the cell budget
/// alone would still allow a near-zero one (a zero radius divides by zero and asks for a
/// grid of `usize::MAX` cells — the saturating `f32 as usize` cast makes that an
/// allocation, not a panic).
const MIN_RADIUS: f32 = 1e-4;

/// Rejection tries for a uniform direction. Acceptance is `π/4 ≈ 79%`, so all eight
/// missing has probability `~5e-6`; that dart then flies along `+X`, which is a valid
/// (merely not uniformly-chosen) direction — the invariant the node promises (no two
/// points closer than `r`) is checked afterwards regardless.
const DIR_TRIES: u32 = 8;

/// The smallest radius this rectangle can afford, given the cell budget.
///
/// The grid is `(w/cell) × (h/cell)` cells with `cell = r/√2`, i.e. `2·w·h/r²` of them,
/// so `r ≥ √(2·w·h / MAX_CELLS)`.
fn clamp_radius(w: f32, h: f32, radius: f32) -> f32 {
    let floor = (2.0 * w * h / MAX_CELLS as f32).sqrt();
    radius.max(floor).max(MIN_RADIUS)
}

/// A dart from `center`: uniform direction × area-uniform radius in the annulus `[r, 2r]`.
fn dart(d: &mut Draws, center: [f32; 2], r: f32) -> [f32; 2] {
    let mut dir = [1.0, 0.0];
    for _ in 0..DIR_TRIES {
        let x = d.next() * 2.0 - 1.0;
        let y = d.next() * 2.0 - 1.0;
        let len_sq = x * x + y * y;
        if len_sq > 1e-8 && len_sq <= 1.0 {
            let len = len_sq.sqrt();
            dir = [x / len, y / len];
            break;
        }
    }
    let rho = (r * r + d.next() * 3.0 * r * r).sqrt();
    [center[0] + dir[0] * rho, center[1] + dir[1] * rho]
}

/// The grid cell a point falls in.
fn cell_of(p: [f32; 2], cell: f32, gw: usize, gh: usize) -> (usize, usize) {
    let cx = ((p[0] / cell) as usize).min(gw - 1);
    let cy = ((p[1] / cell) as usize).min(gh - 1);
    (cx, cy)
}

/// **A GRADE DE FUNDO do Bridson** — as células e o que já foi colocado.
///
/// ⚠️ Agrupada por ser um conceito, e não para caber num teto: o `cell`, o `gw`, o `gh`
/// e o `grid` são a MESMA estrutura (o índice espacial que torna a pergunta *«há alguma
/// coisa a menos de `r`?»* uma varredura de bloco fixo), e passá-los soltos deixava um
/// deles poder chegar de outro sítio.
#[derive(Copy, Clone)]
struct Bg<'a> {
    grid: &'a [u32],
    pts: &'a [[f32; 2]],
    cell: f32,
    gw: usize,
    gh: usize,
}

/// **The invariant, checked**: is `p` at least `r` from every point already placed?
///
/// A Bridson cell is `r/√2` across, so anything within `r` of `p` lies in the 5×5 block
/// of cells around it — a fixed scan, which is where the `O(N)` comes from.
///
/// ## ⚠️ Com raio VARIÁVEL o `span` cresce, e o teste é o MÁXIMO dos dois
///
/// A densidade graduada faz de `r` uma função do ponto (`r = r_base/densidade`), e aí
/// duas coisas mudam. **A varredura** deixa de ser 5×5: o vizinho mais distante que
/// ainda pode conflitar está a `r_max`, logo o bloco é `⌈r_max/cell⌉` células de cada
/// lado — e é por isso que a densidade tem um PISO (`MIN_DENSITY`), que é o que torna
/// esse número finito. **E o teste** passa a ser `dist < max(r(p), r(q))`: um ponto
/// grosso e um fino em conflito têm de concordar sobre quem manda, senão a relação
/// deixa de ser simétrica e a ordem de colocação muda o resultado.
///
/// ⚠️ Sem gradação (`adaptive = None`) este é o corpo de sempre, **verbatim** — o
/// `span` volta a `2` e `r_sq` a `r·r`.
fn far_enough(bg: &Bg<'_>, p: [f32; 2], r: f32, adaptive: Option<&Adaptive<'_>>) -> bool {
    let Bg {
        grid,
        pts,
        cell,
        gw,
        gh,
    } = *bg;
    let (cx, cy) = cell_of(p, cell, gw, gh);
    let span = adaptive.map_or(2, Adaptive::span);
    let lo_x = cx.saturating_sub(span);
    let lo_y = cy.saturating_sub(span);
    let hi_x = (cx + span).min(gw - 1);
    let hi_y = (cy + span).min(gh - 1);
    let r_sq = r * r;
    for y in lo_y..=hi_y {
        for x in lo_x..=hi_x {
            let at = grid[y * gw + x];
            if at == u32::MAX {
                continue;
            }
            let q = pts[at as usize];
            let (dx, dy) = (p[0] - q[0], p[1] - q[1]);
            let bar = match adaptive {
                None => r_sq,
                Some(a) => {
                    let m = r.max(a.radius(q));
                    m * m
                }
            };
            if dx * dx + dy * dy < bar {
                return false;
            }
        }
    }
    true
}

/// **A densidade graduada, vista pelo dardo** — o raio local e o quanto ele obriga a
/// varredura a crescer.
///
/// ⚠️ **Ela guarda a meia-extensão** porque este algoritmo autora em `[0,w)×[0,h)` e
/// só re-centra no fim, enquanto a [`Region`] fala sempre em coordenadas centradas na
/// origem. *Uma das duas tem de converter, e é quem sabe das duas convenções.*
struct Adaptive<'a> {
    region: &'a Region,
    falloff: f32,
    base: f32,
    hw: f32,
    hh: f32,
}

impl Adaptive<'_> {
    /// O raio local em `p` (coordenadas de autoria).
    fn radius(&self, p: [f32; 2]) -> f32 {
        self.base
            / self
                .region
                .density([p[0] - self.hw, p[1] - self.hh], self.falloff)
    }

    /// Quantas células de cada lado a varredura precisa. `r_max = base/MIN_DENSITY` e a
    /// célula é `base/√2`, então o bloco é `⌈√2/MIN_DENSITY⌉` — **8** com o piso de hoje.
    fn span(&self) -> usize {
        (SQRT2 / ph2d_motion_region::MIN_DENSITY).ceil() as usize
    }
}

/// Fill `region` (centred on the origin, bounded by the `w × h` box) with points no two
/// of which are closer than `radius`. The count is **implicit** — it is whatever the
/// spacing allows, which is the whole difference between this node and `motion.scatter`.
///
/// ## ⭐⭐ A densidade aqui é uma CAPACIDADE, não ergonomia
///
/// A célula da folha 01 dava a cadeia `poisson → field.remap(probability) → cull` como
/// exprimindo isto, *"e correto por construção: o cull só REMOVE, então o piso de
/// distância mínima sobrevive"*. **A primeira metade é verdade e a segunda é o
/// problema.** Sortear quem morre deixa os sobreviventes no espaçamento original: a
/// zona rala fica com **buracos**, não com um azul mais grosso. O que a referência
/// (*Distribute Points on Faces*, `Density Max` × campo) dá é a outra coisa —
/// **espaçamento maior**, com cada ponto ainda maximamente empacotado para o raio
/// local. Culling não consegue produzir esse conjunto, porque ele nunca MOVE um ponto.
///
/// ⇒ o raio passa a ser `r_base / densidade(p)`, e é isso que um Poisson adaptativo é.
pub(crate) fn sample(
    region: &Region,
    w: f32,
    h: f32,
    radius: f32,
    falloff: f32,
    seed: u32,
) -> Vec<[f32; 2]> {
    if !w.is_finite() || !h.is_finite() || !radius.is_finite() || w <= 0.0 || h <= 0.0 {
        return Vec::new();
    }
    let r = clamp_radius(w, h, radius);
    let cell = r / SQRT2;
    let gw = (w / cell).ceil().max(1.0) as usize;
    let gh = (h / cell).ceil().max(1.0) as usize;
    // The clamp above already bounds this; the guard is what makes that a fact rather
    // than an argument (a `ceil` of a huge float saturates the cast instead of wrapping).
    if gw.saturating_mul(gh) > MAX_CELLS.saturating_mul(2) {
        return Vec::new();
    }
    let mut grid = vec![u32::MAX; gw * gh];
    let mut pts: Vec<[f32; 2]> = Vec::new();
    let mut active: Vec<u32> = Vec::new();
    let mut d = Draws { seed, n: 0 };

    // Author in [0,w)×[0,h), hand back centred on the origin (the convention every
    // distribution here shares: the rectangle is around where you dropped the node).
    let (hw, hh) = (w * 0.5, h * 0.5);
    let adaptive = (falloff > 0.0).then_some(Adaptive {
        region,
        falloff,
        base: r,
        hw,
        hh,
    });
    // ⚠️ **A caixa é o caminho de sempre, e ele não ganha uma pergunta a mais.** Um
    // `region.contains` incondicional daria a MESMA resposta num `Rect` e ainda assim
    // mudaria o resultado, porque o ponto-semente abaixo passa a poder ser rejeitado —
    // e uma rejeição a mais desloca toda a sequência de sorteios que vem depois.
    let boxed = region.is_rect();
    let inside = |p: [f32; 2]| {
        p[0] >= 0.0
            && p[0] < w
            && p[1] >= 0.0
            && p[1] < h
            && (boxed || region.contains([p[0] - hw, p[1] - hh]))
    };

    let place = |p: [f32; 2], grid: &mut [u32], pts: &mut Vec<[f32; 2]>, active: &mut Vec<u32>| {
        let (cx, cy) = cell_of(p, cell, gw, gh);
        grid[cy * gw + cx] = pts.len() as u32;
        active.push(pts.len() as u32);
        pts.push(p);
    };

    // A semente. Numa caixa o primeiro sorteio serve sempre; numa forma recortada ele
    // pode cair fora, e aí re-sorteia-se um número LIMITADO de vezes — a aceitação de
    // um anel fino é pequena, mas nunca zero, e desistir devolve o conjunto vazio em
    // vez de rodar para sempre.
    let mut seedp = [d.next() * w, d.next() * h];
    if !boxed {
        let mut tries = 0;
        while !inside(seedp) && tries < SEED_TRIES {
            seedp = [d.next() * w, d.next() * h];
            tries += 1;
        }
        if !inside(seedp) {
            return Vec::new();
        }
    }
    place(seedp, &mut grid, &mut pts, &mut active);

    while !active.is_empty() && pts.len() < MAX_CELLS {
        // Bridson picks the active point at RANDOM (not FIFO): the front grows in every
        // direction at once instead of sweeping across the rectangle in a wave.
        let a = ((d.next() * active.len() as f32) as usize).min(active.len() - 1);
        let center = pts[active[a] as usize];
        // O anel de dardos usa o raio LOCAL do ponto ativo: numa zona rala ele atira
        // mais longe, que é como o espaçamento maior de facto acontece.
        let local = adaptive.as_ref().map_or(r, |ad| ad.radius(center));
        let mut landed = false;
        for _ in 0..K {
            let c = dart(&mut d, center, local);
            if !inside(c) {
                continue;
            }
            let bar = adaptive.as_ref().map_or(r, |ad| ad.radius(c));
            let bg = Bg {
                grid: &grid,
                pts: &pts,
                cell,
                gw,
                gh,
            };
            if far_enough(&bg, c, bar, adaptive.as_ref()) {
                place(c, &mut grid, &mut pts, &mut active);
                landed = true;
                break;
            }
        }
        if !landed {
            active.swap_remove(a); // its neighbourhood is full — retire it
        }
    }

    pts.iter().map(|p| [p[0] - hw, p[1] - hh]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O retangulo de sempre — o que estes gates mediam antes de a regiao existir.
    fn rect(w: f32, h: f32) -> Region {
        Region::of(0.0, w, h, 0.0)
    }

    /// The closest pair in the set. `f32::MAX` for fewer than two points.
    fn min_gap(pts: &[[f32; 2]]) -> f32 {
        let mut best = f32::MAX;
        for (i, p) in pts.iter().enumerate() {
            for q in &pts[i + 1..] {
                let (dx, dy) = (p[0] - q[0], p[1] - q[1]);
                best = best.min((dx * dx + dy * dy).sqrt());
            }
        }
        best
    }

    /// **The one promise the node makes.** Everything else is a consequence.
    #[test]
    fn no_two_points_are_closer_than_the_radius() {
        for seed in 0..6u32 {
            let pts = sample(&rect(4.0, 3.0), 4.0, 3.0, 0.3, 0.0, seed);
            assert!(pts.len() > 10, "seed {seed} produced almost nothing");
            let gap = min_gap(&pts);
            assert!(
                gap >= 0.3 - 1e-4,
                "seed {seed}: two points {gap} apart, closer than the 0.3 radius"
            );
        }
    }

    /// …and it does not buy that promise by placing three points and giving up: the
    /// disc packs the rectangle. The theoretical maximum for radius `r` is the hexagonal
    /// packing `area / (r²·√3/2)`; Bridson reaches ~65-75% of it, and a broken
    /// neighbourhood check (one that rejects everything) would sit at 1 point.
    #[test]
    fn it_actually_fills_the_rectangle() {
        let (w, h, r) = (4.0f32, 3.0f32, 0.3f32);
        let pts = sample(&rect(w, h), w, h, r, 0.0, 1);
        let hex_max = (w * h) / (r * r * 0.866);
        let ratio = pts.len() as f32 / hex_max;
        assert!(
            (0.5..=1.0).contains(&ratio),
            "packed {} points, {ratio:.2} of the hexagonal maximum {hex_max:.0}",
            pts.len()
        );
        // Every point is inside the centred rectangle it was asked for.
        for p in &pts {
            assert!(
                p[0] >= -w * 0.5 && p[0] <= w * 0.5,
                "x out of bounds: {p:?}"
            );
            assert!(
                p[1] >= -h * 0.5 && p[1] <= h * 0.5,
                "y out of bounds: {p:?}"
            );
        }
    }

    /// **The radius is the knob, the count is the answer** — the inverse-square law of
    /// the family, and the thing that makes this node different from `motion.scatter`.
    #[test]
    fn halving_the_radius_roughly_quadruples_the_count() {
        let coarse = sample(&rect(4.0, 4.0), 4.0, 4.0, 0.4, 0.0, 1).len();
        let fine = sample(&rect(4.0, 4.0), 4.0, 4.0, 0.2, 0.0, 1).len();
        let factor = fine as f32 / coarse as f32;
        assert!(
            (3.0..5.0).contains(&factor),
            "half the radius gave {factor:.2}x the points ({coarse} -> {fine}), not ~4x"
        );
    }

    /// Pure function of the seed: a scrub, a re-cook or another machine redraws the
    /// exact same layout — and another seed redraws a different one.
    #[test]
    fn the_layout_is_a_pure_function_of_the_seed() {
        assert_eq!(
            sample(&rect(4.0, 3.0), 4.0, 3.0, 0.3, 0.0, 5),
            sample(&rect(4.0, 3.0), 4.0, 3.0, 0.3, 0.0, 5)
        );
        assert_ne!(
            sample(&rect(4.0, 3.0), 4.0, 3.0, 0.3, 0.0, 5),
            sample(&rect(4.0, 3.0), 4.0, 3.0, 0.3, 0.0, 6)
        );
    }

    /// **A radius of zero must not hang the app, and must not allocate the world.** A
    /// count-less distribution has no `param_as_count` to hide behind: the *radius* is
    /// the allocation vector, so it is the radius that gets clamped.
    #[test]
    fn a_pathological_radius_is_bounded_not_fatal() {
        for r in [0.0, -1.0, f32::NAN, f32::INFINITY, 1e-30] {
            let pts = sample(&rect(4.0, 4.0), 4.0, 4.0, r, 0.0, 1);
            assert!(
                pts.len() <= MAX_CELLS,
                "radius {r} produced {} points",
                pts.len()
            );
        }
        // A degenerate rectangle is empty, not a panic.
        assert!(sample(&rect(0.0, 4.0), 0.0, 4.0, 0.3, 0.0, 1).is_empty());
        assert!(sample(&rect(f32::NAN, 4.0), f32::NAN, 4.0, 0.3, 0.0, 1).is_empty());
        // A radius larger than the rectangle fits exactly one point (the seed dart).
        assert_eq!(sample(&rect(1.0, 1.0), 1.0, 1.0, 10.0, 0.0, 1).len(), 1);
    }
}
