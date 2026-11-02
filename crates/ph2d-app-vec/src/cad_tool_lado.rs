//! **O lado de uma forma no ecrã, como ela o DESENHA** — a cúbica do segmento (âncora, alça de saída, alça de
//! entrada, âncora), nunca a corda entre as duas âncoras.
//!
//! Report do dono (01–02/11, `img_F45/dono_linha_residual_1.png`): depois do Smooth, a cor CAD dos lados ficava em
//! rectas sobre a arte curva — «a linha do caminho antigo». A regra continua a falar da RECTA entre os dois pontos (é
//! o que o solver prende); o que se pinta, se acerta e se acende é o lado da forma.

use ph2d_vec_scene::cad::{PontoCad, SegCad};
use ph2d_vec_scene::{VecScene, VecVertex, VecViewState, VecXforms};
use ph2d_vector::{Affine, BezPath, ParamCurve, ParamCurveNearest, PathSeg, Point, Shape};

/// Um lado no ecrã: os quatro pontos de controlo da cúbica, em px.
pub(crate) type Curva = [[f64; 2]; 4];

/// O que se acende no ecrã: um ponto (disco) ou um lado (a curva dele).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Marca {
    Ponto([f64; 2]),
    Lado(Curva),
}

/// A recta de `a` a `b` como cúbica — o que liga dois pontos que não são um lado da forma.
pub(crate) fn reta(a: [f64; 2], b: [f64; 2]) -> Curva {
    [a, a, b, b]
}

/// **O lado `s` no ecrã**: a cúbica do segmento se os dois pontos são vizinhos num contorno (em qualquer sentido); senão
/// a recta entre eles. `None` se um ponto não está no ecrã (forma escondida, vértice apagado).
pub(crate) fn no_ecra(
    scene: &VecScene,
    view: &VecViewState,
    xf: &VecXforms,
    cam: Affine,
    s: SegCad,
    ponto: impl Fn(PontoCad) -> Option<[f64; 2]>,
) -> Option<Curva> {
    let (qa, qb) = (ponto(s.0)?, ponto(s.1)?);
    let (PontoCad::Vert(pa, va), PontoCad::Vert(pb, vb)) = (s.0, s.1) else {
        return Some(reta(qa, qb));
    };
    let path = scene.path(pa).filter(|_| pa == pb);
    let t = ph2d_vec_render::overlay_transform(view, xf, pa, cam);
    let contornos = path.into_iter().flat_map(|p| {
        std::iter::once((&p.verts, p.closed)).chain(p.subpaths.iter().map(|c| (&c.verts, c.closed)))
    });
    for (verts, fechado) in contornos {
        let n = verts.len();
        let pares = if fechado { n } else { n.saturating_sub(1) };
        for i in 0..pares {
            let (a, b) = (&verts[i], &verts[(i + 1) % n]);
            if (a.id, b.id) == (va, vb) {
                return Some(cubica(t, a, b));
            }
            if (a.id, b.id) == (vb, va) {
                let [p0, p1, p2, p3] = cubica(t, a, b);
                return Some([p3, p2, p1, p0]);
            }
        }
    }
    Some(reta(qa, qb))
}

/// O segmento de `a` ao vértice seguinte `b`, levado ao ecrã por `t` (o afim dos overlays da forma).
pub(crate) fn cubica(t: Affine, a: &VecVertex, b: &VecVertex) -> Curva {
    let tela = |p: [f64; 2]| {
        let q = t * Point::new(p[0], p[1]);
        [q.x, q.y]
    };
    [
        tela(a.anchor),
        tela(a.out_handle),
        tela(b.in_handle),
        tela(b.anchor),
    ]
}

fn seg(c: &Curva) -> PathSeg {
    caminho(c)
        .segments()
        .next()
        .expect("a cúbica tem um segmento")
}

/// A curva como `BezPath` (para pintar).
pub(crate) fn caminho(c: &Curva) -> BezPath {
    let p = |q: [f64; 2]| Point::new(q[0], q[1]);
    let mut b = BezPath::new();
    b.move_to(p(c[0]));
    b.curve_to(p(c[1]), p(c[2]), p(c[3]));
    b
}

/// A distância (px) de `p` à curva.
pub(crate) fn distancia(c: &Curva, p: [f64; 2]) -> f64 {
    seg(c)
        .nearest(Point::new(p[0], p[1]), 1e-3)
        .distance_sq
        .sqrt()
}

/// O meio da curva (`t = 0,5`) e a normal unitária ali — à direita de quem anda de `c[0]` para `c[3]`, a mesma da
/// recta `(dy, −dx)`.
pub(crate) fn meio_e_normal(c: &Curva) -> ([f64; 2], [f64; 2]) {
    let m = seg(c).eval(0.5);
    let d = |i: usize, k: f64| [(c[i + 1][0] - c[i][0]) * k, (c[i + 1][1] - c[i][1]) * k];
    let (d0, d1, d2) = (d(0, 0.75), d(1, 1.5), d(2, 0.75));
    let (dx, dy) = (d0[0] + d1[0] + d2[0], d0[1] + d1[1] + d2[1]);
    let n = dx.hypot(dy);
    let normal = if n > 0.0 {
        [dy / n, -dx / n]
    } else {
        [1.0, 0.0]
    };
    ([m.x, m.y], normal)
}

/// Os cantos da caixa da curva (a âncora da ④ cobre o lado inteiro).
pub(crate) fn caixa(c: &Curva) -> [[f64; 2]; 2] {
    let r = caminho(c).bounding_box();
    [[r.x0, r.y0], [r.x1, r.y1]]
}

#[cfg(test)]
#[path = "cad_tool_lado_tests.rs"]
mod tests;
