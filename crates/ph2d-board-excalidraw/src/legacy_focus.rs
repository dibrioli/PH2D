//! ⭐ **O legado `{focus, gap}` como o 0.18.1 o LÊ** (W10, estudo 07 XV4 — medido no oráculo:
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

use ph2d_board_model::{Element, ShapeType};

/// O menor `|gap|` que escrevemos: `gap 0` no 0.18.1 não é «no contorno» — põe a ponta DENTRO, no
/// ponto do foco (medido, `F_*_g0`).
const MIN_GAP: f64 = 0.01;

/// O `|focus|` escreve-se `UNTIE` vezes maior: os dois cantos de uma recta paralela a um eixo deixam
/// de empatar, e a ponta mexe ~`1e-4` px.
const UNTIE: f64 = 1.0 + 1e-6;

enum Native {
    Rectangle,
    Ellipse,
    Diamond,
}

/// `(focus, gap)` para a ponta `tip` de uma seta presa a `t`, cujo ponto vizinho é `from` (mundo).
/// `fallback` = o vão que se escreve onde a lei do vão do 0.18.1 não está medida (o losango e o
/// rectângulo de cantos redondos: o resíduo medido está no estudo).
pub(crate) fn legacy(t: &Element, from: [f64; 2], tip: [f64; 2], fallback: f64) -> (f64, f64) {
    let Some(s) = t.shape() else {
        return (0.0, fallback);
    };
    let kind = match s.kind {
        ShapeType::Rectangle
        | ShapeType::Sticky
        | ShapeType::StickyStack
        | ShapeType::StickyWide => Native::Rectangle,
        ShapeType::Ellipse => Native::Ellipse,
        ShapeType::Diamond => Native::Diamond,
        _ => return (0.0, fallback),
    };
    let (hw, hh) = (t.w / 2.0, t.h / 2.0);
    let c = [t.x + hw, t.y + hh];
    let local = |p: [f64; 2]| {
        let q = t.unrotate(p);
        [q[0] - c[0], q[1] - c[1]]
    };
    let (a, p) = (local(from), local(tip));
    let u = [p[0] - a[0], p[1] - a[1]];
    let len = u[0].hypot(u[1]);
    let focus = if len < 1e-12 {
        0.0
    } else {
        let signed = (u[0] * -a[1] - u[1] * -a[0]) / len;
        let (nx, ny) = ((u[1] / len).abs(), (u[0] / len).abs());
        let support = match kind {
            Native::Diamond => (hw * nx).max(hh * ny),
            _ => hw * nx + hh * ny,
        };
        signed / support.max(f64::EPSILON) * UNTIE
    };
    let gap = match kind {
        Native::Rectangle if !s.style.round => (p[0].abs() - hw).max(p[1].abs() - hh),
        Native::Ellipse => ellipse_gap(hw, hh, p),
        _ => fallback,
    };
    let gap = if gap.abs() < MIN_GAP {
        MIN_GAP.copysign(gap)
    } else {
        gap
    };
    (focus, gap)
}

/// O `g` da elipse de semi-eixos `(a + g, b + g)` que passa por `p` (o 1.º membro decresce com `g`).
fn ellipse_gap(a: f64, b: f64, p: [f64; 2]) -> f64 {
    let f = |g: f64| (p[0] / (a + g)).powi(2) + (p[1] / (b + g)).powi(2) - 1.0;
    let (mut lo, mut hi) = (-a.min(b) * (1.0 - 1e-12), p[0].hypot(p[1]));
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if f(mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}
