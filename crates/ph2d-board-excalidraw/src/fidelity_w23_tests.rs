//! W23 B9 — a lei das PEÇAS (losango vivo e redondo, rectângulo redondo) contra a varredura do
//! oráculo 0.18.1 (`saidas/w23_V_*.0181.json`, `gera_w23_vao_casos.py` + `corre_fidelidade.mjs
//! --prefixo w23`; o subconjunto gravado no repo está no README do oráculo §W23). Módulo filho de
//! `fidelity_tests`.

use super::*;

/// Os casos gravados: cada família × vão positivo · negativo · `gap 0` · ponto fixo dentro · rodado ·
/// medidas fraccionárias · o tecto do raio (`type 3` acima de 128) — e o ramo de um só cruzamento.
const CASES: [&str; 15] = [
    "V_losango_173p4x97p8_gneg5",
    "V_losango_200x120_rot_g6",
    "V_losango_200x120_g0",
    "V_losango_173p4x97p8_dentro",
    "V_losango_r2_173p4x97p8_g6",
    "V_losango_r2_200x120_rot_gneg5",
    "V_losango_r2_200x120_g0",
    "V_losango_r2_173p4x97p8_dentro",
    "V_losango_r3_300x200_g6",
    "V_losango_r3_300x200_gneg5",
    "V_retangulo_r3_173p4x97p8_g6",
    "V_retangulo_r3_200x120_rot_gneg5",
    "V_retangulo_r3_300x200_g1",
    "V_retangulo_r3_200x120_g0",
    "V_retangulo_r3_173p4x97p8_dentro",
];

/// O resíduo medido da lei (`mede_w23_vao.py`: 0,0022 px, a tolerância do Newton dele) com folga.
const TOL: f64 = 0.01;

/// Os pontos fixos dentro do `caso_dentro` do gerador (fracções da meia-caixa; os três últimos só no
/// rectângulo).
const INSIDE: [[f64; 2]; 11] = [
    [0.0, 0.0],
    [0.3, 0.0],
    [0.0, 0.3],
    [0.2, 0.2],
    [-0.25, 0.1],
    [0.4, -0.15],
    [0.1, -0.5],
    [-0.35, -0.25],
    [0.85, 0.75],
    [-0.8, 0.8],
    [0.9, -0.6],
];

#[derive(Default, Debug)]
struct Branches {
    round: bool,
    entry: usize,
    focus: usize,
    corner: usize,
    stay: usize,
    first: usize,
}

/// ⭐⭐ A lei das peças reproduz o 0.18.1 ao EMPURRAR a forma (+1 px, depois −1: duas re-ligações; a
/// 2.ª sem cruzamentos deixa a ponta onde a 1.ª a pôs, sem nenhuma a ponta não mexe) a ≤ [`TOL`], em
/// cada ramo: a entrada (≥ 2 cruzamentos), o ponto do foco (`gap 0`, ou 1 cruzamento: `corner`), ficar.
#[test]
fn w23_the_pieces_law_reproduces_the_0181_sweep() {
    let mut fam: std::collections::BTreeMap<&str, Branches> = Default::default();
    let mut worst: f64 = 0.0;
    for caso in CASES {
        let p = format!("{DIR}/saidas/w23_{caso}.0181.json");
        let d: Value = serde_json::from_str(
            &std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")),
        )
        .unwrap();
        let els = passo(&d, "empurrar");
        let f = &els[0];
        let num = |k: &str| f[k].as_f64().unwrap();
        let (kind, family) = match (f["type"].as_str().unwrap(), f["roundness"]["type"].as_u64()) {
            ("diamond", None) => (ShapeType::Diamond, "losango"),
            ("diamond", Some(2)) => (ShapeType::Diamond, "losango r2"),
            ("diamond", Some(3)) => (ShapeType::Diamond, "losango r3"),
            ("rectangle", Some(3)) => (ShapeType::Rectangle, "rectângulo r3"),
            other => panic!("{caso}: {other:?}"),
        };
        let round = f["roundness"]["type"].as_u64().map(|t| t as u8);
        let mut st = Style::new(None, Some(Rgba([30, 30, 30, 255])), Rgba([30, 30, 30, 255]));
        st.round = round.is_some();
        let make = |dx: f64| {
            let mut s = Element::new_shape(
                ph2d_board_model::ElementId(1),
                ph2d_board_model::FracKey::between(None, None),
                Shape {
                    kind,
                    style: st.clone(),
                    text: RichText::plain(""),
                },
                [num("x") + dx, num("y"), num("width"), num("height")],
            );
            s.angle = num("angle");
            s
        };
        let (shape, pushed) = (make(0.0), make(1.0));
        let c = shape.center();
        let br = fam.entry(family).or_default();
        br.round = round.is_some();
        for a in &els[1..] {
            let pts = world(a);
            let id = a["id"].as_str().unwrap();
            let end = a["endBinding"].is_object();
            let (got, from, b) = if end {
                (pts[pts.len() - 1], pts[pts.len() - 2], &a["endBinding"])
            } else {
                (pts[0], pts[1], &a["startBinding"])
            };
            let (focus, gap) = (b["focus"].as_f64().unwrap(), b["gap"].as_f64().unwrap());
            let start = if caso.ends_with("_dentro") {
                let i: usize = id[1..id.find('_').unwrap()].parse().unwrap();
                let q = INSIDE[i];
                [c[0] + q[0] * shape.w / 2.0, c[1] + q[1] * shape.h / 2.0]
            } else {
                c
            };
            let land = |s: &Element| crate::legacy_focus::landing_typed(s, round, from, focus, gap);
            let (law, branch) = match (land(&shape), land(&pushed)) {
                (Some(q), _) => {
                    let aim = crate::legacy_focus::landing_typed(&shape, round, from, focus, 0.0);
                    if gap != 0.0 && aim.is_some_and(|m| dist(m, q) < 1e-9) {
                        (q, &mut br.corner)
                    } else if gap == 0.0 {
                        (q, &mut br.focus)
                    } else {
                        (q, &mut br.entry)
                    }
                }
                (None, Some(q)) => (q, &mut br.first),
                (None, None) => (start, &mut br.stay),
            };
            *branch += 1;
            let e = dist(law, got);
            worst = worst.max(e);
            assert!(
                e <= TOL,
                "{caso} {id}: a lei põe {law:?}, o 0.18.1 {got:?} ({e:.4})"
            );
        }
    }
    eprintln!("{fam:#?}\npior {worst:.4}");
    for (family, b) in &fam {
        assert!(b.entry > 100 && b.stay > 0, "{family}: {b:?}");
        // O ramo de um só cruzamento (o ponto do foco com `gap ≠ 0`) só existe nos cantos redondos.
        assert_eq!(b.corner > 0, b.round, "{family}: {b:?}");
    }
    assert!(
        fam.values().any(|b| b.first > 0),
        "a 2.ª re-ligação sem cruzamentos (r3 300×200 −5)"
    );
}
