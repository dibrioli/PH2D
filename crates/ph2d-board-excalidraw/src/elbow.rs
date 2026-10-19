//! W23 B5(1) — o segmento do meio do cotovelo de ida e volta com o Excalidraw ([doc 13]). Ele guarda
//! `fixedSegments: [{index, start, end}]` (coordenadas LOCAIS à seta; `index` = o último ponto do
//! segmento) e nós a fracção do vão: ler mede a fracção do segmento dele contra a NOSSA rota já
//! resolvida (o vão depende das caixas); escrever dá-lhe o nosso segmento do meio onde ele está.
//! Um segmento fixo dele que não é o nosso do meio (o 1.º trecho, `C11` do oráculo) não tem par: fica.
//!
//! [doc 13]: ../../../docs/MiroClone/13_trecho_do_cotovelo.md

use std::collections::BTreeMap;

use ph2d_board_model::{BoardDoc, BoardOp, Connector, ElementId};
use ph2d_board_route::{RouteCache, Routed};
use serde_json::{Value, json};

use crate::style::num;

const EPS: f64 = 1e-6;

fn pt(v: &Value) -> Option<[f64; 2]> {
    Some([v[0].as_f64()?, v[1].as_f64()?])
}

/// Depois de montado o documento: cada seta cotovelo dele com um segmento fixo que é o NOSSO do meio
/// (a mesma orientação, dentro do vão) fica com a fracção dele — salvo se já trazia a nossa.
pub(crate) fn pin(doc: &mut BoardDoc, kept: &[&Value], map: &BTreeMap<String, ElementId>) {
    let mut routes = RouteCache::default();
    routes.sync(doc);
    let mut puts = Vec::new();
    for v in kept {
        if !v["elbowed"].as_bool().unwrap_or(false) {
            continue;
        }
        let Some(id) = map.get(v["id"].as_str().unwrap_or("")) else {
            continue;
        };
        let Some(e) = routes.get(*id).and_then(|r| r.elbows.first().copied().flatten()) else {
            continue;
        };
        let (x, y) = (num(v, "x").unwrap_or(0.0), num(v, "y").unwrap_or(0.0));
        let ax = usize::from(!e.vertical);
        let ours = v["fixedSegments"].as_array().into_iter().flatten().find_map(|s| {
            let (a, b) = (pt(&s["start"])?, pt(&s["end"])?);
            let c = [x + (a[0] + b[0]) / 2.0, y + (a[1] + b[1]) / 2.0];
            let lo = e.span[0].min(e.span[1]);
            let hi = e.span[0].max(e.span[1]);
            ((a[ax] - b[ax]).abs() < EPS && c[ax] > lo && c[ax] < hi).then(|| e.frac(c))
        });
        let Some(frac) = ours else { continue };
        let Some(mut el) = doc.get(*id).cloned() else {
            continue;
        };
        if let Some(c) = el.connector_mut().filter(|c| c.mid(0).is_none()) {
            c.set_mid(0, Some(frac));
            puts.push(el);
        }
    }
    for el in puts {
        BoardOp::Put(el).apply(doc);
    }
}

/// O `fixedSegments` que escrevemos: o segmento do meio de cada trecho que o artista fixou, pelos
/// pontos escritos `pts` (mundo) relativos a `(x0, y0)`; `null` quando nenhum.
pub(crate) fn fixed_segments(c: &Connector, r: &Routed, pts: &[[f64; 2]], x0: f64, y0: f64) -> Value {
    let segs: Vec<Value> = (0..r.elbows.len())
        .filter(|leg| c.mid(*leg).is_some())
        .filter_map(|leg| r.elbows[leg])
        .filter_map(|e| {
            let k = (1..pts.len()).find(|k| {
                let (a, b) = (pts[k - 1], pts[*k]);
                let m = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
                (m[0] - e.at[0]).abs() < EPS && (m[1] - e.at[1]).abs() < EPS
            })?;
            let rel = |p: [f64; 2]| json!([p[0] - x0, p[1] - y0]);
            Some(json!({ "index": k, "start": rel(pts[k - 1]), "end": rel(pts[k]) }))
        })
        .collect();
    if segs.is_empty() {
        Value::Null
    } else {
        Value::Array(segs)
    }
}

#[cfg(test)]
#[path = "elbow_tests.rs"]
mod tests;
