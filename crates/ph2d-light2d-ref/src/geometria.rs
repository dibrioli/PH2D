//! Raio × fronteira e fronteira × fronteira, em `f64`.

use std::f64::consts::TAU;

/// A fronteira de um objeto: um disco ou um polígono simples (qualquer orientação).
pub(crate) enum Fronteira {
    Disco { c: [f64; 2], r: f64 },
    Poligono(Vec<[f64; 2]>),
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn cruz(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// O ângulo de `q` visto de `p`, em `[0, 2π)`.
pub(crate) fn angulo(p: [f64; 2], q: [f64; 2]) -> f64 {
    (q[1] - p[1]).atan2(q[0] - p[0]).rem_euclid(TAU)
}

fn arestas(v: &[[f64; 2]]) -> impl Iterator<Item = ([f64; 2], [f64; 2])> + '_ {
    v.iter()
        .zip(v.iter().cycle().skip(1))
        .map(|(&a, &b)| (a, b))
}

/// As distâncias `t > 0` em que o raio `o + t·d` (`|d| = 1`) cruza a fronteira, ordenadas. Pela
/// regra de Jordan, a origem está DENTRO sse a contagem é ímpar — a mesma lei para disco e
/// polígono, decidida pelo próprio raio que se lança.
pub(crate) fn cruzamentos(f: &Fronteira, o: [f64; 2], d: [f64; 2], out: &mut Vec<f64>) {
    out.clear();
    match f {
        Fronteira::Disco { c, r } => {
            let oc = sub(o, *c);
            let b = dot(d, oc);
            let cc = dot(oc, oc) - r * r;
            let disc = b * b - cc;
            if disc > 0.0 {
                let s = disc.sqrt();
                out.extend([-b - s, -b + s].into_iter().filter(|&t| t > 0.0));
            }
        }
        Fronteira::Poligono(v) => {
            if v.len() < 3 {
                return;
            }
            for (a, b) in arestas(v) {
                let e = sub(b, a);
                let den = cruz(d, e);
                if den == 0.0 {
                    continue;
                }
                let w = sub(a, o);
                let t = cruz(w, e) / den;
                let u = cruz(w, d) / den;
                if t > 0.0 && (0.0..1.0).contains(&u) {
                    out.push(t);
                }
            }
            out.sort_by(f64::total_cmp);
        }
    }
}

fn seg_seg(a: [f64; 2], b: [f64; 2], p: [f64; 2], q: [f64; 2], out: &mut Vec<[f64; 2]>) {
    let r = sub(b, a);
    let s = sub(q, p);
    let den = cruz(r, s);
    if den == 0.0 {
        return;
    }
    let w = sub(p, a);
    let t = cruz(w, s) / den;
    let u = cruz(w, r) / den;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        out.push([a[0] + t * r[0], a[1] + t * r[1]]);
    }
}

fn seg_circ(a: [f64; 2], b: [f64; 2], c: [f64; 2], raio: f64, out: &mut Vec<[f64; 2]>) {
    let e = sub(b, a);
    let w = sub(a, c);
    let qa = dot(e, e);
    if qa == 0.0 {
        return;
    }
    let qb = dot(w, e);
    let qc = dot(w, w) - raio * raio;
    let disc = qb * qb - qa * qc;
    if disc < 0.0 {
        return;
    }
    let s = disc.sqrt();
    for u in [(-qb - s) / qa, (-qb + s) / qa] {
        if (0.0..=1.0).contains(&u) {
            out.push([a[0] + u * e[0], a[1] + u * e[1]]);
        }
    }
}

fn circ_circ(c1: [f64; 2], r1: f64, c2: [f64; 2], r2: f64, out: &mut Vec<[f64; 2]>) {
    let v = sub(c2, c1);
    let d = v[0].hypot(v[1]);
    if d == 0.0 || d > r1 + r2 || d < (r1 - r2).abs() {
        return;
    }
    let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
    let h = (r1 * r1 - a * a).max(0.0).sqrt();
    let m = [c1[0] + a * v[0] / d, c1[1] + a * v[1] / d];
    out.push([m[0] - h * v[1] / d, m[1] + h * v[0] / d]);
    out.push([m[0] + h * v[1] / d, m[1] - h * v[0] / d]);
}

/// Acrescenta a `out` os pontos onde as duas fronteiras se cruzam.
pub(crate) fn intersecoes(f: &Fronteira, g: &Fronteira, out: &mut Vec<[f64; 2]>) {
    match (f, g) {
        (Fronteira::Disco { c: c1, r: r1 }, Fronteira::Disco { c: c2, r: r2 }) => {
            circ_circ(*c1, *r1, *c2, *r2, out);
        }
        (Fronteira::Disco { c, r }, Fronteira::Poligono(v))
        | (Fronteira::Poligono(v), Fronteira::Disco { c, r }) => {
            for (a, b) in arestas(v) {
                seg_circ(a, b, *c, *r, out);
            }
        }
        (Fronteira::Poligono(v), Fronteira::Poligono(w)) => {
            for (a, b) in arestas(v) {
                for (p, q) in arestas(w) {
                    seg_seg(a, b, p, q, out);
                }
            }
        }
    }
}
