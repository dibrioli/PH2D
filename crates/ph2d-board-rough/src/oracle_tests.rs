//! A porta contra o ORÁCULO, por passo: o rough.js 4.6.4 e o perfect-freehand 1.2.0 corridos sobre
//! `entradas/portas.json` por `corre_portas.mjs` (2026-10-07), saídas em `saidas/portas_*.json`.
//! Cada op do rough.js com a MESMA semente; cada ponto alisado e cada ponto do contorno do
//! perfect-freehand. Tolerância `1e-9` relativa: o V8 e a libm podem diferir no último bit de um
//! `cos`/`hypot`, nunca mais.

use serde_json::Value;

use crate::freehand::{self, End, Input, Taper};
use crate::rough::{self, FillStyle, Op, Options, Seg, SetKind};

const INPUT: &str =
    include_str!("../../../docs/MiroClone/ferramentas/excalidraw_oracle/entradas/portas.json");
const ROUGH: &str =
    include_str!("../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/portas_rough.json");
const FREEHAND: &str = include_str!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/portas_freehand.json"
);

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

fn num(v: &Value) -> f64 {
    v.as_f64().expect("número")
}

fn point(v: &Value) -> [f64; 2] {
    [num(&v[0]), num(&v[1])]
}

fn points(v: &Value) -> Vec<[f64; 2]> {
    v.as_array()
        .expect("lista de pontos")
        .iter()
        .map(point)
        .collect()
}

fn options(v: &Value) -> Options {
    let mut o = Options::default();
    let f = |k: &str| v.get(k).and_then(Value::as_f64);
    let b = |k: &str| v.get(k).and_then(Value::as_bool).unwrap_or(false);
    o.seed = f("seed").expect("semente") as u32;
    if let Some(x) = f("roughness") {
        o.roughness = x;
    }
    if let Some(x) = f("bowing") {
        o.bowing = x;
    }
    if let Some(x) = f("strokeWidth") {
        o.stroke_width = x;
    }
    if let Some(x) = f("hachureGap") {
        o.hachure_gap = x;
    }
    if let Some(x) = f("hachureAngle") {
        o.hachure_angle = x;
    }
    o.preserve_vertices = b("preserveVertices");
    o.disable_multi_stroke = b("disableMultiStroke");
    o.fill = v.get("fill").is_some();
    o.fill_style = match v.get("fillStyle").and_then(Value::as_str) {
        None | Some("hachure") => FillStyle::Hachure,
        Some("solid") => FillStyle::Solid,
        Some("cross-hatch") => FillStyle::CrossHatch,
        Some(s) => panic!("preenchimento sem porta: {s}"),
    };
    o
}

/// Um `d` absoluto com `M L Q C Z` separados por espaços (o que as entradas escrevem).
fn parse_path(d: &str) -> Vec<Seg> {
    let mut out = Vec::new();
    let mut tokens: Vec<&str> = d.split_whitespace().collect();
    tokens.reverse();
    while let Some(cmd) = tokens.pop() {
        let mut take = |k: usize| -> Vec<f64> {
            (0..k)
                .map(|_| tokens.pop().expect("número").parse().expect("número"))
                .collect()
        };
        out.push(match cmd {
            "M" => {
                let v = take(2);
                Seg::M([v[0], v[1]])
            }
            "L" => {
                let v = take(2);
                Seg::L([v[0], v[1]])
            }
            "Q" => {
                let v = take(4);
                Seg::Q([v[0], v[1]], [v[2], v[3]])
            }
            "C" => {
                let v = take(6);
                Seg::C([v[0], v[1]], [v[2], v[3]], [v[4], v[5]])
            }
            "Z" => Seg::Z,
            other => panic!("comando sem porta: {other}"),
        });
    }
    out
}

fn op_numbers(op: &Op) -> (&'static str, Vec<f64>) {
    match *op {
        Op::Move(p) => ("move", p.to_vec()),
        Op::Line(p) => ("lineTo", p.to_vec()),
        Op::Cubic(a, b, c) => ("bcurveTo", vec![a[0], a[1], b[0], b[1], c[0], c[1]]),
    }
}

fn kind_name(k: SetKind) -> &'static str {
    match k {
        SetKind::Path => "path",
        SetKind::FillPath => "fillPath",
        SetKind::FillSketch => "fillSketch",
    }
}

#[test]
fn every_rough_case_matches_the_oracle_op_by_op_with_the_same_seed() {
    let input: Value = serde_json::from_str(INPUT).expect("entrada");
    let oracle: Value = serde_json::from_str(ROUGH).expect("oráculo");
    assert_eq!(oracle["meta"]["package_version"], "4.6.4");
    let cases = input["rough"].as_array().expect("casos");
    let expected = oracle["cases"].as_array().expect("casos");
    assert_eq!(cases.len(), expected.len());
    assert!(cases.len() >= 29, "a cobertura do oráculo não encolhe");
    for (case, want) in cases.iter().zip(expected) {
        let name = case["name"].as_str().expect("nome");
        assert_eq!(want["name"], name);
        let o = options(&case["options"]);
        let a = &case["args"];
        let n = |i: usize| num(&a[i]);
        let got = match case["kind"].as_str().expect("tipo") {
            "line" => rough::line(n(0), n(1), n(2), n(3), &o),
            "rectangle" => rough::rectangle(n(0), n(1), n(2), n(3), &o),
            "ellipse" => rough::ellipse(n(0), n(1), n(2), n(3), &o),
            "polygon" => rough::polygon(&points(&a[0]), &o),
            "linearPath" => rough::linear_path(&points(&a[0]), &o),
            "curve" => rough::curve(&points(&a[0]), &o),
            "path" => rough::path(&parse_path(a[0].as_str().expect("d")), &o),
            k => panic!("{name}: tipo sem porta {k}"),
        };
        let want_sets = want["sets"].as_array().expect("sets");
        assert_eq!(got.len(), want_sets.len(), "{name}: número de sets");
        for (si, (g, w)) in got.iter().zip(want_sets).enumerate() {
            assert_eq!(kind_name(g.kind), w["type"], "{name}: tipo do set {si}");
            let wops = w["ops"].as_array().expect("ops");
            assert_eq!(g.ops.len(), wops.len(), "{name}: set {si}: número de ops");
            for (oi, (gop, wop)) in g.ops.iter().zip(wops).enumerate() {
                let (gname, gnums) = op_numbers(gop);
                let wop = wop.as_array().expect("op");
                assert_eq!(gname, wop[0], "{name}: set {si} op {oi}");
                for (k, (x, y)) in gnums.iter().zip(&wop[1..]).enumerate() {
                    assert!(
                        close(*x, num(y)),
                        "{name}: set {si} op {oi} [{k}]: {x} ≠ oráculo {}",
                        num(y)
                    );
                }
            }
        }
    }
}

fn freehand_options(v: &Value) -> freehand::Options {
    let mut o = freehand::Options::default();
    let f = |k: &str| v.get(k).and_then(Value::as_f64);
    if let Some(x) = f("size") {
        o.size = x;
    }
    if let Some(x) = f("thinning") {
        o.thinning = x;
    }
    if let Some(x) = f("smoothing") {
        o.smoothing = x;
    }
    if let Some(x) = f("streamline") {
        o.streamline = x;
    }
    if let Some(x) = v.get("simulatePressure").and_then(Value::as_bool) {
        o.simulate_pressure = x;
    }
    o.last = v.get("last").and_then(Value::as_bool).unwrap_or(false);
    let end = |e: Option<&Value>| {
        let mut out = End::default();
        if let Some(e) = e {
            if let Some(c) = e.get("cap").and_then(Value::as_bool) {
                out.cap = c;
            }
            match e.get("taper") {
                Some(Value::Bool(true)) => out.taper = Taper::Full,
                Some(t) if t.is_number() => out.taper = Taper::Length(num(t)),
                _ => {}
            }
        }
        out
    };
    o.start = end(v.get("start"));
    o.end = end(v.get("end"));
    o
}

#[test]
fn every_freehand_case_matches_the_oracle_in_both_steps() {
    let input: Value = serde_json::from_str(INPUT).expect("entrada");
    let oracle: Value = serde_json::from_str(FREEHAND).expect("oráculo");
    assert_eq!(oracle["meta"]["package_version"], "1.2.0");
    let cases = input["freehand"].as_array().expect("casos");
    let expected = oracle["cases"].as_array().expect("casos");
    assert_eq!(cases.len(), expected.len());
    assert!(cases.len() >= 11, "a cobertura do oráculo não encolhe");
    for (case, want) in cases.iter().zip(expected) {
        let name = case["name"].as_str().expect("nome");
        let o = freehand_options(&case["options"]);
        let input: Vec<Input> = case["points"]
            .as_array()
            .expect("pontos")
            .iter()
            .map(|p| Input {
                x: num(&p[0]),
                y: num(&p[1]),
                pressure: p.get(2).map(num),
            })
            .collect();
        let sp = freehand::stroke_points(&input, &o);
        let wsp = want["stroke_points"].as_array().expect("passo 1");
        assert_eq!(sp.len(), wsp.len(), "{name}: passo 1, número de pontos");
        for (i, (g, w)) in sp.iter().zip(wsp).enumerate() {
            let got = [
                g.point[0],
                g.point[1],
                g.pressure,
                g.vector[0],
                g.vector[1],
                g.distance,
                g.running_length,
            ];
            for (k, x) in got.iter().enumerate() {
                assert!(
                    close(*x, num(&w[k])),
                    "{name}: passo 1 ponto {i} campo {k}: {x} ≠ oráculo {}",
                    num(&w[k])
                );
            }
        }
        let outline = freehand::outline(&sp, &o);
        let wo = points(&want["outline"]);
        assert_eq!(
            outline.len(),
            wo.len(),
            "{name}: passo 2, número de pontos do contorno"
        );
        for (i, (g, w)) in outline.iter().zip(&wo).enumerate() {
            assert!(
                close(g[0], w[0]) && close(g[1], w[1]),
                "{name}: passo 2 ponto {i}: {g:?} ≠ oráculo {w:?}"
            );
        }
    }
}

#[test]
fn the_same_seed_draws_the_same_stroke_and_another_seed_another() {
    let o = |seed| Options {
        seed,
        ..Options::default()
    };
    let a = rough::rectangle(0.0, 0.0, 100.0, 60.0, &o(7));
    assert_eq!(a, rough::rectangle(0.0, 0.0, 100.0, 60.0, &o(7)));
    assert_ne!(a, rough::rectangle(0.0, 0.0, 100.0, 60.0, &o(8)));
}

/// ⭐ A TERCEIRA passagem (ordem do dono, 07/10: *«a linha dá 2 voltas por desenho; coloque 3»*)
/// junta-se SEM mexer nas duas do rough.js: tirando-a, sobra o traço do rough.js ao último número.
#[test]
fn a_third_pass_adds_a_stroke_and_keeps_the_two_of_rough_js() {
    let two = |seed| Options {
        seed,
        ..Options::default()
    };
    let three = |seed| Options {
        passes: 3,
        ..two(seed)
    };
    // Segmentos: cada aresta traz um par (move + cúbica) a mais, a seguir às duas dela.
    let (a, b) = (
        rough::rectangle(0.0, 0.0, 160.0, 120.0, &two(9)),
        rough::rectangle(0.0, 0.0, 160.0, 120.0, &three(9)),
    );
    let edges = 4;
    assert_eq!(b[0].ops.len(), a[0].ops.len() / 2 * 3);
    for e in 0..edges {
        assert_eq!(
            b[0].ops[e * 6..e * 6 + 4],
            a[0].ops[e * 4..e * 4 + 4],
            "aresta {e}: as duas do rough.js"
        );
        assert_ne!(
            b[0].ops[e * 6 + 4..e * 6 + 6],
            a[0].ops[e * 4..e * 4 + 2],
            "a terceira é outra mão"
        );
    }
    // Elipse, curva e caminho de cúbicas: a terceira vem no FIM, o começo é o do rough.js.
    for (x, y) in [
        (
            rough::ellipse(80.0, 60.0, 160.0, 120.0, &two(5)),
            rough::ellipse(80.0, 60.0, 160.0, 120.0, &three(5)),
        ),
        (
            rough::curve(&[[0.0, 0.0], [80.0, 40.0], [160.0, 0.0]], &two(6)),
            rough::curve(&[[0.0, 0.0], [80.0, 40.0], [160.0, 0.0]], &three(6)),
        ),
        (
            rough::path(
                &[
                    Seg::M([0.0, 0.0]),
                    Seg::C([40.0, 0.0], [80.0, 60.0], [120.0, 60.0]),
                ],
                &two(7),
            ),
            rough::path(
                &[
                    Seg::M([0.0, 0.0]),
                    Seg::C([40.0, 0.0], [80.0, 60.0], [120.0, 60.0]),
                ],
                &three(7),
            ),
        ),
    ] {
        let (x, y) = (&x.last().unwrap().ops, &y.last().unwrap().ops);
        assert!(y.len() > x.len(), "há uma passagem a mais");
        assert_eq!(
            &y[..x.len()],
            &x[..],
            "as duas primeiras são as do rough.js"
        );
    }
}
