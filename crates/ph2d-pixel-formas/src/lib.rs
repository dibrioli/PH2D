//! **As formas do Vector em células** (P6b, ADR-0196 §12): a geometria é a do Vector
//! ([`ph2d_vec_scene::cook`], a mesma porta que o editor vetorial usa) sobre a caixa do gesto em
//! células, achatada em vértices de célula inteira; o contorno e o interior são a lei do polígono do
//! núcleo ([`polygon_cells`]: Bresenham entre vértices, par-ímpar nos centros), com os furos em
//! par-ímpar entre contornos. O contorno ABERTO (a linha, o arco, a espiral, as arestas de dentro
//! do cubo) é só o traço ([`line_path`]); a forma que o Vector não enche
//! ([`ShapeKind::is_closed`](ph2d_vec_scene::ShapeKind::is_closed)) não tem interior.
//!
//! ⚠️ Com os campos NEUTROS o retângulo e a elipse continuam as leis medidas do Pixel
//! (`rounded_rect_cells` — o canto do Aseprite — e a elipse de Zingl): o desenho aprovado não muda.

use std::collections::BTreeSet;

use ph2d_editor_core::pixel_panel::{PixelShape, PixelShapeValues};
use ph2d_pixel_core::raster::{
    Cell, ellipse_cells, ellipse_filled_cells, line_path, polygon_cells, rounded_rect_cells,
};
use ph2d_vec_scene::{VecPath, VecVertex, cook, trim_tool::contours_of};

/// O passo do achatamento das curvas, em células de corda.
const PASSO: f64 = 0.5;

/// A forma `s` na caixa `a..=b` (cantos opostos, em células): `(contorno, interior)`; o interior só
/// com `cheia`.
#[must_use]
pub fn cells(
    s: PixelShape,
    v: &PixelShapeValues,
    a: Cell,
    b: Cell,
    cheia: bool,
) -> (Vec<Cell>, Vec<Cell>) {
    match neutra(s, v) {
        Some(Neutra::Retangulo(r)) => return retangulo(a, b, r, cheia),
        Some(Neutra::Elipse) => return elipse(a, b, cheia),
        None => {}
    }
    // O Vector cozinha com Y para cima: a caixa vai espelhada e os vértices voltam.
    let (pa, pb) = (
        [f64::from(a.0), -f64::from(a.1)],
        [f64::from(b.0), -f64::from(b.1)],
    );
    let path = cook(s.kind(), pa, pb, &v.values());
    celulas(&path, &|p| [p[0], -p[1]], cheia && s.kind().is_closed())
}

/// A forma `s` cozida na caixa CENTRADA de meia-extensão `half` (o espaço local do Vector: Y para
/// cima, em células de centro a centro) — a mesma porta que as alças da forma viva lêem.
#[must_use]
pub fn cook_local(s: PixelShape, half: [f64; 2], v: &ph2d_vec_scene::ShapeValues) -> VecPath {
    cook(s.kind(), [-half[0], -half[1]], half, v)
}

/// A forma `s` no espaço local ([`cook_local`]) levada por `map` aos CENTROS das células (a forma
/// viva girada): `(contorno, interior)`.
#[must_use]
pub fn cells_mapped(
    s: PixelShape,
    v: &PixelShapeValues,
    half: [f64; 2],
    map: &dyn Fn([f64; 2]) -> [f64; 2],
    cheia: bool,
) -> (Vec<Cell>, Vec<Cell>) {
    let cheia = cheia && s.kind().is_closed();
    celulas(&cook_local(s, half, &v.values()), map, cheia)
}

/// Os contornos de `path`, levados por `map` às células: contorno e interior par-ímpar.
fn celulas(
    path: &VecPath,
    map: &dyn Fn([f64; 2]) -> [f64; 2],
    cheia: bool,
) -> (Vec<Cell>, Vec<Cell>) {
    let mut contorno = BTreeSet::new();
    let mut dentro = BTreeSet::new();
    for c in contours_of(path) {
        let pts = achata(&c.verts, c.closed, map);
        if !c.closed {
            contorno.extend(pts.windows(2).flat_map(|w| line_path(w[0], w[1])));
            contorno.extend(pts.first());
            continue;
        }
        let (o, i) = polygon_cells(&pts);
        contorno.extend(o);
        // Par-ímpar entre contornos: o furo da rosquinha tira o miolo.
        for p in i {
            if !dentro.remove(&p) {
                dentro.insert(p);
            }
        }
    }
    let interior = if cheia {
        dentro.difference(&contorno).copied().collect()
    } else {
        Vec::new()
    };
    (contorno.into_iter().collect(), interior)
}

/// A lei medida do Pixel que cobre a forma.
enum Neutra {
    /// O retângulo de canto redondo do Aseprite, de raio inteiro.
    Retangulo(u32),
    /// A elipse de Zingl.
    Elipse,
}

/// A lei medida do Pixel quando os campos não pedem nada que ela não tenha.
fn neutra(s: PixelShape, v: &PixelShapeValues) -> Option<Neutra> {
    match s.kind() {
        // O retângulo do Vector sem canto: o do Aseprite.
        ph2d_vec_scene::ShapeKind::Rectangle if v.get(0) == 0.0 => Some(Neutra::Retangulo(0)),
        // O raio base é o canto do Aseprite; os desvios, a suavização e o tipo do canto a zero.
        ph2d_vec_scene::ShapeKind::RoundRect
            if (1..6).all(|i| v.get(i) == 0.0) && v.get(0) >= 0.0 && v.get(0).fract() == 0.0 =>
        {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "raio inteiro e não negativo, preso pelo painel"
            )]
            let r = v.get(0) as u32;
            Some(Neutra::Retangulo(r))
        }
        // Inteira, sem furo nem cantos.
        ph2d_vec_scene::ShapeKind::Ellipse
            if v.get(0) >= ph2d_vec_scene::FULL_TURN && v.get(2) == 0.0 && v.get(3) == 0.0 =>
        {
            Some(Neutra::Elipse)
        }
        _ => None,
    }
}

fn retangulo(a: Cell, b: Cell, r: u32, cheia: bool) -> (Vec<Cell>, Vec<Cell>) {
    let out = collect(|cb| rounded_rect_cells(a.0, a.1, b.0, b.1, r, false, cb));
    let inner = if cheia {
        collect(|cb| rounded_rect_cells(a.0, a.1, b.0, b.1, r, true, cb))
    } else {
        Vec::new()
    };
    (out, inner)
}

fn elipse(a: Cell, b: Cell, cheia: bool) -> (Vec<Cell>, Vec<Cell>) {
    let out = collect(|cb| ellipse_cells(a.0, a.1, b.0, b.1, cb));
    let inner = if cheia {
        collect(|cb| ellipse_filled_cells(a.0, a.1, b.0, b.1, cb))
    } else {
        Vec::new()
    };
    (out, inner)
}

fn collect(f: impl FnOnce(&mut dyn FnMut(i32, i32))) -> Vec<Cell> {
    let mut v = Vec::new();
    f(&mut |x, y| v.push((x, y)));
    v
}

/// Os vértices de célula de um contorno do Vector: cada cúbica amostrada a [`PASSO`] células de
/// corda, levada por `map` às células e arredondada ao centro, sem repetidos seguidos.
fn achata(verts: &[VecVertex], fechado: bool, map: &dyn Fn([f64; 2]) -> [f64; 2]) -> Vec<Cell> {
    let n = verts.len();
    let mut out: Vec<Cell> = Vec::new();
    let mut poe = |p: [f64; 2]| {
        let q = map(p);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "célula de um sprite (≤ 65535)"
        )]
        let c = (q[0].round() as i32, q[1].round() as i32);
        if out.last() != Some(&c) {
            out.push(c);
        }
    };
    let arestas = if fechado { n } else { n.saturating_sub(1) };
    for i in 0..arestas {
        let (v0, v1) = (&verts[i], &verts[(i + 1) % n]);
        let (p0, c1, c2, p3) = (v0.anchor, v0.out_handle, v1.in_handle, v1.anchor);
        let corda = dist(p0, c1) + dist(c1, c2) + dist(c2, p3);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "passos de uma curva de um sprite"
        )]
        let k = ((corda / PASSO).ceil() as usize).max(1);
        for j in 0..k {
            #[expect(clippy::cast_precision_loss, reason = "índice de passo")]
            let t = j as f64 / k as f64;
            poe(cubica(p0, c1, c2, p3, t));
        }
    }
    if !fechado && let Some(v) = verts.last() {
        poe(v.anchor);
    }
    if fechado && out.len() > 1 && out.first() == out.last() {
        out.pop();
    }
    out
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn cubica(p0: [f64; 2], c1: [f64; 2], c2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        a * p0[0] + b * c1[0] + c * c2[0] + d * p3[0],
        a * p0[1] + b * c1[1] + c * c2[1] + d * p3[1],
    ]
}
