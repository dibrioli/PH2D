//! **Simetria e Tiled Mode** — o funil de TODA célula pintada: uma célula do caminho × o carimbo ×
//! as imagens da simetria × o ladrilho → as células do documento.
//!
//! O eixo vive em MEIAS células (`axis2 = 2 × posição`), logo pode ficar a meio pixel e o espelho é
//! exato no reticulado: a imagem de `x` pelo eixo vertical é `axis2 − 1 − x`. Com `axis2 = w` é o
//! `w − 1 − x` do MVP.

use crate::raster::Cell;

/// Que espelhos estão ligados (Aseprite 1.3.11: horizontal, vertical e as duas diagonais). As
/// imagens são o GRUPO que eles geram: H+V = 4, as duas diagonais = 4, H+diagonal = as 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mirrors {
    /// Espelho pelo eixo VERTICAL (esquerda ↔ direita) — o `x` do MVP.
    pub h: bool,
    /// Espelho pelo eixo HORIZONTAL (cima ↔ baixo) — o `y` do MVP.
    pub v: bool,
    /// A diagonal que desce para a direita (`\`, troca x e y à volta do centro).
    pub d1: bool,
    /// A diagonal que sobe para a direita (`/`).
    pub d2: bool,
}

impl Mirrors {
    #[must_use]
    pub const fn any(self) -> bool {
        self.h || self.v || self.d1 || self.d2
    }
}

/// A simetria de desenho: os espelhos e o centro, em meias células.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Symmetry {
    pub mirrors: Mirrors,
    /// `2 × x` do eixo vertical (o centro em x).
    pub axis2_x: i32,
    /// `2 × y` do eixo horizontal (o centro em y).
    pub axis2_y: i32,
}

impl Symmetry {
    /// Os eixos no meio de um documento `w × h` (a meio pixel quando o lado é ímpar).
    #[must_use]
    pub fn centered(mirrors: Mirrors, w: u32, h: u32) -> Self {
        Self {
            mirrors,
            axis2_x: i32::try_from(w).unwrap_or(i32::MAX),
            axis2_y: i32::try_from(h).unwrap_or(i32::MAX),
        }
    }
}

/// Que eixos dão a volta (Aseprite *Tiled Mode*: nenhum, x, y, ambos).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tiling {
    pub x: bool,
    pub y: bool,
}

/// Uma transformação inteira do reticulado: `p' = M·p + t`, `M` com entradas em `{-1, 0, 1}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Xf {
    m: [i32; 4],
    t: (i32, i32),
}

impl Xf {
    const ID: Self = Self {
        m: [1, 0, 0, 1],
        t: (0, 0),
    };

    fn apply(self, (x, y): Cell) -> Cell {
        (
            self.m[0] * x + self.m[1] * y + self.t.0,
            self.m[2] * x + self.m[3] * y + self.t.1,
        )
    }

    /// Só a parte linear: o offset de um carimbo REFLETE-SE, não se desloca (MVP W24).
    fn linear(self, (x, y): Cell) -> Cell {
        (self.m[0] * x + self.m[1] * y, self.m[2] * x + self.m[3] * y)
    }

    /// `self ∘ o` (primeiro `o`, depois `self`).
    fn after(self, o: Self) -> Self {
        let [a, b, c, d] = self.m;
        let [e, f, g, h] = o.m;
        Self {
            m: [a * e + b * g, a * f + b * h, c * e + d * g, c * f + d * h],
            t: self.apply(o.t),
        }
    }
}

/// As transformações das imagens: a identidade primeiro, depois na ordem do fecho (H, V, d1, d2) —
/// com H+V é a ordem do MVP (própria, X, Y, XY).
fn images(s: Symmetry) -> Vec<Xf> {
    let (ax, ay) = (s.axis2_x, s.axis2_y);
    let mut gens = Vec::new();
    if s.mirrors.h {
        gens.push(Xf {
            m: [-1, 0, 0, 1],
            t: (ax - 1, 0),
        });
    }
    if s.mirrors.v {
        gens.push(Xf {
            m: [1, 0, 0, -1],
            t: (0, ay - 1),
        });
    }
    if s.mirrors.d1 {
        let k = (ax - ay).div_euclid(2);
        gens.push(Xf {
            m: [0, 1, 1, 0],
            t: (k, -k),
        });
    }
    if s.mirrors.d2 {
        let m = (ax + ay).div_euclid(2) - 1;
        gens.push(Xf {
            m: [0, -1, -1, 0],
            t: (m, m),
        });
    }
    let mut out = vec![Xf::ID];
    let mut i = 0;
    while i < out.len() {
        for g in &gens {
            let n = g.after(out[i]);
            if !out.contains(&n) {
                out.push(n);
            }
        }
        i += 1;
    }
    out
}

/// As imagens da célula `(x, y)`: `cb(x', y', xf)`. Uma imagem que cai na MESMA célula que outra já
/// emitida não sai de novo (a célula sobre o eixo é a sua própria imagem).
fn each_image(s: Symmetry, cell: Cell, mut cb: impl FnMut(Cell, Xf)) {
    let mut seen: Vec<Cell> = Vec::with_capacity(8);
    for xf in images(s) {
        let p = xf.apply(cell);
        if seen.contains(&p) {
            continue;
        }
        seen.push(p);
        cb(p, xf);
    }
}

/// As imagens de uma célula (sem carimbo), com as reflexões que as produziram em `x`/`y` — a forma
/// que a fixtura do MVP compara (`flipX`, `flipY`).
#[must_use]
pub fn symmetry_images(s: Symmetry, cell: Cell) -> Vec<(Cell, bool, bool)> {
    let mut out = Vec::new();
    each_image(s, cell, |p, xf| out.push((p, xf.m[0] < 0, xf.m[3] < 0)));
    out
}

/// **O funil**: a célula do caminho `cell`, o carimbo `offs`, a simetria `s` e o ladrilho `tile`
/// num documento `w × h` → `out(x, y, k)`, `k` = a célula do carimbo. Fora dos limites, o eixo que
/// dá a volta dobra e o outro corta.
pub fn expand(
    cell: Cell,
    offs: &[Cell],
    s: Symmetry,
    tile: Tiling,
    (w, h): (u32, u32),
    mut out: impl FnMut(i32, i32, usize),
) {
    let (w, h) = (
        i32::try_from(w).unwrap_or(i32::MAX),
        i32::try_from(h).unwrap_or(i32::MAX),
    );
    each_image(s, cell, |(sx, sy), xf| {
        for (k, &o) in offs.iter().enumerate() {
            let (dx, dy) = xf.linear(o);
            let (mut x, mut y) = (sx + dx, sy + dy);
            if tile.x {
                x = x.rem_euclid(w);
            } else if x < 0 || x >= w {
                continue;
            }
            if tile.y {
                y = y.rem_euclid(h);
            } else if y < 0 || y >= h {
                continue;
            }
            out(x, y, k);
        }
    });
}
