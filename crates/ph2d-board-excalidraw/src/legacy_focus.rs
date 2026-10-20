//! ⭐ **O legado `{focus, gap}` como o 0.18.1 o LÊ** (W10, estudo 07 XV7 — medido no oráculo:
//! `saidas/w10_F_*.0181.json`, `mede_w10.py`). Ao re-ligar uma seta (a forma mexeu), o 0.18.1 põe a
//! ponta na recta que sai do ponto VIZINHO da seta e passa a `focus · h(n)` do centro da forma (com
//! sinal; `h` a função de suporte na normal da recta, no referencial da forma: a da CAIXA no rectângulo
//! e na elipse, a do losango no losango), no 1.º ponto a `gap` da forma — o rectângulo alargado de
//! `gap` (cantos vivos) ou a elipse de semi-eixos + `gap`; `gap` negativo entra na forma. Escrevemos o
//! par que o faz pôr a ponta onde a escrevemos.
//!
//! A recta passa por `|focus|·K`, `K` o canto da caixa (o vértice do losango) do lado do sinal — e numa
//! recta PARALELA a um eixo (a uma aresta do losango) dois cantos empatam e ele escolhe o do outro
//! lado: a ponta saía espelhada (medido, `F_*_eixos_*`). [`UNTIE`] desfaz o empate.
//!
//! O losango (vivo e redondo) e o rectângulo redondo têm a lei das PEÇAS (W23 B9, estudo 07 XV7;
//! `mede_w23_vao.py`, 28 188 setas): [`pieces`] e [`landing_pieces`].
//!
//! Ele pára na ENTRADA do nível `gap`: um ponto fixo dentro da forma, do lado de LÁ da recta, não se
//! exprime — escreve-se o par cuja ponta ([`landing`]) cai mais perto.

use ph2d_board_model::{Element, ShapeType};

/// O menor `|gap|` que escrevemos: `gap 0` no 0.18.1 não é «no contorno» — põe a ponta DENTRO, no
/// ponto do foco (medido, `F_*_g0`).
const MIN_GAP: f64 = 0.01;

/// O `|focus|` escreve-se `UNTIE` vezes maior: os dois cantos de uma recta paralela a um eixo deixam
/// de empatar, e a ponta mexe ~`1e-4` px.
const UNTIE: f64 = 1.0 + 1e-6;

/// Abaixo disto a ponta da lei É a nossa (o par exprime-a); acima procura-se o vão de menor erro.
const REACHED: f64 = 1e-6;

#[derive(Clone, Copy, PartialEq)]
enum Native {
    Rectangle,
    Ellipse,
    Diamond,
}

/// A forma no referencial dela: centro na origem, sem rotação. `round` = o `roundness.type` escrito.
struct Frame {
    kind: Native,
    round: Option<u8>,
    w: f64,
    h: f64,
    hw: f64,
    hh: f64,
    c: [f64; 2],
}

impl Frame {
    fn of(t: &Element) -> Option<Self> {
        let s = t.shape()?;
        let kind = match s.kind {
            ShapeType::Rectangle
            | ShapeType::Sticky
            | ShapeType::StickyStack
            | ShapeType::StickyWide => Native::Rectangle,
            ShapeType::Ellipse => Native::Ellipse,
            ShapeType::Diamond => Native::Diamond,
            _ => return None,
        };
        let (hw, hh) = (t.w / 2.0, t.h / 2.0);
        Some(Self {
            kind,
            round: s.style.round.then(|| crate::write::roundness_type(s.kind)),
            w: t.w,
            h: t.h,
            hw,
            hh,
            c: [t.x + hw, t.y + hh],
        })
    }

    fn local(&self, t: &Element, p: [f64; 2]) -> [f64; 2] {
        let q = t.unrotate(p);
        [q[0] - self.c[0], q[1] - self.c[1]]
    }

    /// A lei das peças (W23): o losango e o rectângulo redondo.
    fn by_pieces(&self) -> bool {
        self.kind == Native::Diamond || (self.kind == Native::Rectangle && self.round.is_some())
    }
}

/// `(focus, gap)` para a ponta `tip` de uma seta presa a `t`, cujo ponto vizinho é `from` (mundo).
/// `fallback` = o vão das formas que ele não tem.
pub(crate) fn legacy(t: &Element, from: [f64; 2], tip: [f64; 2], fallback: f64) -> (f64, f64) {
    let Some(fr) = Frame::of(t) else {
        return (0.0, fallback);
    };
    let (a, p) = (fr.local(t, from), fr.local(t, tip));
    let focus = focus_of(&fr, a, p);
    let level = match fr.kind {
        _ if fr.by_pieces() => pieces_level(&fr, p),
        Native::Ellipse => ellipse_gap(fr.hw, fr.hh, p),
        _ => Some((p[0].abs() - fr.hw).max(p[1].abs() - fr.hh)),
    }
    .map(at_least_min);
    let miss = |g: f64| landing_local(&fr, a, focus, g).map_or(f64::INFINITY, |q| dist(q, p));
    // O 1.º de dois iguais: o nível ganha ao ponto do foco.
    let mut best = level
        .into_iter()
        .chain([0.0])
        .map(|g| (miss(g), g))
        .min_by(|x, y| x.0.total_cmp(&y.0))
        .unwrap_or((f64::INFINITY, 0.0));
    if fr.by_pieces() && best.0 > REACHED {
        // Inexprimível (do lado de LÁ, ou a corda curta num só canto ⇒ o ponto do foco): o vão de
        // menor erro (estudo 07 XV7, W23 B9).
        best = nearest_gap(&fr, &miss, best);
    }
    (focus, best.1)
}

/// O vão de menor erro numa varredura de `[-max, max]` da meia-caixa, afinada à volta do melhor.
fn nearest_gap(fr: &Frame, miss: &dyn Fn(f64) -> f64, mut best: (f64, f64)) -> (f64, f64) {
    const N: usize = 256;
    let span = fr.hw.max(fr.hh);
    let step = 2.0 * span / N as f64;
    let at = |g: f64, best: &mut (f64, f64)| {
        let g = at_least_min(g);
        let m = miss(g);
        if m < best.0 {
            *best = (m, g);
        }
    };
    for i in 0..=N {
        at(-span + step * i as f64, &mut best);
    }
    let (mut lo, mut hi) = (best.1 - step, best.1 + step);
    for _ in 0..50 {
        let (m1, m2) = (lo + (hi - lo) / 3.0, hi - (hi - lo) / 3.0);
        at(m1, &mut best);
        at(m2, &mut best);
        if miss(at_least_min(m1)) < miss(at_least_min(m2)) {
            hi = m2;
        } else {
            lo = m1;
        }
    }
    best
}

/// ⭐ Onde o 0.18.1 põe a ponta ao re-ligar a ligação `{focus, gap}` a partir de `from` (mundo) — a
/// lei medida (XV7); `None` noutras formas, ou quando a recta do foco não toca o nível (ele deixa a
/// ponta onde estava).
#[cfg(test)]
pub(crate) fn landing(t: &Element, from: [f64; 2], focus: f64, gap: f64) -> Option<[f64; 2]> {
    let fr = Frame::of(t)?;
    landing_in(&fr, t, from, focus, gap)
}

/// [`landing`] com o `roundness.type` dado (o oráculo escreve o 3 num losango; nós o 2).
#[cfg(test)]
pub(crate) fn landing_typed(
    t: &Element,
    round: Option<u8>,
    from: [f64; 2],
    focus: f64,
    gap: f64,
) -> Option<[f64; 2]> {
    let mut fr = Frame::of(t)?;
    fr.round = round;
    landing_in(&fr, t, from, focus, gap)
}

#[cfg(test)]
fn landing_in(fr: &Frame, t: &Element, from: [f64; 2], focus: f64, gap: f64) -> Option<[f64; 2]> {
    let q = landing_local(fr, fr.local(t, from), focus, gap)?;
    Some(t.rotate([q[0] + fr.c[0], q[1] + fr.c[1]]))
}

/// A recta passa pelo `|focus|·K` que mais se afasta do centro do lado do sinal; a ponta é o ponto
/// do foco (`gap 0`) ou a ENTRADA da recta no nível `gap`.
fn landing_local(fr: &Frame, a: [f64; 2], focus: f64, gap: f64) -> Option<[f64; 2]> {
    let aim = if focus == 0.0 {
        [0.0, 0.0]
    } else {
        let (r, o) = (focus.abs(), focus.signum());
        let ks = if fr.kind == Native::Diamond {
            [[0.0, -1.0], [1.0, 0.0], [0.0, 1.0], [-1.0, 0.0]]
        } else {
            [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
        };
        let mut best = None;
        let mut most = 0.0;
        for k in ks {
            let q = [k[0] * fr.hw * r, k[1] * fr.hh * r];
            let s = o * signed_distance(a, q);
            if s > most {
                most = s;
                best = Some(q);
            }
        }
        best?
    };
    if gap == 0.0 {
        return Some(aim);
    }
    if fr.by_pieces() {
        return landing_pieces(fr, a, aim, gap);
    }
    let (ex, ey) = (fr.hw + gap, fr.hh + gap);
    if ex <= 0.0 || ey <= 0.0 {
        return None;
    }
    let d = [aim[0] - a[0], aim[1] - a[1]];
    let t = if fr.kind == Native::Ellipse {
        let qa = (d[0] / ex).powi(2) + (d[1] / ey).powi(2);
        let qb = 2.0 * (a[0] * d[0] / (ex * ex) + a[1] * d[1] / (ey * ey));
        let qc = (a[0] / ex).powi(2) + (a[1] / ey).powi(2) - 1.0;
        let disc = qb * qb - 4.0 * qa * qc;
        if qa <= 0.0 || disc < 0.0 {
            return None;
        }
        (-qb - disc.sqrt()) / (2.0 * qa)
    } else {
        let (mut lo, mut hi) = (f64::NEG_INFINITY, f64::INFINITY);
        for (ak, dk, e) in [(a[0], d[0], ex), (a[1], d[1], ey)] {
            if dk == 0.0 {
                if ak.abs() > e {
                    return None;
                }
                continue;
            }
            let (t1, t2) = ((-e - ak) / dk, (e - ak) / dk);
            lo = lo.max(t1.min(t2));
            hi = hi.min(t1.max(t2));
        }
        if hi < lo {
            return None;
        }
        lo
    };
    (t >= 0.0).then(|| [a[0] + t * d[0], a[1] + t * d[1]])
}

/// Uma peça do contorno alargado: um lado ou um canto (cúbica).
#[derive(Clone, Copy)]
enum Piece {
    Line([f64; 2], [f64; 2]),
    Cubic([[f64; 2]; 4]),
}

/// O raio do canto do 0.18.1 (`type 3` = o adaptativo, tecto [`ADAPTIVE_RADIUS`] acima de
/// [`ADAPTIVE_FROM`]; `type 2` = sempre um quarto).
fn corner_radius(x: f64, round: u8) -> f64 {
    if round == 3 && x > ADAPTIVE_FROM {
        ADAPTIVE_RADIUS
    } else {
        x / 4.0
    }
}
const ADAPTIVE_FROM: f64 = 128.0;
const ADAPTIVE_RADIUS: f64 = 32.0;

/// ⭐ As peças da forma alargada de `g` (W23, medido a ≤ 0,0022 px em 28 188 setas): o losango tem os
/// vértices em `⌊w/2⌋+1`, `⌊h/2⌋+1` (não no meio); vivo, cada vértice + `g` no SEU eixo; redondo,
/// cantos cúbicos (os dois controlos no vértice) de `R(⌊w/2⌋+1)` × `R(⌊h/2⌋+1)` transladados de
/// `g·V/|V|`; o rectângulo redondo, cantos quadráticos de `R(min(w, h))` transladados de
/// `(±g(hw+g), ±g(hh+g)) / |(hw+g, hh+g)|`. Os lados ligam as pontas dos cantos.
fn pieces(fr: &Frame, g: f64) -> Vec<Piece> {
    let (hw, hh) = (fr.hw, fr.hh);
    let corners: Vec<[[f64; 2]; 4]> = match (fr.kind, fr.round) {
        (Native::Diamond, None) => {
            let (ox, oy) = diamond_offset(fr);
            let v = [[ox, -hh - g], [hw + g, oy], [ox, hh + g], [-hw - g, oy]];
            return (0..4).map(|i| Piece::Line(v[i], v[(i + 1) % 4])).collect();
        }
        (Native::Diamond, Some(round)) => {
            let (ox, oy) = diamond_offset(fr);
            let (vr, hr) = (corner_radius(hw + ox, round), corner_radius(hh + oy, round));
            let (t, r, b, l) = ([ox, -hh], [hw, oy], [ox, hh], [-hw, oy]);
            [
                ([t[0] - vr, t[1] + hr], t, [t[0] + vr, t[1] + hr]),
                ([r[0] - vr, r[1] - hr], r, [r[0] - vr, r[1] + hr]),
                ([b[0] + vr, b[1] - hr], b, [b[0] - vr, b[1] - hr]),
                ([l[0] + vr, l[1] + hr], l, [l[0] + vr, l[1] - hr]),
            ]
            .into_iter()
            .map(|(a, v, b)| {
                let n = v[0].hypot(v[1]);
                let m = [g * v[0] / n, g * v[1] / n];
                [a, v, v, b].map(|q| [q[0] + m[0], q[1] + m[1]])
            })
            .collect()
        }
        _ => {
            let r = corner_radius(fr.w.min(fr.h), fr.round.unwrap_or(3));
            let n = (hw + g).hypot(hh + g);
            let u = [g * (hw + g) / n, g * (hh + g) / n];
            [(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)]
                .into_iter()
                .map(|(sx, sy)| {
                    let v = [sx * hw, sy * hh];
                    let (a, b) = if sx * sy < 0.0 {
                        ([v[0] - sx * r, v[1]], [v[0], v[1] - sy * r])
                    } else {
                        ([v[0], v[1] - sy * r], [v[0] - sx * r, v[1]])
                    };
                    let lift = |e: [f64; 2]| {
                        [
                            e[0] + 2.0 / 3.0 * (v[0] - e[0]),
                            e[1] + 2.0 / 3.0 * (v[1] - e[1]),
                        ]
                    };
                    [a, lift(a), lift(b), b].map(|q| [q[0] + sx * u[0], q[1] + sy * u[1]])
                })
                .collect()
        }
    };
    let mut out: Vec<Piece> = corners.iter().map(|c| Piece::Cubic(*c)).collect();
    for i in 0..4 {
        out.push(Piece::Line(corners[i][3], corners[(i + 1) % 4][0]));
    }
    out
}

/// O vértice do losango do 0.18.1 está em `⌊w/2⌋+1`, `⌊h/2⌋+1` do canto da caixa.
fn diamond_offset(fr: &Frame) -> (f64, f64) {
    (
        (fr.w / 2.0).floor() + 1.0 - fr.hw,
        (fr.h / 2.0).floor() + 1.0 - fr.hh,
    )
}

/// ⭐ A re-ligação nas peças: a recta de `a` pelo ponto do foco como SEGMENTO longo a partir de `a`;
/// cada peça dá no máximo UM ponto; ≥ 2 ⇒ o mais perto de `a`; 1 ⇒ o PONTO DO FOCO (a corda que entra
/// e sai pelo mesmo canto); 0 ⇒ a ponta não mexe (`None`).
fn landing_pieces(fr: &Frame, a: [f64; 2], aim: [f64; 2], g: f64) -> Option<[f64; 2]> {
    let d = [aim[0] - a[0], aim[1] - a[1]];
    let len = d[0].hypot(d[1]);
    if len < 1e-12 {
        return None;
    }
    let reach = a[0].hypot(a[1]) + 2.0 * (fr.w.max(fr.h) + g.abs());
    let q = [a[0] + d[0] / len * reach, a[1] + d[1] / len * reach];
    let hits: Vec<[f64; 2]> = pieces(fr, g)
        .iter()
        .filter_map(|p| match *p {
            Piece::Line(u, v) => cross_segments(u, v, a, q),
            Piece::Cubic(c) => cross_cubic(&c, a, q),
        })
        .collect();
    match hits.len() {
        0 => None,
        1 => Some(aim),
        _ => hits
            .into_iter()
            .min_by(|x, y| dist(*x, a).total_cmp(&dist(*y, a))),
    }
}

/// O ponto de `u → v` no segmento `p → q` (extremos incluídos).
fn cross_segments(u: [f64; 2], v: [f64; 2], p: [f64; 2], q: [f64; 2]) -> Option<[f64; 2]> {
    let d = [q[0] - p[0], q[1] - p[1]];
    let e = [v[0] - u[0], v[1] - u[1]];
    let den = d[0] * e[1] - d[1] * e[0];
    if den.abs() < 1e-12 {
        return None;
    }
    let w = [u[0] - p[0], u[1] - p[1]];
    let s = (w[0] * e[1] - w[1] * e[0]) / den;
    let k = (w[0] * d[1] - w[1] * d[0]) / den;
    ((0.0..=1.0).contains(&s) && (0.0..=1.0).contains(&k))
        .then(|| [p[0] + s * d[0], p[1] + s * d[1]])
}

fn bez(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let m = 1.0 - t;
    let (b0, b1, b2, b3) = (m * m * m, 3.0 * m * m * t, 3.0 * m * t * t, t * t * t);
    [0, 1].map(|k| b0 * c[0][k] + b1 * c[1][k] + b2 * c[2][k] + b3 * c[3][k])
}

fn dbez(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let m = 1.0 - t;
    [0, 1].map(|k| {
        3.0 * m * m * (c[1][k] - c[0][k])
            + 6.0 * m * t * (c[2][k] - c[1][k])
            + 3.0 * t * t * (c[3][k] - c[2][k])
    })
}

/// UM cruzamento da cúbica com o segmento `p → q`, como o 0.18.1 o acha (medido, `mede_w23_vao.py`):
/// Newton em `(t, s)` a partir das sementes 0,5 · 0,2 · 0,8, aceite a 1e-3 com `t, s ∈ [0, 1]`; o
/// ponto aceite afina-se pelo mesmo Newton.
fn cross_cubic(c: &[[f64; 2]; 4], p: [f64; 2], q: [f64; 2]) -> Option<[f64; 2]> {
    let d = [q[0] - p[0], q[1] - p[1]];
    let off = |t: f64, s: f64| {
        let b = bez(c, t);
        [b[0] - p[0] - s * d[0], b[1] - p[1] - s * d[1]]
    };
    let step = |t: &mut f64, s: &mut f64| -> bool {
        let f = off(*t, *s);
        let db = dbez(c, *t);
        let det = db[0] * -d[1] - -d[0] * db[1];
        if det == 0.0 {
            return false;
        }
        *t += (-f[0] * -d[1] + f[1] * -d[0]) / det;
        *s += (-db[0] * f[1] + db[1] * f[0]) / det;
        true
    };
    for seed in [0.5, 0.2, 0.8] {
        let (mut t, mut s) = (seed, 0.0);
        let mut ok = false;
        for _ in 0..10 {
            if !step(&mut t, &mut s) {
                break;
            }
            let f = off(t, s);
            if f[0].abs() + f[1].abs() < 1e-3 {
                ok = true;
                break;
            }
        }
        if ok && (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&s) {
            let (mut tp, mut sp) = (t, s);
            for _ in 0..30 {
                if !step(&mut tp, &mut sp) {
                    break;
                }
            }
            let good = (tp - t).abs() < 1e-2 && off(tp, sp).iter().all(|e| e.abs() < 1e-9);
            return Some(bez(c, if good { tp } else { t }));
        }
    }
    None
}

/// O `g` cujo contorno ([`pieces`]) passa por `p`: o contorno cresce com `g` e é estrelado no centro,
/// então o raio dele na direcção de `p` sobe com `g` — bissecção.
fn pieces_level(fr: &Frame, p: [f64; 2]) -> Option<f64> {
    let r = p[0].hypot(p[1]);
    if r < 1e-9 {
        return None;
    }
    let u = [p[0] / r, p[1] / r];
    let f = |g: f64| radius_along(fr, g, u).map_or(-r, |x| x - r);
    // O nível mais perto de 0: muito para dentro os cantos cruzam-se e o contorno deixa de crescer.
    let step = fr.hw.max(fr.hh) / 64.0;
    let (floor, ceil) = (-fr.hw.min(fr.hh), r + fr.hw.max(fr.hh));
    let (mut lo, mut hi) = (0.0, 0.0);
    if f(0.0) > 0.0 {
        while f(lo) > 0.0 {
            hi = lo;
            lo -= step;
            if lo < floor {
                return None;
            }
        }
    } else {
        while f(hi) <= 0.0 {
            lo = hi;
            hi += step;
            if hi > ceil {
                return None;
            }
        }
    }
    for _ in 0..64 {
        let mid = (lo + hi) / 2.0;
        if f(mid) > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// A distância do centro ao contorno de `g` na direcção `u` (o cruzamento mais longe).
fn radius_along(fr: &Frame, g: f64, u: [f64; 2]) -> Option<f64> {
    let far = 4.0 * (fr.w + fr.h + g.abs());
    let q = [u[0] * far, u[1] * far];
    let n = [-u[1], u[0]];
    let mut best: Option<f64> = None;
    let mut keep = |x: [f64; 2]| {
        let along = x[0] * u[0] + x[1] * u[1];
        if along >= 0.0 && best.is_none_or(|b| along > b) {
            best = Some(along);
        }
    };
    for piece in pieces(fr, g) {
        match piece {
            Piece::Line(a, b) => {
                if let Some(x) = cross_segments(a, b, [0.0, 0.0], q) {
                    keep(x);
                }
            }
            Piece::Cubic(c) => {
                let h = |t: f64| {
                    let b = bez(&c, t);
                    b[0] * n[0] + b[1] * n[1]
                };
                const K: usize = 16;
                for i in 0..K {
                    let (mut lo, mut hi) = (i as f64 / K as f64, (i + 1) as f64 / K as f64);
                    let (hl, hh) = (h(lo), h(hi));
                    if hl == 0.0 {
                        keep(bez(&c, lo));
                    }
                    if hl * hh >= 0.0 {
                        continue;
                    }
                    for _ in 0..52 {
                        let mid = (lo + hi) / 2.0;
                        if (h(mid) > 0.0) == (hl > 0.0) {
                            lo = mid;
                        } else {
                            hi = mid;
                        }
                    }
                    keep(bez(&c, (lo + hi) / 2.0));
                }
            }
        }
    }
    best
}

/// `d / h(n)`: a distância com sinal do centro à recta `a → p` pelo suporte da caixa (do losango).
fn focus_of(fr: &Frame, a: [f64; 2], p: [f64; 2]) -> f64 {
    let u = [p[0] - a[0], p[1] - a[1]];
    let len = u[0].hypot(u[1]);
    if len < 1e-12 {
        return 0.0;
    }
    let (nx, ny) = ((u[1] / len).abs(), (u[0] / len).abs());
    let support = match fr.kind {
        Native::Diamond => (fr.hw * nx).max(fr.hh * ny),
        _ => fr.hw * nx + fr.hh * ny,
    };
    let d = signed_distance(a, p);
    if d.abs() < THROUGH_CENTRE {
        return 0.0; // a recta do centro: `focus 0` (um foco de 1e-16 não acha o canto do sinal)
    }
    d / support.max(f64::EPSILON) * UNTIE
}

/// Uma recta a menos disto do centro passa pelo centro.
const THROUGH_CENTRE: f64 = 1e-9;

/// A distância com sinal do centro (a origem) à recta `a → q`.
fn signed_distance(a: [f64; 2], q: [f64; 2]) -> f64 {
    let u = [q[0] - a[0], q[1] - a[1]];
    (u[0] * -a[1] - u[1] * -a[0]) / u[0].hypot(u[1]).max(f64::EPSILON)
}

fn at_least_min(g: f64) -> f64 {
    if g.abs() < MIN_GAP {
        MIN_GAP.copysign(g)
    } else {
        g
    }
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// O `g` da elipse de semi-eixos `(a + g, b + g)` que passa por `p` (o 1.º membro decresce com `g`);
/// `None` se só a passa abaixo de [`THINNEST`] num semi-eixo (perto do eixo maior, fundo: a elipse
/// dele degenera — medido, 9,8 px de erro com um semi-eixo de 6e-11).
fn ellipse_gap(a: f64, b: f64, p: [f64; 2]) -> Option<f64> {
    let f = |g: f64| (p[0] / (a + g)).powi(2) + (p[1] / (b + g)).powi(2) - 1.0;
    let floor = THINNEST - a.min(b);
    if f(floor) <= 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (floor, p[0].hypot(p[1]));
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if f(mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// O semi-eixo mais fino que se escreve no nível da elipse (1 px).
const THINNEST: f64 = 1.0;

#[cfg(test)]
#[path = "legacy_focus_tests.rs"]
mod tests;
