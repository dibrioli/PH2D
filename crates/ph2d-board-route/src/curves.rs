//! As CURVAS das rotas (cortadas do `lib.rs` no fecho da W9, tecto de LOC): a do Miro ([`curve`]), a
//! do Excalidraw ([`spline`]), o recuo do cotovelo que se olha ([`facing_jetty`]) e as medidas do
//! caminho (a corda das pontas, a caixa, o meio).

use ph2d_vec_connect::Dir;
use ph2d_vec_scene::{VecPath, VecVertex};

use crate::{END_ARM, JETTY, POINT_ARM, dist};

/// A corda do 1.º e do último trecho de `pts`.
pub(crate) fn chords(pts: &[[f64; 2]]) -> [f64; 2] {
    let n = pts.len();
    if n < 2 {
        return [0.0; 2];
    }
    [dist(pts[0], pts[1]), dist(pts[n - 2], pts[n - 1])]
}

/// ⭐ **A curva do Excalidraw pelas estações** (W9, XA1, `F_fuste` nas duas versões): Catmull-Rom
/// UNIFORME com pegas de 1/6 — a pega de saída de Pᵢ é `(Pᵢ₊₁ − Pᵢ₋₁)/6`, nas pontas `(P₁ − P₀)/6`.
/// Com duas estações sai uma recta (as pegas na própria corda). ⛔ Não é a [`curve`] do Miro (essa sai
/// perpendicular às formas): é a de quem desenhou no Excalidraw.
pub(crate) fn spline(st: &[[f64; 2]]) -> Vec<VecVertex> {
    let n = st.len();
    let at = |i: isize| st[i.clamp(0, n as isize - 1) as usize];
    (0..n)
        .map(|i| {
            let k = i as isize;
            let (prev, next) = (at(k - 1), at(k + 1));
            let t = [(next[0] - prev[0]) / 6.0, (next[1] - prev[1]) / 6.0];
            let mut v = VecVertex::corner(st[i]);
            if i + 1 < n {
                v.out_handle = [st[i][0] + t[0], st[i][1] + t[1]];
            }
            if i > 0 {
                v.in_handle = [st[i][0] - t[0], st[i][1] - t[1]];
            }
            v
        })
        .collect()
}

/// ⭐ **O recuo de um trecho cujas duas saídas se OLHAM** (opostas, com a chegada à frente da partida):
/// metade do vão, no máximo [`JETTY`] — a dobra fica a meio dele. ⛔ Com o recuo fixo de 40 num vão
/// menor que 80 os dois recuos cruzavam-se e a rota dava voltas que mudavam de forma a cada passo
/// do arrasto (6.º smoke do dono, 06/10: «as setas rectangulares tremem»). Fora disso, o medido.
pub(crate) fn facing_jetty(a: [f64; 2], da: Dir, b: [f64; 2], db: Dir) -> f64 {
    if db != da.opposite() {
        return JETTY;
    }
    let v = da.vec();
    let gap = (b[0] - a[0]) * v[0] + (b[1] - a[1]) * v[1];
    if gap > 0.0 {
        (gap / 2.0).min(JETTY)
    } else {
        JETTY
    }
}

/// ⭐ **A curva do Miro** pelas estações: em cada PONTA a tangente é a da saída (perpendicular à
/// face, `d[k]` aponta para FORA da forma) e o braço [`END_ARM`] da distância ao vizinho; num ponto
/// de ajuste a tangente é a do Catmull-Rom (do vizinho de trás ao da frente) e o braço
/// [`POINT_ARM`] do trecho.
pub(crate) fn curve(st: &[[f64; 2]], d: [[f64; 2]; 2]) -> Vec<VecVertex> {
    let n = st.len();
    let sub = |a: [f64; 2], b: [f64; 2]| [a[0] - b[0], a[1] - b[1]];
    let unit = |v: [f64; 2]| {
        let l = v[0].hypot(v[1]);
        if l < 1e-12 {
            [0.0, 0.0]
        } else {
            [v[0] / l, v[1] / l]
        }
    };
    // A tangente de PERCURSO em cada estação (no fim, entra contra a saída da forma).
    let tangent = |i: usize| -> [f64; 2] {
        if i == 0 {
            d[0]
        } else if i == n - 1 {
            [-d[1][0], -d[1][1]]
        } else {
            unit(sub(st[i + 1], st[i - 1]))
        }
    };
    let arm = |i: usize, toward: usize| -> f64 {
        let chord = dist(st[i], st[toward]);
        if i == 0 || i == n - 1 {
            END_ARM * chord
        } else {
            POINT_ARM * chord
        }
    };
    (0..n)
        .map(|i| {
            let t = tangent(i);
            let mut v = VecVertex::corner(st[i]);
            if i + 1 < n {
                let a = arm(i, i + 1);
                v.out_handle = [st[i][0] + t[0] * a, st[i][1] + t[1] * a];
            }
            if i > 0 {
                let a = arm(i, i - 1);
                v.in_handle = [st[i][0] - t[0] * a, st[i][1] - t[1] * a];
            }
            v
        })
        .collect()
}

pub(crate) fn bbox_of(path: &VecPath) -> [f64; 4] {
    use ph2d_vector::Shape as _;
    let r = ph2d_vec_render::build_bezpath(path).bounding_box();
    [r.x0, r.y0, r.x1, r.y1]
}

/// O ponto a meio comprimento do caminho desenhado.
pub(crate) fn midpoint(path: &VecPath) -> [f64; 2] {
    use ph2d_vector::{ParamCurve, ParamCurveArclen};
    let bp = ph2d_vec_render::build_bezpath(path);
    let segs: Vec<_> = bp.segments().collect();
    let lens: Vec<f64> = segs.iter().map(|s| s.arclen(ARCLEN_ACCURACY)).collect();
    let mut left = lens.iter().sum::<f64>() / 2.0;
    for (s, l) in segs.iter().zip(&lens) {
        if left <= *l {
            let p = s.eval(s.inv_arclen(left, ARCLEN_ACCURACY));
            return [p.x, p.y];
        }
        left -= l;
    }
    path.verts.first().map_or([0.0, 0.0], |v| v.anchor)
}

/// A precisão do comprimento de arco (mundo): um centésimo de unidade, abaixo de um px a qualquer
/// zoom de trabalho.
const ARCLEN_ACCURACY: f64 = 1e-2;
