//! W23 bloco 6 (M4) — o laço contra o oráculo `next` 0.18.0-143b5b6 (`saidas/bloco6_laco.next*.json`,
//! `corre_bloco6.mjs` sobre `entradas/bloco6_laco.json`; doc 17 §10). Módulo filho de `fidelity_tests`.

use super::*;
use ph2d_board_edit::{Editor, LASSO_COVER, Metrics};
use ph2d_board_model::ElementId;
use std::collections::{BTreeMap, BTreeSet};

/// A divergência escolhida: o nosso laço traz o grupo inteiro por um membro dentro (a porta do
/// rectângulo); o «contain» deles é tudo-ou-nada.
const GROUPS: &str = "10_grupo";

/// Os casos em que o nosso laço com `cover = 1.0` tem de dar o conjunto deles.
const SAME: [&str; 11] = [
    "1_fracao",
    "1b_limiar",
    "2_elipse",
    "3_losango",
    "4_rodado",
    "5_oito",
    "6_C_fundo",
    "6b_C_raso",
    "7_trancado",
    "8_linha_seta",
    "9_livre",
];

/// As três maneiras de armar o laço lá: o menu (todos os casos) · Ctrl+Alt e clicar a Selecção activa
/// (só o caso 1 — as três dão o mesmo).
const RUNS: [(&str, &[&str]); 3] = [
    ("", &SAME),
    (".ctrlalt", &["1_fracao"]),
    (".selecao", &["1_fracao"]),
];

fn json(rel: &str) -> Value {
    let p = format!("{DIR}/{rel}");
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")))
        .unwrap()
}

fn editor() -> Editor {
    let ink = Rgba([20, 20, 20, 255]);
    Editor::new(
        Style::new(None, Some(ink), ink),
        Metrics {
            handle: 8.0,
            rotate_offset: 24.0,
            snap: 8.0,
            drag: 3.0,
            click_size: [160.0, 100.0],
            paste_offset: 10.0,
            bind: 8.0,
            dot: 40.0,
            eraser: 10.0,
            frame_title: 20.0,
        },
    )
}

/// O nosso laço com `cover` sobre o caso (os elementos deles pelo nosso leitor) → os ids DELES.
fn ours(case: &Value, cover: f64) -> BTreeSet<String> {
    let els = case["elementos"].as_array().unwrap();
    let doc = abre(els).doc;
    assert_eq!(doc.live_len(), els.len(), "{}: o leitor deixou cair alguém", case["nome"]);
    // Os de fora ganham ids pela ordem do ficheiro (`read`); confere-se pela posição nas formas (as
    // linhas, as setas e a tinta têm a caixa dos pontos, com outra origem).
    let name = |id: ElementId| -> String {
        let i = usize::try_from(id.0 - 1).unwrap();
        let (v, el) = (&els[i], doc.get(id).unwrap());
        assert!(
            el.connector().is_some()
                || el.ink().is_some()
                || (el.x - v["x"].as_f64().unwrap()).abs() < 1e-9
                && (el.y - v["y"].as_f64().unwrap()).abs() < 1e-9,
            "{}: o id {} não é {}",
            case["nome"],
            id.0,
            v["id"]
        );
        v["id"].as_str().unwrap().to_owned()
    };
    let trail: Vec<[f64; 2]> = case["laco"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
        .collect();
    editor()
        .lasso_pick_covering(&doc, &trail, cover)
        .into_iter()
        .map(name)
        .collect()
}

fn theirs(r: &Value) -> BTreeSet<String> {
    r["selectedElementIds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

/// ⭐⭐ O nosso laço com `cover = 1.0` é o «contain» do Excalidraw (o contorno inteiro dentro, sem
/// limiar; elipse, losango e rodado pelo contorno e não pela caixa; o 8 apanha as duas voltas;
/// travados nunca; linhas, setas e tinta pelo comprimento) — nas três maneiras de o armar lá. O grupo
/// é a divergência escolhida e fica escrita aqui; e o nosso `LASSO_COVER` (o Miro) é o controlo: com
/// ele a comparação TEM de cair nos casos que medem a fracção.
#[test]
fn our_lasso_at_full_cover_is_their_contain() {
    let input = json("entradas/bloco6_laco.json");
    let cases: BTreeMap<&str, &Value> = input["casos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["nome"].as_str().unwrap(), c))
        .collect();
    let mut seen = 0;
    for (run, expect) in RUNS {
        let out = json(&format!("saidas/bloco6_laco.next{run}.json"));
        assert_eq!(out["cabecalho"]["entrada"], "bloco6_laco.json");
        assert_eq!(out["cabecalho"]["versao"], "0.18.0-143b5b6");
        let mut done = BTreeSet::new();
        let mut miro_differs = BTreeSet::new();
        for r in out["resultados"].as_array().unwrap() {
            if r["boxSelectionMode"] != "contain" {
                continue;
            }
            let name = r["caso"].as_str().unwrap();
            let case = cases[name];
            let want = theirs(r);
            let got = ours(case, 1.0);
            if name == GROUPS {
                assert!(want.is_empty(), "{run}: o contain deles mudou nos grupos: {want:?}");
                let all: BTreeSet<String> = ["gA", "gB"].map(str::to_owned).into();
                assert_eq!(got, all, "{run}: o grupo inteiro por um membro dentro");
                continue;
            }
            assert_eq!(got, want, "{run} {name}: o nosso laço (cover 1.0) ≠ o contain deles");
            if ours(case, LASSO_COVER) != want {
                miro_differs.insert(name);
            }
            done.insert(name);
            seen += 1;
        }
        assert_eq!(done, expect.iter().copied().collect(), "{run}: casos corridos");
        let fraction: BTreeSet<&str> = ["1_fracao", "1b_limiar", "6b_C_raso"].into();
        assert_eq!(
            miro_differs,
            fraction.intersection(&done).copied().collect(),
            "{run}: o controlo — a 90 % só os casos da fracção divergem"
        );
    }
    assert_eq!(seen, SAME.len() + 2);
}
