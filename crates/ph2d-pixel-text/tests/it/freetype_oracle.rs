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

const GRAY: &str = include_str!("../fixtures/oracle_freetype_gray.json");

/// Por caso: (rótulo, células com cobertura da FreeType, células que diferem em suporte, maior |Δ|,
/// soma |Δ|). Compara-se a cobertura bruta (0..255) na régua da caneta e da linha de base.
fn compare_gray() -> Vec<(String, usize, usize, u8, u32)> {
    let doc: Value = serde_json::from_str(GRAY).unwrap();
    let mut out = Vec::new();
    for case in doc["cases"].as_array().unwrap() {
        let path = case["file"].as_str().unwrap();
        let data = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let font = face(&data, 0).unwrap();
        let px = case["px"].as_u64().unwrap() as u32;
        let hint = hinter(&font, px, true);
        let (mut ink, mut supp, mut max, mut sum) = (0, 0, 0u8, 0u32);
        for g in case["glyphs"].as_array().unwrap() {
            let ch = char::from_u32(g["cp"].as_u64().unwrap() as u32).unwrap();
            let mut theirs = std::collections::BTreeMap::new();
            let (left, top) = (
                g["left"].as_i64().unwrap() as i32,
                g["top"].as_i64().unwrap() as i32,
            );
            for (y, row) in g["rows"].as_array().unwrap().iter().enumerate() {
                let row = row.as_str().unwrap();
                for (x, v) in row.split(' ').filter(|s| !s.is_empty()).enumerate() {
                    let v: u8 = v.parse().unwrap();
                    if v != 0 {
                        theirs.insert((left + x as i32, top - y as i32), v);
                    }
                }
            }
            let ours = rasterize_glyph(&font, ch, px, true, hint.as_ref()).unwrap();
            let mut mine = std::collections::BTreeMap::new();
            for y in 0..ours.h {
                for x in 0..ours.w {
                    let v = ours.cover[(y * ours.w + x) as usize];
                    if v != 0 {
                        mine.insert((ours.left + x as i32, ours.top - y as i32), v);
                    }
                }
            }
            let keys: BTreeSet<_> = theirs.keys().chain(mine.keys()).copied().collect();
            for k in keys {
                let (a, b) = (
                    theirs.get(&k).copied().unwrap_or(0),
                    mine.get(&k).copied().unwrap_or(0),
                );
                ink += usize::from(a != 0);
                supp += usize::from((a != 0) != (b != 0));
                max = max.max(a.abs_diff(b));
                sum += u32::from(a.abs_diff(b));
            }
        }
        let label = format!("{} {}px", case["family"].as_str().unwrap(), px);
        out.push((label, ink, supp, max, sum));
    }
    out
}

/// A nossa cobertura AA hintada contra a FreeType em escala de cinza (`oracle_freetype_gray.json`),
/// MEDIDA em 02/11, na ordem Sans·Serif·Sans Mono × 8·12·16: (suporte que difere, maior |Δ|). Antes
/// (16 passos fixos por cónica, round(área·255)): suporte 0–4, Δ 11–16. Com a subdivisão e o ⌊área·256⌋
/// da FreeType: suporte 0, Δ ≤ 4 — o resto é aritmética f32 contra a inteira dela.
const MEASURED_GRAY: [(usize, u8); 9] = [
    (0, 1),
    (0, 4),
    (0, 4),
    (0, 1),
    (0, 2),
    (0, 2),
    (0, 1),
    (0, 3),
    (0, 3),
];

#[test]
fn freetype_gray_hinted_matches_cell_for_cell() {
    let rows = compare_gray();
    let rep = rows
        .iter()
        .map(|(l, i, s, mx, sm)| format!("{l}: {i} células FT, {s} suporte, Δ máx {mx}, soma {sm}"))
        .collect::<Vec<_>>()
        .join("\n");
    eprintln!("{rep}");
    assert_eq!(
        rows.iter().map(|r| (r.2, r.3)).collect::<Vec<_>>(),
        MEASURED_GRAY,
        "\n{rep}"
    );
}
