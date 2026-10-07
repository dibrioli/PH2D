//! Curvas em pontos: o `normalize` do path-data-parser 0.1.0 (para os segmentos que o quadro
//! produz), o `pointsOnPath` do points-on-path 0.2.1 e o points-on-curve 0.2.0 (achatar, RDP e a
//! Catmull-Rom em Bézier).

use crate::P;
use crate::rough::Seg;

/// Só `M`, `L`, `C`, `Z`: a quadrática vira a cúbica exacta.
pub(crate) fn normalize(segs: &[Seg]) -> Vec<Seg> {
    let mut out = Vec::with_capacity(segs.len());
    let (mut cx, mut cy) = (0.0, 0.0);
    let (mut subx, mut suby) = (0.0, 0.0);
    for seg in segs {
        match *seg {
            Seg::M(p) => {
                out.push(*seg);
                [cx, cy] = p;
                [subx, suby] = p;
            }
            Seg::L(p) => {
                out.push(*seg);
                [cx, cy] = p;
            }
            Seg::C(_, _, p) => {
                out.push(*seg);
                [cx, cy] = p;
            }
            Seg::Q([x1, y1], [x, y]) => {
                let c1 = [cx + 2.0 * (x1 - cx) / 3.0, cy + 2.0 * (y1 - cy) / 3.0];
                let c2 = [x + 2.0 * (x1 - x) / 3.0, y + 2.0 * (y1 - y) / 3.0];
                out.push(Seg::C(c1, c2, [x, y]));
                (cx, cy) = (x, y);
            }
            Seg::Z => {
                out.push(Seg::Z);
                (cx, cy) = (subx, suby);
            }
        }
    }
    out
}

/// Os polígonos de um caminho normalizado (um por subcaminho), simplificados a `distance`.
pub(crate) fn points_on_path(segs: &[Seg], tolerance: f64, distance: f64) -> Vec<Vec<P>> {
    let mut sets: Vec<Vec<P>> = Vec::new();
    let mut current: Vec<P> = Vec::new();
    let mut start = [0.0, 0.0];
    let mut pending: Vec<P> = Vec::new();
    fn flush_curve(pending: &mut Vec<P>, current: &mut Vec<P>, tolerance: f64) {
        if pending.len() >= 4 {
            current.extend(points_on_bezier_curves(pending, tolerance, 0.0));
        }
        pending.clear();
    }
    for seg in segs {
        match *seg {
            Seg::M(p) => {
                flush_curve(&mut pending, &mut current, tolerance);
                if !current.is_empty() {
                    sets.push(std::mem::take(&mut current));
                }
                start = p;
                current.push(start);
            }
            Seg::L(p) => {
                flush_curve(&mut pending, &mut current, tolerance);
                current.push(p);
            }
            Seg::C(c1, c2, p) => {
                if pending.is_empty() {
                    pending.push(current.last().copied().unwrap_or(start));
                }
                pending.extend([c1, c2, p]);
            }
            Seg::Z => {
                flush_curve(&mut pending, &mut current, tolerance);
                current.push(start);
            }
            Seg::Q(..) => unreachable!("normalize converte Q em C"),
        }
    }
    flush_curve(&mut pending, &mut current, tolerance);
    if !current.is_empty() {
        sets.push(current);
    }
    if distance == 0.0 {
        return sets;
    }
    sets.into_iter()
        .map(|set| simplify(&set, distance))
        .filter(|s| !s.is_empty())
        .collect()
}

fn distance_sq(a: P, b: P) -> f64 {
    (a[0] - b[0]) * (a[0] - b[0]) + (a[1] - b[1]) * (a[1] - b[1])
}

fn lerp(a: P, b: P, t: f64) -> P {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

fn distance_to_segment_sq(p: P, v: P, w: P) -> f64 {
    let l2 = distance_sq(v, w);
    if l2 == 0.0 {
        return distance_sq(p, v);
    }
    let t = ((p[0] - v[0]) * (w[0] - v[0]) + (p[1] - v[1]) * (w[1] - v[1])) / l2;
    distance_sq(p, lerp(v, w, t.clamp(0.0, 1.0)))
}

fn flatness(p: &[P]) -> f64 {
    let (p1, p2, p3, p4) = (p[0], p[1], p[2], p[3]);
    let mut ux = 3.0 * p2[0] - 2.0 * p1[0] - p4[0];
    ux *= ux;
    let mut uy = 3.0 * p2[1] - 2.0 * p1[1] - p4[1];
    uy *= uy;
    let mut vx = 3.0 * p3[0] - 2.0 * p4[0] - p1[0];
    vx *= vx;
    let mut vy = 3.0 * p3[1] - 2.0 * p4[1] - p1[1];
    vy *= vy;
    if ux < vx {
        ux = vx;
    }
    if uy < vy {
        uy = vy;
    }
    ux + uy
}

fn split_until_flat(p: &[P], tolerance: f64, out: &mut Vec<P>) {
    if flatness(p) < tolerance {
        let p0 = p[0];
        match out.last() {
            Some(&last) if distance_sq(last, p0).sqrt() > 1.0 => out.push(p0),
            Some(_) => {}
            None => out.push(p0),
        }
        out.push(p[3]);
    } else {
        let (p1, p2, p3, p4) = (p[0], p[1], p[2], p[3]);
        let q1 = lerp(p1, p2, 0.5);
        let q2 = lerp(p2, p3, 0.5);
        let q3 = lerp(p3, p4, 0.5);
        let r1 = lerp(q1, q2, 0.5);
        let r2 = lerp(q2, q3, 0.5);
        let red = lerp(r1, r2, 0.5);
        split_until_flat(&[p1, q1, r1, red], tolerance, out);
        split_until_flat(&[red, r2, q3, p4], tolerance, out);
    }
}

/// Ramer–Douglas–Peucker.
pub(crate) fn simplify(points: &[P], epsilon: f64) -> Vec<P> {
    let mut out = Vec::new();
    simplify_points(points, 0, points.len(), epsilon, &mut out);
    out
}

fn simplify_points(points: &[P], start: usize, end: usize, epsilon: f64, out: &mut Vec<P>) {
    let s = points[start];
    let e = points[end - 1];
    let mut max_sq = 0.0;
    let mut max_i = 1;
    for (i, &p) in points.iter().enumerate().take(end - 1).skip(start + 1) {
        let d = distance_to_segment_sq(p, s, e);
        if d > max_sq {
            max_sq = d;
            max_i = i;
        }
    }
    if max_sq.sqrt() > epsilon {
        simplify_points(points, start, max_i + 1, epsilon, out);
        simplify_points(points, max_i, end, epsilon, out);
    } else {
        if out.is_empty() {
            out.push(s);
        }
        out.push(e);
    }
}

/// `pointsOnBezierCurves`: `points` = `p0 c1 c2 p1 c1 c2 p2 …`; `distance > 0` simplifica no fim.
pub(crate) fn points_on_bezier_curves(points: &[P], tolerance: f64, distance: f64) -> Vec<P> {
    let mut out = Vec::new();
    let segments = (points.len() - 1) / 3;
    for i in 0..segments {
        split_until_flat(&points[i * 3..i * 3 + 4], tolerance, &mut out);
    }
    if distance > 0.0 {
        return simplify(&out, distance);
    }
    out
}

/// `curveToBezier` (Catmull-Rom → Bézier), para o preenchimento de uma curva.
pub(crate) fn curve_to_bezier(input: &[P], tightness: f64) -> Vec<P> {
    let len = input.len();
    assert!(len >= 3, "uma curva tem pelo menos três pontos");
    if len == 3 {
        return vec![input[0], input[1], input[2], input[2]];
    }
    let mut points = vec![input[0], input[0]];
    for (i, &p) in input.iter().enumerate().skip(1) {
        points.push(p);
        if i == len - 1 {
            points.push(p);
        }
    }
    let s = 1.0 - tightness;
    let mut out = vec![points[0]];
    let mut i = 1;
    while i + 2 < points.len() {
        let v = points[i];
        out.push([
            v[0] + (s * points[i + 1][0] - s * points[i - 1][0]) / 6.0,
            v[1] + (s * points[i + 1][1] - s * points[i - 1][1]) / 6.0,
        ]);
        out.push([
            points[i + 1][0] + (s * points[i][0] - s * points[i + 2][0]) / 6.0,
            points[i + 1][1] + (s * points[i][1] - s * points[i + 2][1]) / 6.0,
        ]);
        out.push(points[i + 1]);
        i += 1;
    }
    out
}
