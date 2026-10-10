//! A LIGAÇÃO de uma seta lida de um `.excalidraw` (W9, as duas gramáticas — estudo 07 XV1–XV3; cortada
//! do `read.rs` no fecho, tecto de LOC).

use std::collections::BTreeMap;

use ph2d_board_model::{Anchor, ElementId, End};
use serde_json::Value;

use crate::style::num;

/// Os ids dele → os nossos, e os elementos dele pelo id.
pub(crate) type Links<'a, 'v> = (
    &'a BTreeMap<String, ElementId>,
    &'a BTreeMap<&'v str, &'v Value>,
);

/// ⭐ **A ponta de uma seta pela ligação dele** (W9, as duas gramáticas — estudo 07 XV1–XV3), ou solta
/// no ponto `at`; `next` = a estação vizinha (o outro ponto da recta). `mode "inside"` ⇒ o ponto fixo (a
/// ponta É o ponto, XB2); `orbit`/`skip` ⇒ em órbita do `fixedPoint` (XB1, XB3); só o legado `{focus,
/// gap}` (0.18.1) ⇒ em órbita do foco que a recta da estação vizinha pela ponta gravada aponta (o que o
/// next faz, XV2; o `gap` gravado deita-se fora — recusa §3). O cotovelo continua no ponto fixo/centro.
pub(crate) fn end_of(
    v: &Value,
    key: &str,
    at: [f64; 2],
    next: [f64; 2],
    links: Links<'_, '_>,
) -> End {
    let b = &v[key];
    let Some(target) = b["elementId"].as_str().and_then(|t| links.0.get(t)) else {
        return End::Free(at);
    };
    let fp = b["fixedPoint"]
        .as_array()
        .filter(|f| f.len() == 2)
        .map(|f| [f[0].as_f64().unwrap_or(0.5), f[1].as_f64().unwrap_or(0.5)]);
    let anchor = if v["elbowed"].as_bool().unwrap_or(false) {
        fp.map_or(Anchor::Center, Anchor::Fixed)
    } else {
        match (b["mode"].as_str(), fp) {
            (Some("inside"), Some(fp)) => Anchor::Fixed(fp),
            (_, Some(fp)) => Anchor::Orbit(fp),
            (_, None) => Anchor::Orbit(
                b["elementId"]
                    .as_str()
                    .and_then(|t| links.1.get(t))
                    .map_or([0.5, 0.5], |t| focus_through(t, next, at)),
            ),
        }
    };
    End::Bound {
        target: *target,
        anchor,
    }
}

/// O foco (fracção da caixa de `shape`, antes de rodar) para onde a recta de `from` pela ponta
/// gravada `tip` aponta: o ponto dela mais perto do centro da forma, nunca aquém da ponta. A ponta
/// sobre a recta que vem do centro dá o centro (`focus 0` ⇒ `[0.5, 0.5]`, como o next).
fn focus_through(shape: &Value, from: [f64; 2], tip: [f64; 2]) -> [f64; 2] {
    let (x, y) = (
        num(shape, "x").unwrap_or(0.0),
        num(shape, "y").unwrap_or(0.0),
    );
    let (w, h) = (
        num(shape, "width").unwrap_or(0.0),
        num(shape, "height").unwrap_or(0.0),
    );
    if w <= 0.0 || h <= 0.0 {
        return [0.5, 0.5];
    }
    let c = [x + w / 2.0, y + h / 2.0];
    let d = [tip[0] - from[0], tip[1] - from[1]];
    let l2 = d[0] * d[0] + d[1] * d[1];
    let q = if l2 < 1e-18 {
        c
    } else {
        let t = (((c[0] - from[0]) * d[0] + (c[1] - from[1]) * d[1]) / l2).max(1.0);
        [from[0] + d[0] * t, from[1] + d[1] * t]
    };
    let q = ph2d_board_model::rotate_about(q, c, -num(shape, "angle").unwrap_or(0.0));
    [
        ((q[0] - x) / w).clamp(0.0, 1.0),
        ((q[1] - y) / h).clamp(0.0, 1.0),
    ]
}
