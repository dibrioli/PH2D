//! As curvas do pincel em JSON: falloff, pegada e taper (filho de [`super`], LOC cap). Mesmas leis do pai.

use super::valores::{de_nome, f, inteiro, nome, num};
use ph2d_painter_brush::{FalloffCurve, FalloffPoint, FootprintCurve, HandleType};
use serde_json::{Value, json};

pub(super) fn curva_para_json(c: &FalloffCurve) -> Value {
    Value::Array(
        c.points()
            .iter()
            .map(|p| json!({"id": p.id, "x": num(p.x), "y": num(p.y), "handle": nome(p.handle)}))
            .collect(),
    )
}

pub(super) fn curva_de_json(v: Option<&Value>, base: FalloffCurve) -> FalloffCurve {
    let Some(a) = v.and_then(Value::as_array) else {
        return base;
    };
    let pts: Vec<FalloffPoint> = a
        .iter()
        .filter_map(Value::as_object)
        .map(|o| FalloffPoint {
            id: inteiro(o.get("id"), 0u8),
            x: f(o.get("x"), 0.0),
            y: f(o.get("y"), 0.0),
            handle: de_nome(
                o.get("handle"),
                [HandleType::Auto, HandleType::Vector],
                HandleType::Auto,
            ),
        })
        .collect();
    FalloffCurve::from_points(&pts).unwrap_or(base)
}

pub(super) fn pegada_para_json(c: FootprintCurve) -> Value {
    Value::Array(
        c.rows()
            .iter()
            .map(|r| Value::Array(r.iter().map(|&x| num(x)).collect()))
            .collect(),
    )
}

pub(super) fn pegada_de_json(v: Option<&Value>, base: FootprintCurve) -> FootprintCurve {
    let Some(a) = v.and_then(Value::as_array) else {
        return base;
    };
    let mut rows = base.rows();
    for (r, linha) in rows.iter_mut().zip(a) {
        let l = linha.as_array();
        for (i, x) in r.iter_mut().enumerate() {
            *x = f(l.and_then(|l| l.get(i)), *x);
        }
    }
    FootprintCurve::from_rows(rows)
}

/// O taper: as três chaves de sempre, e a cauda e o fall-off (L5) só quando armados — o preset sem eles é o de
/// antes, ao byte.
pub(super) fn taper_para_json(t: &ph2d_painter_brush::taper::Taper) -> Value {
    let mut v = json!({
        "start": num(t.start),
        "tip_start": num(t.tip_start),
        "opacity": num(t.opacity),
    });
    if let Some(o) = v.as_object_mut() {
        for (k, x) in [
            ("end", t.end),
            ("tip_end", t.tip_end),
            ("fall_off", t.fall_off),
            ("fall_off_start", t.fall_off_start),
        ] {
            if x != 0.0 {
                o.insert(k.to_owned(), num(x));
            }
        }
    }
    v
}
