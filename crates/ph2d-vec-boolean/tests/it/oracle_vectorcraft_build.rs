//! **O VectorCraft como oráculo do Shape Builder** (prompt 49, W4): as FACES do nosso
//! [`Arrangement`] contra as `regions` do `shapeBuilder.regions` dele, sobre o corpus da W0 e os
//! casos do Build (a cena do BUGS #28, uma linha ABERTA a cortar um círculo, três círculos em fila).
//!
//! As nossas faces saem como o produto as dá: o que o cursor encontra (`face_at`) numa grelha sobre
//! a cena — uma face que o artista não consegue apontar não é face para ele. As dele saem do
//! `oraculo_vc_build.py`, gravadas em `fixtures/vectorcraft_build_regions.tsv`.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use ph2d_vec_boolean::{Arrangement, FaceId, area};
use ph2d_vec_scene::{ShapeKind, VecPath, VecVertex, Xform, bake_xform, cook};

use super::arrangement_product_shapes::enios_scene;
use super::oracle_vectorcraft::{boolean_corpus, rule_attr, svg_d};

/// Uma face: `(pertinência, área)`.
type Face = (u32, f64);

const FIXTURE: &str = include_str!("fixtures/vectorcraft_build_regions.tsv");
/// Lado da grelha que procura as faces (pontos por eixo).
const GRID: usize = 160;
/// Abaixo disto (fração da área coberta) uma face é lasca: nem a grelha a garante, nem conta.
const SLIVER: f64 = 2e-3;
/// Duas faces com a mesma pertinência concordam se as áreas diferem menos que isto (relativo).
const AREA_TOL: f64 = 0.01;

/// Os casos do Build: `(nome, formas na ordem de z)`. Uma forma ABERTA é uma linha de corte.
pub fn build_corpus() -> Vec<(String, Vec<VecPath>)> {
    // A cena do #28 está em unidades de ecrã (±200); o corpus vive na caixa de 2 do W0.
    let enio = enios_scene()
        .into_iter()
        .map(|mut p| {
            bake_xform(&mut p, &Xform([0.01, 0.0, 0.0, 0.01, 1.0, 1.0]));
            p
        })
        .collect();
    let circle = |x: f64| cook(ShapeKind::Ellipse, [x, 0.0], [x + 2.0, 2.0], &[]);
    let line = VecPath {
        verts: vec![
            VecVertex::corner([-0.5, 0.3]),
            VecVertex::corner([2.5, 1.7]),
        ],
        closed: false,
        ..VecPath::default()
    };
    vec![
        ("enio-28".into(), enio),
        ("linha-aberta".into(), vec![circle(0.0), line]),
        (
            "tres-em-fila".into(),
            vec![circle(0.0), circle(1.2), circle(2.4)],
        ),
    ]
}

/// Todos os casos: os pares do W0 (`bool/`) e os do Build (`build/`).
fn all_cases() -> Vec<(String, Vec<VecPath>)> {
    let mut out: Vec<(String, Vec<VecPath>)> = boolean_corpus()
        .into_iter()
        .map(|(n, a, b)| (format!("bool/{n}"), vec![a, b]))
        .collect();
    out.extend(
        build_corpus()
            .into_iter()
            .map(|(n, s)| (format!("build/{n}"), s)),
    );
    out
}

fn svg_doc(paths: &[VecPath]) -> String {
    let mut s = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"800\" viewBox=\"0 0 800 800\">",
    );
    for p in paths {
        let paint = if p.closed {
            format!("fill=\"#000\" fill-rule=\"{}\"", rule_attr(p))
        } else {
            "fill=\"none\" stroke=\"#000\"".to_owned()
        };
        let _ = write!(s, "<path d=\"{}\" {paint}/>", svg_d(p).trim_end());
    }
    s.push_str("</svg>\n");
    s
}

/// Escreve os casos do Build em `target/vectorcraft/corpus/build` (os `bool/` são do W0).
#[test]
#[ignore = "sonda: escreve o corpus do Build para o vectorcraft-cli"]
fn writes_the_build_corpus_for_the_oracle() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/vectorcraft/corpus/build"
    );
    std::fs::create_dir_all(dir).expect("dir");
    for (name, shapes) in build_corpus() {
        std::fs::write(format!("{dir}/{name}.svg"), svg_doc(&shapes)).expect("svg");
    }
}

/// As faces que o cursor encontra: `(pertinência, área)`, por ordem.
fn our_faces(shapes: &[VecPath]) -> Vec<Face> {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in shapes
        .iter()
        .filter(|p| p.closed)
        .flat_map(|p| p.verts_all())
    {
        for q in [v.anchor, v.in_handle, v.out_handle] {
            lo = [lo[0].min(q[0]), lo[1].min(q[1])];
            hi = [hi[0].max(q[0]), hi[1].max(q[1])];
        }
    }
    let mut arr = Arrangement::new(shapes.to_vec());
    let mut seen: BTreeSet<FaceId> = BTreeSet::new();
    for i in 0..GRID {
        for j in 0..GRID {
            // Frações irracionais: uma grelha racional cai em cima das arestas dos quadrados.
            let fx = (i as f64 + 0.5 + 0.1 * std::f64::consts::SQRT_2) / GRID as f64;
            let fy = (j as f64 + 0.5 + 0.1 * std::f64::consts::PI) / GRID as f64;
            let p = [lo[0] + fx * (hi[0] - lo[0]), lo[1] + fy * (hi[1] - lo[1])];
            if let Some(f) = arr.face_at(p) {
                seen.insert(f);
            }
        }
    }
    seen.into_iter()
        .filter_map(|f| Some((f.membership, area(arr.face_path(f)?))))
        .collect()
}

/// `caso → [(pertinência, área)]` da fixture do VectorCraft.
fn vc_faces() -> Vec<(String, Vec<Face>)> {
    FIXTURE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let (case, rest) = l.split_once('\t').expect("caso\tfaces");
            let faces = rest
                .split(',')
                .filter(|f| !f.is_empty())
                .map(|f| {
                    let (m, a) = f.split_once(':').expect("pertinência:área");
                    (m.parse().expect("m"), a.parse().expect("área"))
                })
                .collect();
            (case.to_owned(), faces)
        })
        .collect()
}

/// Casa as faces de pertinência igual e área a `AREA_TOL`; devolve as que sobram de cada lado.
fn unmatched(ours: &[Face], theirs: &[Face]) -> (Vec<Face>, Vec<Face>) {
    let total: f64 = theirs.iter().map(|f| f.1).sum::<f64>().max(1e-12);
    let big = |f: &&Face| f.1 > SLIVER * total;
    let mut left: Vec<Face> = ours.iter().filter(big).copied().collect();
    let mut right = Vec::new();
    for t in theirs.iter().filter(big) {
        match left
            .iter()
            .position(|o| o.0 == t.0 && (o.1 - t.1).abs() <= AREA_TOL * t.1)
        {
            Some(i) => {
                left.remove(i);
            }
            None => right.push(*t),
        }
    }
    (left, right)
}

/// Onde discordamos, e de que lado está a razão. O gate falha nos DOIS sentidos: uma discordância
/// nova reprova, e uma desta tabela que deixou de discordar também (a tabela envelheceu).
const DISAGREE: [(&str, &str); 1] = [(
    "bool/IsoCube",
    "PH2D certo: as arestas internas do cubo são contornos ABERTOS, que o renderer e o hit-test \
     não preenchem (BUGS, lição 6); no SVG do corpus o preenchimento fecha-os implicitamente e o VC \
     vê um buraco triangular de 0,4625 dentro do cubo — a face «só elipse» que ele tem a mais",
)];

/// ⭐ A medição da W4 como gate: as faces que o nosso Shape Builder deixa apontar são as do mapa
/// planar do VectorCraft — 51 de 52 casos (11/10), e o 52.º está na [`DISAGREE`].
#[test]
fn the_build_faces_agree_with_the_vectorcraft_oracle() {
    let cases = all_cases();
    let theirs = vc_faces();
    assert_eq!(
        theirs.len(),
        cases.len(),
        "controlo: a fixture tem um caso por caso do corpus"
    );
    let mut bad = Vec::new();
    for (name, shapes) in &cases {
        let (_, t) = theirs
            .iter()
            .find(|(c, _)| c == name)
            .expect("caso na fixture");
        let (o, v) = unmatched(&our_faces(shapes), t);
        let differs = !(o.is_empty() && v.is_empty());
        if differs != DISAGREE.iter().any(|(c, _)| c == name) {
            bad.push(format!("{name}: só PH2D {o:?} · só VC {v:?}"));
        }
    }
    assert!(
        bad.is_empty(),
        "a tabela DISAGREE não descreve a medição:\n{}",
        bad.join("\n")
    );
}
