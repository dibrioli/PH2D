//! ⭐ W23 B12 — **quanto de um elemento o laço cobre** (`docs/MiroClone/17_bloco6_plano.md` §2.1): a
//! fracção da ÁREA do contorno real (formas, notas, imagens, frames) ou do COMPRIMENTO (tinta, setas)
//! que fica dentro do laço, sempre fechado, pela regra non-zero (a do Excalidraw: o laço em 8 conta
//! as duas voltas).
//!
//! A área é EXACTA para polígonos, por faixas verticais: corta-se em todos os x de vértices e de
//! cruzamentos; dentro de uma faixa nenhuma aresta cruza outra, cada fronteira é linear em x e o
//! comprimento da intersecção também — o ponto médio da faixa dá o integral sem erro. As curvas
//! entram achatadas a [`FLAT`] da diagonal do elemento. ⛔ Grelha de amostras: cai num conjunto de
//! medida nula e diz «limpo».

use ph2d_board_model::Element;
use ph2d_vector::{BezPath, PathEl, Point, Shape as _, flatten};

/// Tolerância do achatamento das curvas, em fracção da diagonal do elemento.
pub const FLAT: f64 = 1e-3;

type Edge = ([f64; 2], [f64; 2]);

/// Os lados de um caminho achatado: cada sub-caminho fecha-se (a área e o laço são regiões).
#[must_use]
pub fn polygons(path: &BezPath, tolerance: f64) -> Vec<Vec<[f64; 2]>> {
    let mut out: Vec<Vec<[f64; 2]>> = Vec::new();
    flatten(path.iter(), tolerance, |el| match el {
        PathEl::MoveTo(p) => out.push(vec![[p.x, p.y]]),
        PathEl::LineTo(p) => {
            if let Some(last) = out.last_mut() {
                last.push([p.x, p.y]);
            }
        }
        PathEl::ClosePath => {}
        _ => {}
    });
    out.retain(|p| p.len() >= 3);
    out
}

fn edges_of(polys: &[Vec<[f64; 2]>]) -> Vec<Edge> {
    polys
        .iter()
        .flat_map(|p| (0..p.len()).map(move |i| (p[i], p[(i + 1) % p.len()])))
        .filter(|(a, b)| a != b)
        .collect()
}

/// Onde `x = c` atravessa as arestas (meio-aberto, um vértice conta uma vez): `(y, sentido)`.
fn crossings(edges: &[Edge], x: f64) -> Vec<(f64, i32)> {
    let mut v: Vec<(f64, i32)> = edges
        .iter()
        .filter(|(a, b)| (a[0] <= x) != (b[0] <= x))
        .map(|(a, b)| {
            let t = (x - a[0]) / (b[0] - a[0]);
            (a[1] + t * (b[1] - a[1]), if b[0] > a[0] { 1 } else { -1 })
        })
        .collect();
    v.sort_by(|p, q| p.0.total_cmp(&q.0));
    v
}

/// Os intervalos de `y` onde a recta `x` está DENTRO (non-zero) da região das arestas.
fn inside_spans(edges: &[Edge], x: f64) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    let mut w = 0;
    let mut from = 0.0;
    for (y, d) in crossings(edges, x) {
        let was = w != 0;
        w += d;
        match (was, w != 0) {
            (false, true) => from = y,
            (true, false) => out.push([from, y]),
            _ => {}
        }
    }
    out
}

fn overlap(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    let (mut i, mut j, mut s) = (0, 0, 0.0);
    while i < a.len() && j < b.len() {
        let lo = a[i][0].max(b[j][0]);
        let hi = a[i][1].min(b[j][1]);
        s += (hi - lo).max(0.0);
        if a[i][1] < b[j][1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    s
}

/// O `x` onde os segmentos `p` e `q` se cruzam (no interior de ambos), se cruzam.
fn cross_x(p: Edge, q: Edge) -> Option<f64> {
    let (a, b, c, d) = (p.0, p.1, q.0, q.1);
    let r = [b[0] - a[0], b[1] - a[1]];
    let s = [d[0] - c[0], d[1] - c[1]];
    let den = r[0] * s[1] - r[1] * s[0];
    if den.abs() < 1e-300 {
        return None;
    }
    let ac = [c[0] - a[0], c[1] - a[1]];
    let t = (ac[0] * s[1] - ac[1] * s[0]) / den;
    let u = (ac[0] * r[1] - ac[1] * r[0]) / den;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then(|| a[0] + t * r[0])
}

fn span_x(e: &Edge) -> [f64; 2] {
    [e.0[0].min(e.1[0]), e.0[0].max(e.1[0])]
}

fn boxes_meet(p: &Edge, q: &Edge) -> bool {
    let (px, qx) = (span_x(p), span_x(q));
    let py = [p.0[1].min(p.1[1]), p.0[1].max(p.1[1])];
    let qy = [q.0[1].min(q.1[1]), q.0[1].max(q.1[1])];
    px[0] <= qx[1] && qx[0] <= px[1] && py[0] <= qy[1] && qy[0] <= py[1]
}

/// ⭐ A fracção da ÁREA da região `region` (polígonos, non-zero) dentro do laço `lasso` (fechado,
/// non-zero). `0` para uma região sem área.
#[must_use]
pub fn area_cover(region: &[Vec<[f64; 2]>], lasso: &[[f64; 2]]) -> f64 {
    if lasso.len() < 3 {
        return 0.0;
    }
    area_cover_edges(region, &edges_of(&[lasso.to_vec()]))
}

/// [`area_cover`] com as arestas do laço já feitas (basta as que tocam a faixa de x da região).
fn area_cover_edges(region: &[Vec<[f64; 2]>], lasso: &[Edge]) -> f64 {
    let re = edges_of(region);
    if re.is_empty() {
        return 0.0;
    }
    let [x0, x1] = re
        .iter()
        .map(span_x)
        .fold([f64::INFINITY, f64::NEG_INFINITY], |m, s| {
            [m[0].min(s[0]), m[1].max(s[1])]
        });
    let le: Vec<Edge> = lasso
        .iter()
        .filter(|e| {
            let s = span_x(e);
            s[0] <= x1 && s[1] >= x0
        })
        .copied()
        .collect();
    let all: Vec<Edge> = re.iter().chain(&le).copied().collect();
    let mut xs: Vec<f64> = all.iter().flat_map(|e| [e.0[0], e.1[0]]).collect();
    for (i, p) in all.iter().enumerate() {
        for q in &all[i + 1..] {
            if boxes_meet(p, q)
                && let Some(x) = cross_x(*p, *q)
            {
                xs.push(x);
            }
        }
    }
    xs.retain(|x| (x0..=x1).contains(x));
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    let (mut inside, mut total) = (0.0, 0.0);
    for w in xs.windows(2) {
        let dx = w[1] - w[0];
        if dx <= 0.0 {
            continue;
        }
        let xm = (w[0] + w[1]) / 2.0;
        let r = inside_spans(&re, xm);
        let l = inside_spans(&le, xm);
        total += r.iter().map(|s| s[1] - s[0]).sum::<f64>() * dx;
        inside += overlap(&r, &l) * dx;
    }
    if total <= 0.0 { 0.0 } else { inside / total }
}

/// `p` está dentro do laço (non-zero)?
#[must_use]
pub fn in_lasso(lasso: &[[f64; 2]], p: [f64; 2]) -> bool {
    in_edges(&edges_of(&[lasso.to_vec()]), p)
}

fn in_edges(e: &[Edge], p: [f64; 2]) -> bool {
    let mut w = 0;
    for (y, d) in crossings(e, p[0]) {
        if y > p[1] {
            break;
        }
        w += d;
    }
    w != 0
}

/// ⭐ A fracção do COMPRIMENTO das linhas `lines` (abertas) dentro do laço: cada segmento parte-se
/// onde cruza o laço e cada pedaço conta pelo seu meio. Linhas de comprimento zero (um ponto) contam
/// pelo ponto.
#[must_use]
pub fn length_cover(lines: &[Vec<[f64; 2]>], lasso: &[[f64; 2]]) -> f64 {
    if lasso.len() < 3 {
        return 0.0;
    }
    length_cover_edges(lines, &edges_of(&[lasso.to_vec()]))
}

fn length_cover_edges(lines: &[Vec<[f64; 2]>], le: &[Edge]) -> f64 {
    let Some(first) = lines.iter().find_map(|l| l.first().copied()) else {
        return 0.0;
    };
    let (mut inside, mut total) = (0.0, 0.0);
    for s in lines.iter().flat_map(|l| l.windows(2)) {
        let (a, b) = (s[0], s[1]);
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        if len <= 0.0 {
            continue;
        }
        let mut ts = vec![0.0, 1.0];
        let seg = (a, b);
        for e in le.iter().filter(|e| boxes_meet(&seg, e)) {
            if let Some(t) = cross_t(seg, *e) {
                ts.push(t);
            }
        }
        ts.sort_by(f64::total_cmp);
        for w in ts.windows(2) {
            let tm = (w[0] + w[1]) / 2.0;
            let m = [a[0] + tm * (b[0] - a[0]), a[1] + tm * (b[1] - a[1])];
            let piece = (w[1] - w[0]) * len;
            total += piece;
            if in_edges(le, m) {
                inside += piece;
            }
        }
    }
    if total <= 0.0 {
        f64::from(u8::from(in_edges(le, first)))
    } else {
        inside / total
    }
}

/// O parâmetro em `p` onde ele cruza `q`.
fn cross_t(p: Edge, q: Edge) -> Option<f64> {
    let (a, b, c, d) = (p.0, p.1, q.0, q.1);
    let r = [b[0] - a[0], b[1] - a[1]];
    let s = [d[0] - c[0], d[1] - c[1]];
    let den = r[0] * s[1] - r[1] * s[0];
    if den.abs() < 1e-300 {
        return None;
    }
    let ac = [c[0] - a[0], c[1] - a[1]];
    let t = (ac[0] * s[1] - ac[1] * s[0]) / den;
    let u = (ac[0] * r[1] - ac[1] * r[0]) / den;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then_some(t)
}

/// Os pontos de um caminho do vetor achatado (W23 B13: a forma do QuickShape na caneta do quadro).
#[must_use]
pub fn vec_polyline(path: &ph2d_vec_scene::VecPath, tolerance: f64) -> Vec<Vec<[f64; 2]>> {
    polyline(&ph2d_vec_render::build_bezpath(path), tolerance)
}

/// Os pontos de um caminho achatado como uma linha aberta (os sub-caminhos seguidos).
#[must_use]
pub fn polyline(path: &BezPath, tolerance: f64) -> Vec<Vec<[f64; 2]>> {
    let mut out: Vec<Vec<[f64; 2]>> = Vec::new();
    flatten(path.iter(), tolerance, |el| match el {
        PathEl::MoveTo(p) => out.push(vec![[p.x, p.y]]),
        PathEl::LineTo(Point { x, y }) => {
            if let Some(last) = out.last_mut() {
                last.push([x, y]);
            }
        }
        PathEl::ClosePath => {
            if let Some(last) = out.last_mut()
                && let Some(&first) = last.first()
            {
                last.push(first);
            }
        }
        _ => {}
    });
    out
}

/// Quantas faixas de x o índice do laço usa (cada aresta vive nas faixas que atravessa).
const BUCKETS: usize = 512;

/// ⭐ **O laço pronto a perguntar**: as arestas feitas UMA vez por movimento e indexadas por faixas
/// de x — cada elemento só vê as que passam por cima dele. Medido (M2, doc 17 §5): refazer as arestas
/// por elemento custava 42,6 ms por movimento com 2 500 pontos sobre 100 mil caixas.
#[derive(Clone, Debug)]
pub struct Lasso {
    edges: Vec<Edge>,
    buckets: Vec<Vec<u32>>,
    x0: f64,
    dx: f64,
    bbox: [f64; 4],
}

impl Lasso {
    /// O laço do rasto `trail` (fecha-se sozinho). `None` com menos de 3 pontos.
    #[must_use]
    pub fn new(trail: &[[f64; 2]]) -> Option<Self> {
        if trail.len() < 3 {
            return None;
        }
        let edges = edges_of(&[trail.to_vec()]);
        let bbox = trail.iter().fold(
            [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ],
            |b, p| {
                [
                    b[0].min(p[0]),
                    b[1].min(p[1]),
                    b[2].max(p[0]),
                    b[3].max(p[1]),
                ]
            },
        );
        let x0 = bbox[0];
        let dx = ((bbox[2] - bbox[0]) / BUCKETS as f64).max(1e-12);
        let mut buckets = vec![Vec::new(); BUCKETS];
        for (i, e) in edges.iter().enumerate() {
            let [a, b] = span_x(e);
            let (ia, ib) = (Self::slot(x0, dx, a), Self::slot(x0, dx, b));
            for bk in &mut buckets[ia..=ib] {
                bk.push(i as u32);
            }
        }
        Some(Self {
            edges,
            buckets,
            x0,
            dx,
            bbox,
        })
    }

    fn slot(x0: f64, dx: f64, x: f64) -> usize {
        (((x - x0) / dx).floor().max(0.0) as usize).min(BUCKETS - 1)
    }

    /// A caixa do laço `[x0, y0, x1, y1]`.
    #[must_use]
    pub fn bbox(&self) -> [f64; 4] {
        self.bbox
    }

    /// As arestas cuja faixa de x toca `[a, b]`.
    fn near(&self, a: f64, b: f64) -> Vec<Edge> {
        if b < self.bbox[0] || a > self.bbox[2] {
            return Vec::new();
        }
        let (ia, ib) = (
            Self::slot(self.x0, self.dx, a),
            Self::slot(self.x0, self.dx, b),
        );
        let mut ids: Vec<u32> = self.buckets[ia..=ib].iter().flatten().copied().collect();
        ids.sort_unstable();
        ids.dedup();
        ids.into_iter()
            .map(|i| self.edges[i as usize])
            .filter(|e| {
                let s = span_x(e);
                s[0] <= b && s[1] >= a
            })
            .collect()
    }

    /// `p` está dentro (non-zero)?
    #[must_use]
    pub fn contains(&self, p: [f64; 2]) -> bool {
        in_edges(&self.near(p[0], p[0]), p)
    }

    /// ⭐ **A porta do laço**: quanto de `el` (caixa `b = [x0, y0, x1, y1]`) o laço cobre — a área do
    /// contorno real (formas, notas, imagens, vídeos; frames pela caixa) ou o comprimento (a tinta;
    /// as setas pela rota `route`, que o editor calcula). Sem aresta do laço sobre a caixa, o elemento
    /// está todo de um lado: conta pelo centro (exacto — o número de voltas não muda na caixa).
    #[must_use]
    pub fn cover(&self, el: &Element, b: [f64; 4], route: Option<&ph2d_vec_scene::VecPath>) -> f64 {
        let le = self.near(b[0], b[2]);
        let crossing = le.iter().any(|e| {
            let ys = [e.0[1].min(e.1[1]), e.0[1].max(e.1[1])];
            ys[0] <= b[3] && ys[1] >= b[1]
        });
        if !crossing {
            let c = [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0];
            return f64::from(u8::from(in_edges(&le, c)));
        }
        if let Some(route) = route {
            let path = ph2d_vec_render::build_bezpath(route);
            let bb = path.bounding_box();
            let tol = FLAT * bb.width().hypot(bb.height()).max(1e-9);
            return length_cover_edges(&polyline(&path, tol), &le);
        }
        let tol = FLAT * el.w.hypot(el.h).max(1e-9);
        if let Some(ink) = el.ink() {
            let pts: Vec<[f64; 2]> = ink
                .placed([el.x, el.y, el.w, el.h])
                .into_iter()
                .map(|[x, y, _]| el.rotate([x, y]))
                .collect();
            return length_cover_edges(&[pts], &le);
        }
        let region = match crate::world_outline(el) {
            Some(o) => polygons(&o.fill, tol),
            None => {
                let (x, y, w, h) = (el.x, el.y, el.w, el.h);
                vec![
                    [[x, y], [x + w, y], [x + w, y + h], [x, y + h]]
                        .map(|p| el.rotate(p))
                        .to_vec(),
                ]
            }
        };
        area_cover_edges(&region, &le)
    }
}

#[cfg(test)]
#[path = "cover_tests.rs"]
mod tests;
