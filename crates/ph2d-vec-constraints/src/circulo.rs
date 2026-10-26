//! O círculo é GEOMETRIA (§3.2): fechado, sem contornos extra, ≥ 3 âncoras no mesmo raio e cada alça tangente.

use ph2d_vec_scene::{VecPath, Xform};

/// Folga relativa ao raio: o cozimento usa trigonometria, e a escrita de volta é `c + k·(p − c)`.
const TOL_REL: f64 = 1e-7;

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn circuncentro(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Option<[f64; 2]> {
    let (u, v) = (sub(b, a), sub(c, a));
    let d = 2.0 * (u[0] * v[1] - u[1] * v[0]);
    if d.abs() < f64::EPSILON {
        return None;
    }
    let (uu, vv) = (u[0] * u[0] + u[1] * u[1], v[0] * v[0] + v[1] * v[1]);
    Some([a[0] + (v[1] * uu - u[1] * vv) / d, a[1] + (u[0] * vv - v[0] * uu) / d])
}

/// Centro e raio na FOLHA, se o path é um círculo depois do afim (uma escala não-uniforme dá elipse ⇒ `None`).
#[must_use]
pub fn circulo_de(path: &VecPath, xf: Xform) -> Option<([f64; 2], f64)> {
    if !path.closed || !path.subpaths.is_empty() || path.verts.len() < 3 {
        return None;
    }
    let a: Vec<[f64; 2]> = path.verts.iter().map(|v| xf.apply(v.anchor)).collect();
    let c = circuncentro(a[0], a[1], a[2])?;
    let r = sub(a[0], c)[0].hypot(sub(a[0], c)[1]);
    let tol = TOL_REL * r.max(1.0);
    for (v, p) in path.verts.iter().zip(&a) {
        let raio = sub(*p, c);
        if (raio[0].hypot(raio[1]) - r).abs() > tol {
            return None;
        }
        for h in [v.in_handle, v.out_handle] {
            let t = sub(xf.apply(h), *p);
            if (t[0] * raio[0] + t[1] * raio[1]).abs() > tol * t[0].hypot(t[1]).max(1.0) {
                return None;
            }
        }
    }
    Some((c, r))
}
