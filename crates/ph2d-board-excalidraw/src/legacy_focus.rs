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
//! Ele pára na ENTRADA do nível `gap`: um ponto fixo dentro da forma, do lado de LÁ da recta, não se
//! exprime — escreve-se o par cuja ponta ([`landing`]) cai mais perto (a entrada, ou o ponto do foco
//! do `gap 0`). O losango e o rectângulo redondo não têm a lei do vão medida: o nosso `g` por fora,
//! menos a distância ao contorno por dentro.

use ph2d_board_model::{Element, ShapeType};

use ph2d_board_geom::{inside, nearest_on_outline};

/// O menor `|gap|` que escrevemos: `gap 0` no 0.18.1 não é «no contorno» — põe a ponta DENTRO, no
/// ponto do foco (medido, `F_*_g0`).
const MIN_GAP: f64 = 0.01;

/// O `|focus|` escreve-se `UNTIE` vezes maior: os dois cantos de uma recta paralela a um eixo deixam
/// de empatar, e a ponta mexe ~`1e-4` px.
const UNTIE: f64 = 1.0 + 1e-6;

#[derive(Clone, Copy, PartialEq)]
enum Native {
    Rectangle,
    Ellipse,
    Diamond,
}

/// A forma no referencial dela: centro na origem, sem rotação.
struct Frame {
    kind: Native,
    round: bool,
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
            round: s.style.round,
            hw,
            hh,
            c: [t.x + hw, t.y + hh],
        })
    }

    fn local(&self, t: &Element, p: [f64; 2]) -> [f64; 2] {
        let q = t.unrotate(p);
        [q[0] - self.c[0], q[1] - self.c[1]]
    }

    /// A lei do vão está medida (o rectângulo de cantos vivos e a elipse).
    fn measured(&self) -> bool {
        self.kind == Native::Ellipse || (self.kind == Native::Rectangle && !self.round)
    }
}

/// `(focus, gap)` para a ponta `tip` de uma seta presa a `t`, cujo ponto vizinho é `from` (mundo).
/// `fallback` = o vão por fora onde a lei do vão do 0.18.1 não está medida.
pub(crate) fn legacy(t: &Element, from: [f64; 2], tip: [f64; 2], fallback: f64) -> (f64, f64) {
    let Some(fr) = Frame::of(t) else {
        return (0.0, fallback);
    };
    let (a, p) = (fr.local(t, from), fr.local(t, tip));
    let focus = focus_of(&fr, a, p);
    if !fr.measured() {
        let g = if inside(t, tip) {
            -nearest_on_outline(t, tip).map_or(0.0, |q| q.1)
        } else {
            fallback
        };
        return (focus, at_least_min(g));
    }
    let level = match fr.kind {
        Native::Ellipse => ellipse_gap(fr.hw, fr.hh, p),
        _ => Some((p[0].abs() - fr.hw).max(p[1].abs() - fr.hh)),
    }
    .map(at_least_min);
    let miss = |g: f64| landing_local(&fr, a, focus, g).map_or(f64::INFINITY, |q| dist(q, p));
    // O 1.º de dois iguais: o nível ganha ao ponto do foco.
    let gap = level
        .into_iter()
        .chain([0.0])
        .min_by(|x, y| miss(*x).total_cmp(&miss(*y)))
        .unwrap_or(0.0);
    (focus, gap)
}

/// ⭐ Onde o 0.18.1 põe a ponta ao re-ligar a ligação `{focus, gap}` a partir de `from` (mundo) — a
/// lei medida (XV7), no rectângulo de cantos vivos e na elipse; `None` noutras formas, ou quando a
/// recta do foco não toca o nível (ele deixa a ponta onde estava).
#[cfg(test)]
pub(crate) fn landing(t: &Element, from: [f64; 2], focus: f64, gap: f64) -> Option<[f64; 2]> {
    let fr = Frame::of(t).filter(Frame::measured)?;
    let q = landing_local(&fr, fr.local(t, from), focus, gap)?;
    Some(t.rotate([q[0] + fr.c[0], q[1] + fr.c[1]]))
}

/// A recta passa pelo canto `|focus|·K` que mais se afasta do centro do lado do sinal; a ponta é o
/// ponto do foco (`gap 0`) ou a ENTRADA da recta no nível `gap`.
fn landing_local(fr: &Frame, a: [f64; 2], focus: f64, gap: f64) -> Option<[f64; 2]> {
    let aim = if focus == 0.0 {
        [0.0, 0.0]
    } else {
        let (r, o) = (focus.abs(), focus.signum());
        let mut best = None;
        let mut most = 0.0;
        for k in [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]] {
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
    signed_distance(a, p) / support.max(f64::EPSILON) * UNTIE
}

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
mod tests {
    use super::*;
    use ph2d_board_model::{FracKey, Rgba, RichText, Shape, Style};

    /// ⭐ Um ponto fixo fundo numa elipse ACHATADA, junto ao eixo maior: o nível que passa por ele tem
    /// um semi-eixo abaixo de [`THINNEST`] (0,55 px) e recusa-se — fica o ponto do foco (`gap 0`). A
    /// elipse degenerada do 0.18.1 errou 9,8 px no oráculo (semi-eixo 6e-11, `X10_nosso` ph2d-19).
    #[test]
    fn a_level_thinner_than_a_pixel_is_refused() {
        let ink = Rgba([30, 30, 30, 255]);
        let el = Element::new_shape(
            ph2d_board_model::ElementId(1),
            FracKey::between(None, None),
            Shape {
                kind: ShapeType::Ellipse,
                style: Style::new(None, Some(ink), ink),
                text: RichText::plain(""),
            },
            [0.0, 0.0, 200.0, 20.0],
        );
        let (tip, from) = ([160.0, 9.6], [161.0, -190.0]);
        assert!(ellipse_gap(100.0, 10.0, [60.0, -0.4]).is_none());
        let (_, gap) = legacy(&el, from, tip, 6.0);
        assert_eq!(gap, 0.0, "o nível de meio pixel escreveu-se");
    }
}
