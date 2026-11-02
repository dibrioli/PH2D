//! Gate: o nosso Mono (sem hinting) contra a FreeType 2.15 CORRIDA
//! (`docs/PixelArt/ferramentas/oraculo_freetype_texto.py` → `tests/fixtures/oracle_freetype_mono.json`).
//! Compara-se o CONJUNTO de células acesas na régua da caneta e da linha de base (y para cima).

use std::collections::BTreeSet;

use ph2d_pixel_text::outline::{face, hinter, rasterize_glyph};
use serde_json::Value;

const FIXTURE: &str = include_str!("../fixtures/oracle_freetype_mono.json");

type Cells = BTreeSet<(i32, i32)>;

fn ft_cells(g: &Value) -> Cells {
    let (left, top) = (
        g["left"].as_i64().unwrap() as i32,
        g["top"].as_i64().unwrap() as i32,
    );
    let mut out = Cells::new();
    for (y, row) in g["rows"].as_array().unwrap().iter().enumerate() {
        for (x, c) in row.as_str().unwrap().chars().enumerate() {
            if c == '#' {
                out.insert((left + x as i32, top - y as i32));
            }
        }
    }
    out
}

/// Por caso: (rótulo, células da FreeType, células que diferem, avanços que diferem).
fn compare(hinting: bool) -> Vec<(String, usize, usize, usize)> {
    let doc: Value = serde_json::from_str(FIXTURE).unwrap();
    let mut out = Vec::new();
    for case in doc["cases"].as_array().unwrap() {
        if case["hinting"].as_bool() != Some(hinting) {
            continue;
        }
        let path = case["file"].as_str().unwrap();
        let data = std::fs::read(path)
            .unwrap_or_else(|e| panic!("{path}: {e} (a fonte do oráculo tem de estar instalada)"));
        let font = face(&data, 0).unwrap();
        let px = case["px"].as_u64().unwrap() as u32;
        let hint = hinting.then(|| hinter(&font, px, false)).flatten();
        let (mut total, mut diff, mut adv_diff) = (0, 0, 0);
        for g in case["glyphs"].as_array().unwrap() {
            let ch = char::from_u32(g["cp"].as_u64().unwrap() as u32).unwrap();
            let theirs = ft_cells(g);
            let ours = rasterize_glyph(&font, ch, px, false, hint.as_ref()).unwrap();
            let mut mine = Cells::new();
            for y in 0..ours.h {
                for x in 0..ours.w {
                    if ours.cover[(y * ours.w + x) as usize] != 0 {
                        mine.insert((ours.left + x as i32, ours.top - y as i32));
                    }
                }
            }
            total += theirs.len();
            diff += theirs.symmetric_difference(&mine).count();
            if std::env::var_os("PH2D_TEXT_ORACLE_DUMP").is_some() && theirs != mine {
                eprintln!(
                    "  {ch} {px}px só FT {:?} · só nós {:?}",
                    theirs.difference(&mine).collect::<Vec<_>>(),
                    mine.difference(&theirs).collect::<Vec<_>>()
                );
            }
            let ft_adv = (g["advance_26_6"].as_i64().unwrap() + 32) >> 6;
            adv_diff += usize::from(i64::from(ours.adv) != ft_adv);
        }
        let label = format!(
            "{} {}px «{}»",
            case["family"].as_str().unwrap(),
            px,
            case["text"].as_str().unwrap()
        );
        out.push((label, total, diff, adv_diff));
    }
    out
}

fn report(rows: &[(String, usize, usize, usize)]) -> String {
    rows.iter()
        .map(|(l, t, d, a)| format!("{l}: {t} células FT, {d} diferem, {a} avanços diferem"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Células diferentes por caso, na ordem do fixture, MEDIDAS em 02/11 (FreeType 2.15). Não é o alvo
/// (o alvo é zero): é a catraca — melhorar ou piorar obriga a reescrever esta linha. 40 → 27 com o
/// dropout simples SEM stubs (a omissão do `FT_Outline`; com stubs 40, smart sem stubs 68). Restam a
/// barra do «A» Sans 8 (a FreeType acende 3, nós 0) e as hastes do «H»/«D» Sans Mono 8 mais finas que
/// uma célula, que a FreeType põe uma coluna à direita da «esquerda» do dropout simples.
const MEASURED_DIFF: [usize; 18] = [3, 0, 1, 0, 0, 0, 3, 0, 1, 1, 1, 0, 0, 17, 0, 0, 0, 0];

#[test]
fn freetype_mono_unhinted_matches_cell_for_cell() {
    let rows = compare(false);
    eprintln!("{}", report(&rows));
    assert!(rows.iter().all(|r| r.3 == 0), "avanços:\n{}", report(&rows));
    assert_eq!(
        rows.iter().map(|r| r.2).collect::<Vec<_>>(),
        MEASURED_DIFF,
        "\n{}",
        report(&rows)
    );
}

/// Hinting pelo interpretador TrueType do `skrifa` (alvo Mono), MEDIDO em 02/11: 5 células em 18 casos.
/// O dropout que a DejaVu pede (`SCANTYPE 1`, simples sem stubs) não dispara em nenhuma delas: são
/// empates de regra 1 — nos cantos do «g»/«D» a 8 px a curva passa a < 0,01 célula do centro (no «g»
/// 0,495 contra 0,5) e a FreeType decide o empate ao contrário de nós; o «P» Serif 12 idem, a menos.
const MEASURED_HINTED_DIFF: [usize; 18] = [2, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];

#[test]
fn freetype_mono_hinted_matches_cell_for_cell() {
    let rows = compare(true);
    eprintln!("{}", report(&rows));
    assert!(rows.iter().all(|r| r.3 == 0), "avanços:\n{}", report(&rows));
    assert_eq!(
        rows.iter().map(|r| r.2).collect::<Vec<_>>(),
        MEASURED_HINTED_DIFF,
        "\n{}",
        report(&rows)
    );
}
