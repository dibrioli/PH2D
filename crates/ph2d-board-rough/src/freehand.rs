//! O **perfect-freehand 1.2.0** portado: dos pontos da caneta (com pressão) ao CONTORNO que se
//! preenche. Dois passos, como na biblioteca, e os dois conferidos contra o oráculo:
//! [`stroke_points`] (`getStrokePoints`: alisar e medir) e [`outline`] (`getStrokeOutlinePoints`).
//!
//! As facilidades (`easing`) são as de fábrica da biblioteca — linear na pressão, `t(2−t)` no
//! afilar do início, `(t−1)³+1` no do fim; nenhum chamador daqui as troca.

use std::f64::consts::PI;

use crate::P;

/// Como afila uma ponta: `0` = não afila; [`Taper::Full`] = o traço inteiro (`taper: true`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Taper {
    Length(f64),
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct End {
    pub cap: bool,
    pub taper: Taper,
}

impl Default for End {
    fn default() -> Self {
        Self {
            cap: true,
            taper: Taper::Length(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Options {
    /// Diâmetro base.
    pub size: f64,
    /// Quanto a pressão muda a largura (`0` = largura constante, `size / 2` de raio).
    pub thinning: f64,
    pub smoothing: f64,
    pub streamline: f64,
    /// Pressão inventada pela VELOCIDADE (o rato e qualquer dispositivo sem pressão).
    pub simulate_pressure: bool,
    pub start: End,
    pub end: End,
    /// O traço acabou (o último ponto entra tal como veio).
    pub last: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            size: 16.0,
            thinning: 0.5,
            smoothing: 0.5,
            streamline: 0.5,
            simulate_pressure: true,
            start: End::default(),
            end: End::default(),
            last: false,
        }
    }
}

/// Um ponto de entrada; `pressure` ausente vale `0.5` (`0.25` no primeiro), como na biblioteca.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Input {
    pub x: f64,
    pub y: f64,
    pub pressure: Option<f64>,
}

/// Um ponto alisado (`StrokePoint`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokePoint {
    pub point: P,
    pub pressure: f64,
    pub vector: P,
    pub distance: f64,
    pub running_length: f64,
}

const RATE_OF_PRESSURE_CHANGE: f64 = 0.275;
const FIXED_PI: f64 = PI + 1e-4;

fn add(a: P, b: P) -> P {
    [a[0] + b[0], a[1] + b[1]]
}
fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}
fn mul(a: P, n: f64) -> P {
    [a[0] * n, a[1] * n]
}
fn per(a: P) -> P {
    [a[1], -a[0]]
}
fn dot(a: P, b: P) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn neg(a: P) -> P {
    [-a[0], -a[1]]
}
fn len(a: P) -> f64 {
    a[0].hypot(a[1])
}
fn uni(a: P) -> P {
    let l = len(a);
    [a[0] / l, a[1] / l]
}
fn dist2(a: P, b: P) -> f64 {
    let d = sub(a, b);
    d[0] * d[0] + d[1] * d[1]
}
fn lrp(a: P, b: P, t: f64) -> P {
    add(a, mul(sub(b, a), t))
}
fn prj(a: P, b: P, c: f64) -> P {
    add(a, mul(b, c))
}
fn rot_around(a: P, c: P, r: f64) -> P {
    let (s, co) = (r.sin(), r.cos());
    let px = a[0] - c[0];
    let py = a[1] - c[1];
    [px * co - py * s + c[0], px * s + py * co + c[1]]
}

fn radius(size: f64, thinning: f64, pressure: f64) -> f64 {
    size * (0.5 - thinning * (0.5 - pressure))
}

fn taper_len(t: Taper, size: f64, total: f64) -> f64 {
    match t {
        Taper::Length(l) => l,
        Taper::Full => size.max(total),
    }
}

/// `getStrokePoints`: alisa (`streamline`) e mede. Os primeiros pontos até a distância percorrida
/// chegar a `size` são saltados (a biblioteca 1.2.0 faz assim; sem isto o início treme).
pub fn stroke_points(input: &[Input], o: &Options) -> Vec<StrokePoint> {
    if input.is_empty() {
        return Vec::new();
    }
    let t = 0.15 + (1.0 - o.streamline) * 0.85;
    let mut pts: Vec<Input> = input.to_vec();
    if pts.len() == 2 {
        let last = pts[1];
        pts.truncate(1);
        let first = [pts[0].x, pts[0].y];
        for i in 1..5 {
            let [x, y] = lrp(first, [last.x, last.y], f64::from(i) / 4.0);
            pts.push(Input {
                x,
                y,
                pressure: None,
            });
        }
    }
    if pts.len() == 1 {
        let p = pts[0];
        pts.push(Input {
            x: p.x + 1.0,
            y: p.y + 1.0,
            pressure: p.pressure,
        });
    }
    let pressure_of = |p: &Input, default: f64| match p.pressure {
        Some(v) if v >= 0.0 => v,
        _ => default,
    };
    let mut out = vec![StrokePoint {
        point: [pts[0].x, pts[0].y],
        pressure: pressure_of(&pts[0], 0.25),
        vector: [1.0, 1.0],
        distance: 0.0,
        running_length: 0.0,
    }];
    let mut has_reached_min = false;
    let mut running = 0.0;
    let mut prev = out[0];
    let max = pts.len() - 1;
    for (i, p) in pts.iter().enumerate().skip(1) {
        let point = if o.last && i == max {
            [p.x, p.y]
        } else {
            lrp(prev.point, [p.x, p.y], t)
        };
        if point == prev.point {
            continue;
        }
        let distance = (point[1] - prev.point[1]).hypot(point[0] - prev.point[0]);
        running += distance;
        if i < max && !has_reached_min {
            if running < o.size {
                continue;
            }
            has_reached_min = true;
        }
        prev = StrokePoint {
            point,
            pressure: pressure_of(p, 0.5),
            vector: uni(sub(prev.point, point)),
            distance,
            running_length: running,
        };
        out.push(prev);
    }
    out[0].vector = out.get(1).map_or([0.0, 0.0], |p| p.vector);
    out
}

/// `getStrokeOutlinePoints`: o polígono do traço (lado esquerdo, tampa do fim, lado direito ao
/// contrário, tampa do início).
pub fn outline(points: &[StrokePoint], o: &Options) -> Vec<P> {
    let size = o.size;
    if points.is_empty() || size <= 0.0 {
        return Vec::new();
    }
    let n = points.len();
    let total = points[n - 1].running_length;
    let taper_start = if o.start.taper == Taper::Length(0.0) {
        0.0
    } else {
        taper_len(o.start.taper, size, total)
    };
    let taper_end = if o.end.taper == Taper::Length(0.0) {
        0.0
    } else {
        taper_len(o.end.taper, size, total)
    };
    let min_dist = (size * o.smoothing).powi(2);
    let mut left: Vec<P> = Vec::new();
    let mut right: Vec<P> = Vec::new();
    let mut prev_pressure = points.iter().take(10).fold(points[0].pressure, |acc, cur| {
        let mut pressure = cur.pressure;
        if o.simulate_pressure {
            let sp = (cur.distance / size).min(1.0);
            let rp = (1.0 - sp).min(1.0);
            pressure = (acc + (rp - acc) * (sp * RATE_OF_PRESSURE_CHANGE)).min(1.0);
        }
        (acc + pressure) / 2.0
    });
    let mut r = radius(size, o.thinning, points[n - 1].pressure);
    let mut first_radius: Option<f64> = None;
    let mut prev_vector = points[0].vector;
    let mut pl = points[0].point;
    let mut pr = pl;
    let mut tl;
    let mut tr;
    let mut is_prev_sharp = false;
    for i in 0..n {
        let StrokePoint {
            point,
            vector,
            distance,
            running_length,
            ..
        } = points[i];
        let mut pressure = points[i].pressure;
        if i < n - 1 && total - running_length < 3.0 {
            continue;
        }
        if o.thinning != 0.0 {
            if o.simulate_pressure {
                let sp = (distance / size).min(1.0);
                let rp = (1.0 - sp).min(1.0);
                pressure = (prev_pressure + (rp - prev_pressure) * (sp * RATE_OF_PRESSURE_CHANGE))
                    .min(1.0);
            }
            r = radius(size, o.thinning, pressure);
        } else {
            r = size / 2.0;
        }
        if first_radius.is_none() {
            first_radius = Some(r);
        }
        let ts = if running_length < taper_start {
            let t = running_length / taper_start;
            t * (2.0 - t)
        } else {
            1.0
        };
        let te = if total - running_length < taper_end {
            let t = (total - running_length) / taper_end - 1.0;
            t * t * t + 1.0
        } else {
            1.0
        };
        r = (r * ts.min(te)).max(0.01);
        let next_vector = if i < n - 1 {
            points[i + 1].vector
        } else {
            vector
        };
        let next_dpr = if i < n - 1 {
            dot(vector, next_vector)
        } else {
            1.0
        };
        let prev_dpr = dot(vector, prev_vector);
        let is_point_sharp = prev_dpr < 0.0 && !is_prev_sharp;
        let is_next_sharp = next_dpr < 0.0;
        if is_point_sharp || is_next_sharp {
            let offset = mul(per(prev_vector), r);
            let step = 1.0 / 13.0;
            let mut t = 0.0;
            tl = pl;
            tr = pr;
            while t <= 1.0 {
                tl = rot_around(sub(point, offset), point, FIXED_PI * t);
                left.push(tl);
                tr = rot_around(add(point, offset), point, FIXED_PI * -t);
                right.push(tr);
                t += step;
            }
            pl = tl;
            pr = tr;
            if is_next_sharp {
                is_prev_sharp = true;
            }
            continue;
        }
        is_prev_sharp = false;
        if i == n - 1 {
            let offset = mul(per(vector), r);
            left.push(sub(point, offset));
            right.push(add(point, offset));
            continue;
        }
        let offset = mul(per(lrp(next_vector, vector, next_dpr)), r);
        tl = sub(point, offset);
        if i <= 1 || dist2(pl, tl) > min_dist {
            left.push(tl);
            pl = tl;
        }
        tr = add(point, offset);
        if i <= 1 || dist2(pr, tr) > min_dist {
            right.push(tr);
            pr = tr;
        }
        prev_pressure = pressure;
        prev_vector = vector;
    }
    let first = points[0].point;
    let last = if n > 1 {
        points[n - 1].point
    } else {
        add(points[0].point, [1.0, 1.0])
    };
    let mut start_cap = Vec::new();
    let mut end_cap = Vec::new();
    if n == 1 {
        if taper_start == 0.0 && taper_end == 0.0 || o.last {
            let start = prj(
                first,
                uni(per(sub(first, last))),
                -first_radius.filter(|&v| v != 0.0).unwrap_or(r),
            );
            let step = 1.0 / 13.0;
            let mut dot_pts = Vec::new();
            let mut t = step;
            while t <= 1.0 {
                dot_pts.push(rot_around(start, first, FIXED_PI * 2.0 * t));
                t += step;
            }
            return dot_pts;
        }
    } else {
        if taper_start == 0.0 {
            if o.start.cap {
                let step = 1.0 / 13.0;
                let mut t = step;
                while t <= 1.0 {
                    if let Some(&r0) = right.first() {
                        start_cap.push(rot_around(r0, first, FIXED_PI * t));
                    }
                    t += step;
                }
            } else if let (Some(&l0), Some(&r0)) = (left.first(), right.first()) {
                let corners = sub(l0, r0);
                let o1 = mul(corners, 0.5);
                let o2 = mul(corners, 0.51);
                start_cap.extend([
                    sub(first, o1),
                    sub(first, o2),
                    add(first, o2),
                    add(first, o1),
                ]);
            }
        }
        let direction = per(neg(points[n - 1].vector));
        if taper_end != 0.0 {
            end_cap.push(last);
        } else if o.end.cap {
            let start = prj(last, direction, r);
            let step = 1.0 / 29.0;
            let mut t = step;
            while t < 1.0 {
                end_cap.push(rot_around(start, last, FIXED_PI * 3.0 * t));
                t += step;
            }
        } else {
            end_cap.extend([
                add(last, mul(direction, r)),
                add(last, mul(direction, r * 0.99)),
                sub(last, mul(direction, r * 0.99)),
                sub(last, mul(direction, r)),
            ]);
        }
    }
    right.reverse();
    left.extend(end_cap);
    left.extend(right);
    left.extend(start_cap);
    left
}
