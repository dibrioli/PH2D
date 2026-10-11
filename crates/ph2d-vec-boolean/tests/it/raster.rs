//! **Régua de pixels** dos gates de offset/booleana: rasteriza por AMOSTRA no centro do pixel
//! (sem AA), por varredura de linhas com winding — a mesma pergunta que um preenchimento faz.
//!
//! Duas máscaras amostradas na MESMA grelha só diferem onde as REGIÕES diferem: o erro de borda
//! do anti-aliasing não existe aqui, então `xor / área` mede geometria, não suavização.

use kurbo::{BezPath, PathEl, Point};
use ph2d_vec_scene::{FillRule, VecPath};

/// Uma grelha de `w × h` pixels quadrados de lado `px`, canto em `(x0, y0)`.
#[derive(Clone, Copy, Debug)]
pub struct Grid {
    pub x0: f64,
    pub y0: f64,
    pub px: f64,
    pub w: usize,
    pub h: usize,
}

impl Grid {
    /// A grelha de `n` pixels no lado maior que cobre `[min, max]` com uma margem de 2 pixels.
    pub fn covering(min: Point, max: Point, n: usize) -> Self {
        let span = (max.x - min.x).max(max.y - min.y).max(1e-9);
        let px = span / (n as f64 - 4.0);
        let w = ((max.x - min.x) / px).ceil() as usize + 4;
        let h = ((max.y - min.y) / px).ceil() as usize + 4;
        Self {
            x0: min.x - 2.0 * px,
            y0: min.y - 2.0 * px,
            px,
            w,
            h,
        }
    }

    pub fn len(&self) -> usize {
        self.w * self.h
    }

    /// Área de um pixel, em unidades do documento.
    pub fn px_area(&self) -> f64 {
        self.px * self.px
    }
}

/// Uma máscara amostrada: `true` = o centro do pixel está dentro.
pub type Mask = Vec<bool>;

/// Rasteriza um conjunto de polilinhas FECHADAS sob `rule`, numa máscara nova.
pub fn rasterize(polys: &[Vec<Point>], rule: FillRule, g: &Grid) -> Mask {
    let mut m = vec![false; g.len()];
    rasterize_into(polys, rule, g, &mut m);
    m
}

/// Rasteriza e faz OR em `m` (união com o que já lá está).
pub fn rasterize_into(polys: &[Vec<Point>], rule: FillRule, g: &Grid, m: &mut Mask) {
    let mut xs: Vec<(f64, i32)> = Vec::new();
    for row in 0..g.h {
        let y = g.y0 + (row as f64 + 0.5) * g.px;
        xs.clear();
        for poly in polys {
            let n = poly.len();
            for i in 0..n {
                let a = poly[i];
                let b = poly[(i + 1) % n];
                // Meia-aberta [ymin, ymax): cada vértice conta uma vez.
                let (dir, lo, hi) = if a.y < b.y { (1, a, b) } else { (-1, b, a) };
                if y >= lo.y && y < hi.y {
                    let t = (y - lo.y) / (hi.y - lo.y);
                    xs.push((lo.x + t * (hi.x - lo.x), dir));
                }
            }
        }
        xs.sort_by(|p, q| p.0.total_cmp(&q.0));
        let mut wind = 0;
        for k in 0..xs.len() {
            wind += xs[k].1;
            let inside = match rule {
                FillRule::NonZero => wind != 0,
                FillRule::EvenOdd => wind % 2 != 0,
            };
            if !inside || k + 1 == xs.len() {
                continue;
            }
            // Pixels cujo CENTRO cai em [xs[k], xs[k+1]).
            let c0 = ((xs[k].0 - g.x0) / g.px - 0.5).ceil().max(0.0) as usize;
            let c1 = ((xs[k + 1].0 - g.x0) / g.px - 0.5).ceil().max(0.0) as usize;
            let c1 = c1.min(g.w);
            for c in c0..c1 {
                m[row * g.w + c] = true;
            }
        }
    }
}

pub fn count(m: &Mask) -> usize {
    m.iter().filter(|&&b| b).count()
}

pub fn xor_count(a: &Mask, b: &Mask) -> usize {
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

/// Um `BezPath` achatado em polilinhas fechadas (um `Vec` por sub-caminho).
pub fn bez_polylines(bez: &BezPath, tol: f64) -> Vec<Vec<Point>> {
    let mut out: Vec<Vec<Point>> = Vec::new();
    let mut cur: Vec<Point> = Vec::new();
    kurbo::flatten(bez.iter(), tol, |el| match el {
        PathEl::MoveTo(p) => {
            if cur.len() >= 3 {
                out.push(std::mem::take(&mut cur));
            }
            cur.clear();
            cur.push(p);
        }
        PathEl::LineTo(p) => cur.push(p),
        PathEl::ClosePath => {
            if cur.len() >= 3 {
                out.push(std::mem::take(&mut cur));
            }
            cur.clear();
        }
        _ => {}
    });
    if cur.len() >= 3 {
        out.push(cur);
    }
    out
}

/// O `VecPath` COZIDO como `BezPath`, todos os contornos fechados (a região que ele preenche).
pub fn vecpath_bez(p: &VecPath) -> BezPath {
    let p = &*p.cooked();
    let mut bp = BezPath::new();
    let kp = |a: [f64; 2]| Point::new(a[0], a[1]);
    for c in 0..p.contour_count() {
        let Some((verts, _)) = p.contour(c) else {
            continue;
        };
        let Some(first) = verts.first() else {
            continue;
        };
        bp.move_to(kp(first.anchor));
        let n = verts.len();
        for i in 0..n {
            let (a, b) = (&verts[i], &verts[(i + 1) % n]);
            bp.curve_to(kp(a.out_handle), kp(b.in_handle), kp(b.anchor));
        }
        bp.close_path();
    }
    bp
}

/// As âncoras do contorno primário — para polígonos (todas as quinas são retas).
pub fn anchors(p: &VecPath) -> Vec<Point> {
    p.cooked()
        .verts
        .iter()
        .map(|v| Point::new(v.anchor[0], v.anchor[1]))
        .collect()
}

/// Rasteriza a UNIÃO de vários `VecPath` (cada um com a sua regra) — o que a tela mostra.
pub fn rasterize_paths(paths: &[VecPath], g: &Grid, tol: f64) -> Mask {
    let mut m = vec![false; g.len()];
    for p in paths {
        rasterize_into(&bez_polylines(&vecpath_bez(p), tol), p.fill_rule, g, &mut m);
    }
    m
}

/// Área com sinal (positiva = anti-horária num referencial y-para-cima).
pub fn signed_area(poly: &[Point]) -> f64 {
    let n = poly.len();
    (0..n)
        .map(|i| {
            let (a, b) = (poly[i], poly[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum::<f64>()
        * 0.5
}

/// Caixa envolvente de um conjunto de polilinhas.
pub fn bbox(polys: &[Vec<Point>]) -> Option<(Point, Point)> {
    let mut it = polys.iter().flatten();
    let f = *it.next()?;
    let (mut lo, mut hi) = (f, f);
    for p in it {
        lo = Point::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Point::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    Some((lo, hi))
}
