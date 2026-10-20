//! ⭐ **W9 — a fidelidade ao Excalidraw** ([estudo 07](../../../docs/MiroClone/07_fidelidade_excalidraw.md)
//! §6.2). Os gates leem as SAÍDAS do oráculo (`ferramentas/excalidraw_oracle/saidas/w9_*`, o `next`
//! 0.18.0-143b5b6 — o do excalidraw.com — salvo onde se diz), gravadas com cabeçalho; nada a re-correr.

use super::*;
use ph2d_board_model::{
    Anchor, Board, BoardOp, BoardSet, Connector, Element, ElementKind, End, FillStyle, Head,
    Letter, Rgba, RichText, Route, Shape, ShapeType, Style, TextAlign, VAlign,
};
use ph2d_board_route::RouteCache;
use ph2d_vector::PathEl;
use serde_json::{Value, json};

const DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/MiroClone/ferramentas/excalidraw_oracle"
);

fn saida(caso: &str, versao: &str) -> Value {
    let p = format!("{DIR}/saidas/w9_{caso}.{versao}.json");
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")))
        .unwrap()
}

/// Os elementos depois do ÚLTIMO passo `tipo`.
fn passo<'a>(d: &'a Value, tipo: &str) -> &'a [Value] {
    d["passos"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|p| p["passo"]["tipo"] == tipo)
        .and_then(|p| p["elementos"].as_array())
        .map_or(&[], Vec::as_slice)
}

fn abre(els: &[Value]) -> Import {
    read(&json!({ "elements": els }).to_string()).unwrap()
}

fn world(e: &Value) -> Vec<[f64; 2]> {
    let (x, y) = (e["x"].as_f64().unwrap(), e["y"].as_f64().unwrap());
    e["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| [p[0].as_f64().unwrap() + x, p[1].as_f64().unwrap() + y])
        .collect()
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// As setas do documento importado, pela ordem do ficheiro (o z segue o array).
fn setas(doc: &ph2d_board_model::BoardDoc) -> Vec<&Element> {
    doc.live_in_z_order()
        .into_iter()
        .filter(|e| e.connector().is_some())
        .collect()
}

fn rotas(doc: &ph2d_board_model::BoardDoc) -> RouteCache {
    let mut r = RouteCache::default();
    r.sync(doc);
    r
}

/// Os grupos do SVG dele: `(translação, os `d` dos caminhos)` — a régua do `mede_w9.py`.
fn svg_groups(caso: &str) -> Vec<([f64; 2], Vec<String>)> {
    let s = std::fs::read_to_string(format!("{DIR}/saidas/w9_{caso}.next.svg")).unwrap();
    let mut out = Vec::new();
    for chunk in s.split("<g transform=\"translate(").skip(1) {
        let Some((t, rest)) = chunk.split_once(')') else {
            continue;
        };
        let n = numbers(t);
        let body = rest.split("</g>").next().unwrap_or("");
        let ds = body
            .split("<path d=\"")
            .skip(1)
            .filter_map(|p| p.split('"').next())
            .map(str::to_owned)
            .collect();
        out.push(([n[0], n[1]], ds));
    }
    out
}

fn numbers(s: &str) -> Vec<f64> {
    let mut out = Vec::new();
    let mut tok = String::new();
    for ch in s.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_digit() || ch == '.' || ch == 'e' || (ch == '-' && tok.is_empty()) {
            tok.push(ch);
        } else {
            if let Ok(v) = tok.parse() {
                out.push(v);
            }
            tok.clear();
            if ch == '-' {
                tok.push(ch);
            }
        }
    }
    out
}

/// O 1.º sub-caminho de um `d` (o rough.js com rugosidade 0 desenha cada traço duas vezes).
fn first_subpath(d: &str) -> Vec<f64> {
    numbers(d.split('M').find(|p| !p.trim().is_empty()).unwrap_or(""))
}

/// Os caminhos de pontas de cada seta (os grupos com a MESMA translação depois do fuste).
fn svg_heads(caso: &str, n: usize) -> Vec<([f64; 2], Vec<String>)> {
    let gs = svg_groups(caso);
    let mut out = Vec::new();
    let mut i = 0;
    for _ in 0..n {
        let (t, _) = gs[i].clone();
        i += 1;
        let mut heads = Vec::new();
        while i < gs.len() && gs[i].0 == t {
            heads.extend(gs[i].1.iter().cloned());
            i += 1;
        }
        out.push((t, heads));
    }
    out
}

/// A caixa `[x0, y0, x1, y1]` de uma lista de números `x y x y …`.
fn bbox(n: &[f64]) -> [f64; 4] {
    let mut b = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
    for p in n.chunks(2).filter(|p| p.len() == 2) {
        b = [
            b[0].min(p[0]),
            b[1].min(p[1]),
            b[2].max(p[0]),
            b[3].max(p[1]),
        ];
    }
    b
}

fn path_points(p: &ph2d_vector::BezPath) -> Vec<f64> {
    let mut v = Vec::new();
    for el in p.elements() {
        let pts: Vec<ph2d_vector::Point> = match *el {
            PathEl::MoveTo(a) | PathEl::LineTo(a) => vec![a],
            PathEl::QuadTo(a, b) => vec![a, b],
            PathEl::CurveTo(a, b, c) => vec![a, b, c],
            PathEl::ClosePath => vec![],
        };
        for q in pts {
            v.extend([q.x, q.y]);
        }
    }
    v
}

/// ⭐⭐ §6.2-1 — o FICHEIRO DO DONO (`entradas/w9_dono_bv.excalidraw`, smoke 10/10): as três setas
/// «redondas» de dois pontos abrem RECTAS (a curva dele, XA1), presas em órbita nas seis pontas, e
/// as pontas roteadas caem onde o excalidraw.com as gravou a ≤ 0,05 px (XB1). ⛔ Antes da W9: um S de
/// 41–83 px e pontas até 45 px DENTRO das formas.
#[test]
fn the_owners_file_opens_straight_and_stops_where_excalidraw_put_the_tips() {
    let file = std::fs::read_to_string(format!("{DIR}/entradas/w9_dono_bv.excalidraw")).unwrap();
    let theirs: Value = serde_json::from_str(&file).unwrap();
    let arrows: Vec<&Value> = theirs["elements"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "arrow")
        .collect();
    let imp = read(&file).unwrap();
    let ours = setas(&imp.doc);
    assert_eq!(ours.len(), 3);
    let routes = rotas(&imp.doc);
    for (a, el) in arrows.iter().zip(&ours) {
        let c = el.connector().unwrap();
        assert_eq!(c.route, Route::Spline, "a «redonda» dele");
        assert!(c.waypoints.is_empty());
        for e in c.ends() {
            assert!(
                matches!(
                    e,
                    End::Bound {
                        anchor: Anchor::Orbit(_),
                        ..
                    }
                ),
                "{e:?}"
            );
        }
        let r = routes.get(el.id).unwrap();
        let pts = world(a);
        let [p0, p1] = r.ends();
        let (e0, e1) = (dist(p0, pts[0]), dist(p1, pts[pts.len() - 1]));
        assert!(e0 <= 0.05 && e1 <= 0.05, "{}: {e0:.3} / {e1:.3}", a["id"]);
        // Recta: as pegas na corda.
        let d = ph2d_board_route::drawn(r, c.heads, c.style.stroke_width);
        let n = path_points(&d.line);
        let (u, l) = ([p1[0] - p0[0], p1[1] - p0[1]], dist(p0, p1));
        for q in n.chunks(2) {
            let off = ((q[0] - p0[0]) * u[1] - (q[1] - p0[1]) * u[0]).abs() / l;
            assert!(off < 1e-9, "a seta curva-se: {off}");
        }
    }
}

/// ⭐⭐ §6.2-2 — XB1/XB2/XB3 contra o next em TODOS os casos `B_*` (depois de ele re-ligar as setas:
/// `empurrar`): a ponta presa em `orbit` pára no 1.º ponto a `5 + traço da forma/2` na recta entre os
/// focos (rect e elipse a ≤ 0,05; o losango a ≤ 1, a assimetria dele recusada em W1); `inside` = o
/// foco; `skip` = `orbit`; foco fora ⇒ o foco.
#[test]
fn xb1_every_bound_tip_lands_where_the_next_puts_it() {
    let mut casos: Vec<String> = std::fs::read_dir(format!("{DIR}/saidas"))
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter_map(|n| {
            n.strip_prefix("w9_B_")?
                .strip_suffix(".next.json")
                .map(|c| format!("B_{c}"))
        })
        .collect();
    casos.sort();
    assert!(casos.len() >= 20, "{casos:?}");
    let mut medidos = 0;
    let mut falhas = Vec::new();
    for caso in &casos {
        let d = saida(caso, "next");
        let els = passo(&d, "empurrar");
        let imp = abre(els);
        let routes = rotas(&imp.doc);
        let arrows: Vec<&Value> = els.iter().filter(|e| e["type"] == "arrow").collect();
        for (a, el) in arrows.iter().zip(setas(&imp.doc)) {
            let r = routes.get(el.id).unwrap();
            let pts = world(a);
            for (k, key) in ["startBinding", "endBinding"].into_iter().enumerate() {
                let Some(target) = a[key]["elementId"].as_str() else {
                    continue;
                };
                let shape = els.iter().find(|e| e["id"] == target).unwrap();
                let want = if k == 0 { pts[0] } else { pts[pts.len() - 1] };
                let got = r.ends()[k];
                let tol = if shape["type"] == "diamond" {
                    // O losango DELE tem os vértices em ⌊w/2⌋+1 (a assimetria recusada em W1, estudo
                    // §3): as arestas dele e as nossas diferem até 1 px na normal ⇒ ao longo da seta,
                    // 1/sen θ (θ = o ângulo entre a seta e a aresta). A NOSSA lei é exacta: a ponta a
                    // `g` do NOSSO contorno.
                    let our = imp.doc.live_in_z_order().into_iter().find(|e| {
                        e.shape().is_some_and(|s| s.kind == ShapeType::Diamond)
                            && (e.x - shape["x"].as_f64().unwrap()).abs() < 1e-9
                            && (e.y - shape["y"].as_f64().unwrap()).abs() < 1e-9
                    });
                    let our = our.expect("o nosso losango");
                    let (_, d) = ph2d_board_geom::nearest_on_outline(our, got).unwrap();
                    let inside = ph2d_board_geom::inside(our, got);
                    // `inside`, ou o foco fora da faixa: a ponta É o foco (XB1/XB2).
                    let focus = match el.connector().unwrap().ends()[k] {
                        End::Bound {
                            anchor: Anchor::Fixed(uv) | Anchor::Orbit(uv),
                            ..
                        } => dist(got, ph2d_board_route::fixed_world(our, uv)) < 1e-6,
                        _ => false,
                    };
                    if !focus && !inside && (d - ph2d_board_route::gap(our)).abs() > 1e-3 {
                        falhas.push(format!("{caso} losango: o vão é {d:.4}"));
                    }
                    let c = our.center();
                    let edge = [
                        our.w / 2.0 * if got[0] < c[0] { 1.0 } else { -1.0 },
                        our.h / 2.0 * if got[1] < c[1] { -1.0 } else { 1.0 },
                    ];
                    let other = if k == 0 { pts[1] } else { pts[pts.len() - 2] };
                    let line = [got[0] - other[0], got[1] - other[1]];
                    let sin = (edge[0] * line[1] - edge[1] * line[0]).abs()
                        / (edge[0].hypot(edge[1]) * line[0].hypot(line[1]));
                    1.0 / sin.max(1e-3) + 0.05
                } else {
                    0.05
                };
                medidos += 1;
                if dist(got, want) > tol {
                    falhas.push(format!(
                        "{caso} {} {key}: nós {got:?} · ele {want:?} ({:.3})",
                        shape["type"],
                        dist(got, want)
                    ));
                }
            }
        }
    }
    assert!(medidos >= 60, "{medidos}");
    assert!(
        falhas.is_empty(),
        "{} de {medidos}:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}

/// ⭐ XV2 — o LEGADO do 0.18.1 (`{focus, gap}`, sem `fixedPoint`): em órbita do foco para onde a
/// recta da estação vizinha pela ponta gravada aponta; com a seta a apontar ao centro, o centro — e a
/// ponta cai onde o next a põe depois de re-ligar (o `gap` gravado deita-se fora, recusa §3).
#[test]
fn xv2_a_legacy_binding_orbits_the_focus_its_tip_aims_at() {
    // O `_diag` aponta FORA do centro (a prova de mutação W9 viu o foco forçado ao centro passar nos dois
    // horizontais: a recta deles já passa pelo centro).
    for caso in [
        "B_legado_gap1",
        "B_legado_focus05_gap10",
        "B_legado_focus05_gap10_diag",
    ] {
        let legacy = saida(caso, "0181");
        let els = passo(&legacy, "restore");
        assert!(els.iter().any(|e| e["endBinding"]["gap"].is_number()));
        let imp = abre(els);
        let routes = rotas(&imp.doc);
        let next = saida(caso, "next");
        let after = passo(&next, "empurrar");
        let theirs: Vec<&Value> = after.iter().filter(|e| e["type"] == "arrow").collect();
        for (a, el) in theirs.iter().zip(setas(&imp.doc)) {
            let End::Bound { anchor, .. } = el.connector().unwrap().end else {
                panic!("solta");
            };
            assert!(matches!(anchor, Anchor::Orbit(_)), "{anchor:?}");
            let shape = after
                .iter()
                .find(|e| e["id"] == a["endBinding"]["elementId"]);
            if shape.is_some_and(|s| s["type"] == "diamond") {
                continue;
            }
            let pts = world(a);
            let got = routes.get(el.id).unwrap().ends()[1];
            let e = dist(got, pts[pts.len() - 1]);
            assert!(
                e <= 0.05,
                "{caso}: nós {got:?} · next {:?} ({e:.3})",
                pts[pts.len() - 1]
            );
        }
    }
}

/// ⭐⭐ §6.2-3 — XA2: a ponta `arrow` mede `min(25, L/2)` em QUALQUER espessura (39 setas, L 8…400 ×
/// traço 1/2/4): a profundidade dela ao longo da linha é a da risca dele (a 20°). ⛔ Antes da W9 a
/// caixa era fixa e L ≤ 20 ficava SEM ponta.
#[test]
fn xa2_the_arrow_head_shrinks_to_half_the_last_leg() {
    let d = saida("P_comprimento", "next");
    let els = passo(&d, "restore");
    let imp = abre(els);
    let routes = rotas(&imp.doc);
    let heads = svg_heads("P_comprimento", els.len());
    let ours = setas(&imp.doc);
    assert_eq!(ours.len(), 39);
    for ((e, el), (_, theirs)) in els.iter().zip(&ours).zip(&heads) {
        let n = first_subpath(&theirs[0]);
        let risca = (n[0] - n[n.len() - 2]).hypot(n[1] - n[n.len() - 1]);
        let want = risca * 20f64.to_radians().cos();
        let c = el.connector().unwrap();
        let dr = ph2d_board_route::drawn(routes.get(el.id).unwrap(), c.heads, c.style.stroke_width);
        assert_eq!(dr.heads.len(), 1, "{}: a seta curta TEM ponta", e["id"]);
        // As duas riscas: do 1.º e do último ponto do «V» ao bico (o fim da rota).
        let p = path_points(&dr.heads[0].0);
        let (b1, b2) = ([p[0], p[1]], [p[p.len() - 2], p[p.len() - 1]]);
        let tip = routes.get(el.id).unwrap().ends()[1];
        let got = dist(tip, [(b1[0] + b2[0]) / 2.0, (b1[1] + b2[1]) / 2.0]);
        assert!(
            (got - want).abs() <= 0.05,
            "{}: nós {got:.3} · ele {want:.3}",
            e["id"]
        );
    }
}

/// ⭐ XA2 em cada TIPO (`P_tipos_comprimento`: barra, círculo, triângulo, losango e três de
/// cardinalidade × L 10…100): o tamanho ao longo da linha — o fundo do triângulo, o comprimento do
/// losango, o diâmetro do círculo, a meia barra — encolhe como o dele. (A abertura é a do catálogo:
/// só o tamanho se compara; as de cardinalidade, que são DELE, comparam-se inteiras.)
#[test]
fn xa2_every_head_type_has_his_size_and_shrinks_the_same() {
    let d = saida("P_tipos_comprimento", "next");
    let els = passo(&d, "restore");
    let imp = abre(els);
    let routes = rotas(&imp.doc);
    let heads = svg_heads("P_tipos_comprimento", els.len());
    let mut falhas = Vec::new();
    for ((e, el), (t, theirs)) in els.iter().zip(setas(&imp.doc)).zip(&heads) {
        let c = el.connector().unwrap();
        let r = routes.get(el.id).unwrap();
        let o = r.ends()[0];
        let dr = ph2d_board_route::drawn(r, c.heads, c.style.stroke_width);
        let mut ours: Vec<[f64; 4]> = dr
            .heads
            .iter()
            .map(|(p, _)| {
                let b = ph2d_vector::Shape::bounding_box(p);
                [b.x0 - o[0], b.y0 - o[1], b.x1 - o[0], b.y1 - o[1]]
            })
            .collect();
        let mut his: Vec<[f64; 4]> = theirs
            .iter()
            .flat_map(|d| d.split('M').filter(|s| !s.trim().is_empty()).map(numbers))
            .map(|n| bbox(&n))
            .collect();
        let _ = t;
        let union = |v: &[[f64; 4]]| {
            v.iter()
                .fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |a, b| {
                    [
                        a[0].min(b[0]),
                        a[1].min(b[1]),
                        a[2].max(b[2]),
                        a[3].max(b[3]),
                    ]
                })
        };
        let (u, h) = (union(&ours), union(&his));
        let kind = e["endArrowhead"].as_str().unwrap_or("");
        let ok = match kind {
            // Ao longo da linha (x): o fundo/comprimento/diâmetro.
            "triangle" | "diamond" | "circle" => ((u[2] - u[0]) - (h[2] - h[0])).abs() <= 0.1,
            // A meia barra (y).
            "bar" => ((u[3] - u[1]) - (h[3] - h[1])).abs() <= 0.1,
            // As de cardinalidade: as peças, uma a uma.
            _ => {
                ours.sort_by(|a, b| a.partial_cmp(b).unwrap());
                his.sort_by(|a, b| a.partial_cmp(b).unwrap());
                his.dedup_by(|a, b| a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 1e-9));
                ours.len() == his.len()
                    && ours
                        .iter()
                        .zip(&his)
                        .all(|(a, b)| a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 0.1))
            }
        };
        if !ok {
            falhas.push(format!("{}: nós {ours:?} · ele {his:?}", e["id"]));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// ⭐ As pontas de CARDINALIDADE inteiras (`P_tipos`, L 200 e 30): uma, muitos, um ou muitos,
/// exactamente um, zero ou um, zero ou muitos — o nome antigo (`crowfoot_*`) e o novo lêem-se iguais.
#[test]
fn the_cardinality_heads_draw_like_his() {
    let d = saida("P_tipos", "next");
    let els = passo(&d, "restore");
    let imp = abre(els);
    let routes = rotas(&imp.doc);
    let heads = svg_heads("P_tipos", els.len());
    let mut vistos = 0;
    for ((e, el), (_, theirs)) in els.iter().zip(setas(&imp.doc)).zip(&heads) {
        let id = e["id"].as_str().unwrap();
        if !(id.contains("cardinality") || id.contains("crowfoot")) {
            continue;
        }
        vistos += 1;
        let c = el.connector().unwrap();
        let r = routes.get(el.id).unwrap();
        let o = r.ends()[0];
        let dr = ph2d_board_route::drawn(r, c.heads, c.style.stroke_width);
        let mut ours: Vec<[f64; 4]> = dr
            .heads
            .iter()
            .map(|(p, _)| {
                let b = ph2d_vector::Shape::bounding_box(p);
                [b.x0 - o[0], b.y0 - o[1], b.x1 - o[0], b.y1 - o[1]]
            })
            .collect();
        let mut his: Vec<[f64; 4]> = theirs.iter().map(|d| bbox(&first_subpath(d))).collect();
        ours.sort_by(|a, b| a.partial_cmp(b).unwrap());
        his.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // O rough.js pinta o círculo duas vezes, em dois caminhos.
        his.dedup_by(|a, b| a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 0.05));
        assert_eq!(ours.len(), his.len(), "{id}: {ours:?} · {his:?}");
        for (a, b) in ours.iter().zip(&his) {
            assert!(
                a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 0.1),
                "{id}: nós {a:?} · ele {b:?}"
            );
        }
    }
    assert_eq!(vistos, 18);
}

/// ⭐⭐ §6.2-4 — XA1: a seta «redonda» de DOIS pontos é recta (as pegas a 1/6 da corda) e a de TRÊS é
/// o Catmull-Rom dele — as pegas do SVG dele (rugosidade 0) ao centésimo; a linha «redonda» também.
#[test]
fn xa1_the_round_arrow_is_his_curve_through_his_points() {
    let d = saida("F_fuste", "next");
    let els = passo(&d, "restore");
    let imp = abre(els);
    let routes = rotas(&imp.doc);
    let gs = svg_groups("F_fuste");
    let mut shafts = Vec::new();
    for g in &gs {
        if shafts
            .last()
            .is_none_or(|(t, _): &([f64; 2], Vec<String>)| *t != g.0)
        {
            shafts.push(g.clone());
        }
    }
    let mut vistos = 0;
    for ((e, el), (_, ds)) in els.iter().zip(setas(&imp.doc)).zip(&shafts) {
        if e["roughness"] != 0 || e["roundness"].is_null() {
            continue;
        }
        vistos += 1;
        let c = el.connector().unwrap();
        assert_eq!(c.route, Route::Spline, "{}", e["id"]);
        let r = routes.get(el.id).unwrap();
        let o = r.ends()[0];
        let line = ph2d_vec_render_bez(r);
        let ours: Vec<f64> = path_points(&line)
            .chunks(2)
            .flat_map(|p| [p[0] - o[0], p[1] - o[1]])
            .collect();
        let his = first_subpath(&ds[0]);
        assert_eq!(ours.len(), his.len(), "{}", e["id"]);
        for (a, b) in ours.iter().zip(&his) {
            assert!((a - b).abs() <= 0.006, "{}: {ours:?} · {his:?}", e["id"]);
        }
    }
    assert_eq!(vistos, 3);
}

/// O caminho da rota (sem pontas) como `BezPath`.
fn ph2d_vec_render_bez(r: &ph2d_board_route::Routed) -> ph2d_vector::BezPath {
    ph2d_board_route::drawn(r, [Head::None; 2], 1.0).line
}

#[path = "fidelity_file_tests.rs"]
mod file;

#[path = "fidelity_w10_tests.rs"]
mod w10;

#[path = "fidelity_w23_tests.rs"]
mod w23;

#[path = "fidelity_bloco6_tests.rs"]
mod bloco6;
