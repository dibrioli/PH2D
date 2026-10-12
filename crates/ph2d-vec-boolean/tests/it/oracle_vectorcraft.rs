//! **O VectorCraft como ORÁCULO corrido por script** das nossas booleanas e do Offset (CLAUDE.md
//! §0.9, prompt 49 W0).
//!
//! O corpus é NOSSO (as formas do catálogo `ShapeKind` contra uma elipse, mais os casos que
//! partem um motor: aresta partilhada, toque num ponto, coincidentes, círculos tangentes, laço
//! auto-intersectante); o `vectorcraft-cli` 0.9.0 (MIT OU Apache-2.0, `fe8af48f`) corre sobre ele
//! por `docs/Vector Module/ferramentas/vectorcraft_2026-10-11/oraculo_vc.py`, e a saída dele fica
//! gravada em `fixtures/vectorcraft_oracle.tsv` (cabeçalho: origem, comando, data). O gate lê a
//! fixture — o VectorCraft não precisa de estar instalado para ele correr.
//!
//! A comparação é **por passo** (cada operação de cada caso) e pela régua amostrada de
//! [`super::raster`]: `xor ≤ 0,5 %` da área de referência = concordam. Onde discordam, a tabela
//! [`DISAGREE`] diz quais e porquê — e o gate falha nos dois sentidos, como a da estrela.
//! ⚠️ O oráculo não é a verdade: onde ele erra (o Bevel estilhaçado da estrela, estudo §3) a
//! resposta certa é a analítica, e a discordância fica registada com o lado certo.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use kurbo::{BezPath, Point};
use ph2d_vec_boolean::{BoolOp, apply, offset_path};
use ph2d_vec_scene::{
    ALL_SHAPES, FillRule, LineJoin, OffsetSide, ShapeKind, VecPath, VecVertex, cook,
};

use super::raster::{
    Grid, bbox, bez_polylines, count, rasterize, rasterize_paths, vecpath_bez, xor_count,
};

/// Documento → SVG: `px = 100·x + 200` (sem espelho: os dois vivem em y-para-baixo).
const SVG_SCALE: f64 = 100.0;
const SVG_OFFSET: f64 = 200.0;
const GRID_N: usize = 400;
const OK_PCT: f64 = 0.5;

/// As quatro booleanas do Pathfinder dele, pela ordem `fundo → frente` (= `a`, `b`).
pub const OPS: [(&str, BoolOp); 4] = [
    ("unite", BoolOp::Union),
    ("minusFront", BoolOp::Subtract),
    ("intersect", BoolOp::Intersect),
    ("exclude", BoolOp::Exclude),
];

/// As distâncias do Offset no corpus (unidades do documento; a caixa das formas tem 2).
pub const OFFSETS: [f64; 3] = [0.1, 0.4, -0.1];
pub const JOINS: [(&str, LineJoin); 3] = [
    ("miter", LineJoin::Miter),
    ("round", LineJoin::Round),
    ("bevel", LineJoin::Bevel),
];

fn poly(pts: &[[f64; 2]]) -> VecPath {
    VecPath {
        verts: pts.iter().map(|&p| VecVertex::corner(p)).collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// O corpus das booleanas: `(nome, fundo, frente)`.
pub fn boolean_corpus() -> Vec<(String, VecPath, VecPath)> {
    let ellipse = cook(ShapeKind::Ellipse, [1.0, 0.5], [3.0, 2.5], &[]);
    let mut out: Vec<(String, VecPath, VecPath)> = ALL_SHAPES
        .iter()
        .map(|&k| (k, cook(k, [0.0, 0.0], [2.0, 2.0], &[])))
        .filter(|(_, p)| p.closed)
        .map(|(k, p)| (format!("{k:?}"), p, ellipse.clone()))
        .collect();
    let sq = |x: f64, y: f64| poly(&[[x, y], [x + 1.0, y], [x + 1.0, y + 1.0], [x, y + 1.0]]);
    out.push(("aresta-partilhada".into(), sq(0.0, 0.0), sq(1.0, 0.0)));
    out.push(("toque-num-ponto".into(), sq(0.0, 0.0), sq(1.0, 1.0)));
    out.push(("coincidentes".into(), sq(0.0, 0.0), sq(0.0, 0.0)));
    out.push((
        "circulos-tangentes".into(),
        cook(ShapeKind::Ellipse, [0.0, 0.0], [2.0, 2.0], &[]),
        cook(ShapeKind::Ellipse, [2.0, 0.0], [4.0, 2.0], &[]),
    ));
    out.push((
        "laco-auto-intersectante".into(),
        poly(&[[0.0, 0.0], [2.0, 2.0], [2.0, 0.0], [0.0, 2.0]]),
        sq(0.5, 0.5),
    ));
    out
}

/// O corpus do Offset: `(nome, forma)` — o catálogo fechado.
pub fn offset_corpus() -> Vec<(String, VecPath)> {
    ALL_SHAPES
        .iter()
        .map(|&k| (format!("{k:?}"), cook(k, [0.0, 0.0], [2.0, 2.0], &[])))
        .filter(|(_, p)| p.closed)
        .collect()
}

/// O `d` SVG de um `VecPath` cozido (M/C/Z absolutos, em pixels SVG).
pub fn svg_d(p: &VecPath) -> String {
    let mut s = String::new();
    let t = |q: Point| (q.x * SVG_SCALE + SVG_OFFSET, q.y * SVG_SCALE + SVG_OFFSET);
    for el in vecpath_bez(p).elements() {
        match *el {
            kurbo::PathEl::MoveTo(a) => {
                let (x, y) = t(a);
                let _ = write!(s, "M{x:.6} {y:.6} ");
            }
            kurbo::PathEl::LineTo(a) => {
                let (x, y) = t(a);
                let _ = write!(s, "L{x:.6} {y:.6} ");
            }
            kurbo::PathEl::CurveTo(a, b, c) => {
                let ((x1, y1), (x2, y2), (x, y)) = (t(a), t(b), t(c));
                let _ = write!(s, "C{x1:.6} {y1:.6} {x2:.6} {y2:.6} {x:.6} {y:.6} ");
            }
            kurbo::PathEl::QuadTo(a, b) => {
                let ((x1, y1), (x, y)) = (t(a), t(b));
                let _ = write!(s, "Q{x1:.6} {y1:.6} {x:.6} {y:.6} ");
            }
            kurbo::PathEl::ClosePath => s.push_str("Z "),
        }
    }
    s
}

pub fn rule_attr(p: &VecPath) -> &'static str {
    match p.fill_rule {
        FillRule::EvenOdd => "evenodd",
        FillRule::NonZero => "nonzero",
    }
}

fn svg_doc(paths: &[&VecPath]) -> String {
    let mut s = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"800\" viewBox=\"0 0 800 800\">",
    );
    for p in paths {
        let _ = write!(
            s,
            "<path d=\"{}\" fill=\"#000\" fill-rule=\"{}\"/>",
            svg_d(p).trim_end(),
            rule_attr(p)
        );
    }
    s.push_str("</svg>\n");
    s
}

/// Escreve o corpus em `$VC_CORPUS` (por omissão `target/vectorcraft/corpus`) — o passo 1 do
/// script do oráculo.
#[test]
#[ignore = "sonda: escreve o corpus para o vectorcraft-cli"]
fn writes_the_corpus_for_the_oracle() {
    let dir = std::env::var("VC_CORPUS").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../target/vectorcraft/corpus"
        )
        .to_owned()
    });
    std::fs::create_dir_all(format!("{dir}/bool")).expect("dir");
    std::fs::create_dir_all(format!("{dir}/offset")).expect("dir");
    for (name, a, b) in boolean_corpus() {
        std::fs::write(format!("{dir}/bool/{name}.svg"), svg_doc(&[&a, &b])).expect("svg");
    }
    for (name, p) in offset_corpus() {
        std::fs::write(format!("{dir}/offset/{name}.svg"), svg_doc(&[&p])).expect("svg");
    }
    eprintln!("corpus em {dir}");
}

/// Um `d` do exportador do VectorCraft (só `M L C Q Z` absolutos) → polilinhas no DOCUMENTO.
fn parse_vc_d(d: &str) -> BezPath {
    let mut bp = BezPath::new();
    let toks: Vec<&str> = d.split_whitespace().collect();
    let mut i = 0;
    let num = |s: &str| -> f64 { s.parse::<f64>().expect("número no d do VectorCraft") };
    let pt = |x: &str, y: &str| {
        Point::new(
            (num(x) - SVG_OFFSET) / SVG_SCALE,
            (num(y) - SVG_OFFSET) / SVG_SCALE,
        )
    };
    while i < toks.len() {
        let t = toks[i];
        let (cmd, first) = t.split_at(1);
        let mut args: Vec<&str> = Vec::new();
        if !first.is_empty() {
            args.push(first);
        }
        let need = match cmd {
            "M" | "L" => 2,
            "C" => 6,
            "Q" => 4,
            "Z" => 0,
            other => panic!("comando SVG `{other}` fora do que o exportador dele escreve"),
        };
        while args.len() < need {
            i += 1;
            args.push(toks[i]);
        }
        match cmd {
            "M" => bp.move_to(pt(args[0], args[1])),
            "L" => bp.line_to(pt(args[0], args[1])),
            "C" => bp.curve_to(
                pt(args[0], args[1]),
                pt(args[2], args[3]),
                pt(args[4], args[5]),
            ),
            "Q" => bp.quad_to(pt(args[0], args[1]), pt(args[2], args[3])),
            _ => bp.close_path(),
        }
        i += 1;
    }
    bp
}

/// Uma linha da fixture.
pub struct Row {
    pub kind: String,
    pub case: String,
    pub op: String,
    pub param: String,
    pub rc: i32,
    /// Os `d` do resultado dele, separados por `|`, com a regra (`nonzero:`/`evenodd:`).
    pub ds: Vec<(FillRule, String)>,
}

pub fn fixture() -> Vec<Row> {
    include_str!("fixtures/vectorcraft_oracle.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let ds = f
                .get(5)
                .filter(|s| !s.is_empty())
                .map(|s| {
                    s.split('|')
                        .map(|x| {
                            let (r, d) = x.split_once(':').expect("regra:d");
                            let rule = if r == "evenodd" {
                                FillRule::EvenOdd
                            } else {
                                FillRule::NonZero
                            };
                            (rule, d.to_owned())
                        })
                        .collect()
                })
                .unwrap_or_default();
            Row {
                kind: f[0].to_owned(),
                case: f[1].to_owned(),
                op: f[2].to_owned(),
                param: f[3].to_owned(),
                rc: f[4].parse().expect("rc"),
                ds,
            }
        })
        .collect()
}

/// `xor / referência` (%) entre o resultado dele e o nosso, na grelha que cobre os dois e `refs`.
fn compare(theirs: &[(FillRule, String)], ours: &[VecPath], refs: &[&VecPath]) -> (f64, f64, f64) {
    let their_polys: Vec<(FillRule, Vec<Vec<Point>>)> = theirs
        .iter()
        .map(|(r, d)| (*r, bez_polylines(&parse_vc_d(d), 1e-4)))
        .collect();
    let mut all: Vec<Vec<Point>> = their_polys.iter().flat_map(|(_, p)| p.clone()).collect();
    for p in ours.iter().chain(refs.iter().copied()) {
        all.extend(bez_polylines(&vecpath_bez(p), 1e-4));
    }
    let Some((lo, hi)) = bbox(&all) else {
        return (0.0, 0.0, 0.0);
    };
    let g = Grid::covering(lo, hi, GRID_N);
    let mut tm = vec![false; g.len()];
    for (r, p) in &their_polys {
        let m = rasterize(p, *r, &g);
        for (a, b) in tm.iter_mut().zip(m) {
            *a |= b;
        }
    }
    let om = rasterize_paths(ours, &g, g.px * 0.05);
    let rm = rasterize_paths(
        &refs.iter().map(|p| (*p).clone()).collect::<Vec<_>>(),
        &g,
        g.px * 0.05,
    );
    let den = count(&rm).max(1) as f64;
    (
        100.0 * xor_count(&tm, &om) as f64 / den,
        count(&tm) as f64 * g.px_area(),
        count(&om) as f64 * g.px_area(),
    )
}

/// Um passo do corpus: `(kind, caso, op, param)`.
type Step = (String, String, String, String);

/// Cada passo do corpus: `(kind, caso, op, param) → (diff %, área dele, área nossa)`.
pub fn measure_all() -> BTreeMap<Step, (f64, f64, f64)> {
    let bools: BTreeMap<String, (VecPath, VecPath)> = boolean_corpus()
        .into_iter()
        .map(|(n, a, b)| (n, (a, b)))
        .collect();
    let offs: BTreeMap<String, VecPath> = offset_corpus().into_iter().collect();
    let mut out = BTreeMap::new();
    for row in fixture() {
        let key = (
            row.kind.clone(),
            row.case.clone(),
            row.op.clone(),
            row.param.clone(),
        );
        let v = if row.kind == "bool" {
            let (a, b) = &bools[&row.case];
            let op = OPS.iter().find(|(n, _)| *n == row.op).expect("op").1;
            compare(&row.ds, &apply(a, b, op), &[a, b])
        } else {
            let p = &offs[&row.case];
            let j = JOINS.iter().find(|(n, _)| *n == row.op).expect("junção").1;
            let d: f64 = row.param.parse().expect("d");
            compare(&row.ds, &offset_path(p, d, j, OffsetSide::Both), &[p])
        };
        out.insert(key, v);
    }
    out
}

/// Os passos em que os dois DISCORDAM (`xor > 0,5 %` da área da fonte), com quem está certo —
/// medido 11/10 pelo juiz `resvg` (`ferramentas/vectorcraft_2026-10-11/juiz_resvg_offset.py`: cada
/// lado contra `fill ∪/∖ stroke(2d)`, em % da verdade) e, nas booleanas, pela máscara do `resvg`.
/// 582 dos 592 passos concordam.
const DISAGREE: &[(&str, &str, &str, &str, &str)] = &[
    (
        "offset",
        "ArrowBent",
        "bevel",
        "0.4",
        "os dois perto: VC 0,125 · PH2D 0,031 % da verdade",
    ),
    (
        "offset",
        "ArrowDouble",
        "bevel",
        "0.1",
        "VC ERRA: 0,632 % · PH2D 0,000 %",
    ),
    (
        "offset",
        "ArrowDouble",
        "bevel",
        "0.4",
        "os dois perto: VC 0,237 · PH2D 0,138 %",
    ),
    (
        "offset",
        "ArrowRight",
        "bevel",
        "0.1",
        "VC ERRA: 0,237 % · PH2D 0,000 %",
    ),
    (
        "offset",
        "ArrowRight",
        "bevel",
        "0.4",
        "os dois perto: VC 0,099 · PH2D 0,069 %",
    ),
    (
        "offset",
        "Moon",
        "bevel",
        "0.4",
        "PH2D CERTO: é a união das normais (W2); VC e resvg cancelam a dobra da ponta (3,01 %)",
    ),
    ("offset", "Moon", "miter", "0.4", "idem"),
    (
        "offset",
        "SpeechRect",
        "bevel",
        "0.4",
        "PH2D CERTO: a cúspide da cauda — 334 dos 363 px a mais são da união das normais, o \
         resto na linha da normal; VC (0,403) e resvg cancelam a dobra, como na Lua (BUGS #39)",
    ),
    (
        "offset",
        "SpeechRect",
        "miter",
        "0.4",
        "idem (VC 0,000 · PH2D 0,492 %)",
    ),
    (
        "offset",
        "Star",
        "round",
        "0.4",
        "os dois perto: VC 0,132 · PH2D 0,159 % da verdade (este gate divide pela FONTE)",
    ),
];

#[test]
fn the_engine_agrees_with_the_vectorcraft_oracle() {
    let rows = fixture();
    // A fixture cobre o corpus INTEIRO: cada caso × cada op, cada forma × junção × d.
    let want =
        boolean_corpus().len() * OPS.len() + offset_corpus().len() * JOINS.len() * OFFSETS.len();
    assert_eq!(
        rows.len(),
        want,
        "a fixture do oráculo não cobre o corpus — regrave-a"
    );
    for r in &rows {
        // Resultado vazio dele = `exit 1` com mensagem; um exit 0 sem caminho seria outra coisa.
        assert_eq!(
            r.rc != 0,
            r.ds.is_empty(),
            "{} {} {}: rc {} com {} caminhos",
            r.case,
            r.op,
            r.param,
            r.rc,
            r.ds.len()
        );
    }
    let got = measure_all();
    let bad: Vec<String> = got
        .iter()
        .filter(|(_, v)| v.0 > OK_PCT)
        .map(|((k, c, o, p), v)| format!("{k} {c} {o} {p}: {:.2} %", v.0))
        .collect();
    let expected: Vec<String> = DISAGREE
        .iter()
        .map(|(k, c, o, p, _)| format!("{k} {c} {o} {p}"))
        .collect();
    let names: Vec<String> = bad
        .iter()
        .map(|s| s.split(':').next().unwrap_or_default().to_owned())
        .collect();
    let report = bad.join("\n");
    assert_eq!(
        names, expected,
        "as discordâncias com o oráculo mudaram — registe-as em DISAGREE com o lado certo:\n{report}"
    );
}

/// A tabela inteira, dos dois lados (`--ignored --nocapture`).
#[test]
#[ignore = "sonda: imprime a comparação passo a passo"]
fn prints_both_sides_step_by_step() {
    for ((k, c, o, p), (diff, theirs, ours)) in measure_all() {
        println!("{k}\t{c}\t{o}\t{p}\t{diff:.3}\t{theirs:.4}\t{ours:.4}");
    }
    // Com `OURS_DIR`, grava também o NOSSO resultado de cada Offset em SVG (para o juiz `resvg`).
    if let Ok(dir) = std::env::var("OURS_DIR") {
        std::fs::create_dir_all(&dir).expect("dir");
        for (name, p) in offset_corpus() {
            for (jn, j) in JOINS {
                for d in OFFSETS {
                    let out = offset_path(&p, d, j, OffsetSide::Both);
                    let refs: Vec<&VecPath> = out.iter().collect();
                    std::fs::write(format!("{dir}/{name}_{jn}_{d}.svg"), svg_doc(&refs))
                        .expect("svg");
                }
            }
        }
    }
}
