//! Gate: o texto de fonte do sistema contra o Aseprite Trial 1.3.18.6 CORRIDO
//! (`tests/fixtures/oracle_trial_text.json`, cópia da medição de `ph2d-pixel-core`): «Ag1» em DejaVu
//! Sans/Serif/Sans Mono a 8/12/16, Antialias desligado e ligado, Hinting ligado (as omissões).
//! A célula de âncora do Trial é o canto de cima-esquerda da caixa: caneta na coluna da âncora, linha
//! de base `ascent` linhas abaixo. Compara-se a lona inteira da faixa do caso.

use ph2d_pixel_text::outline::{face, face_metrics};
use ph2d_pixel_text::{FontChoice, TextStyle, rasterize_outline};
use serde_json::Value;

const FIXTURE: &str = include_str!("../fixtures/oracle_trial_text.json");
const SIZES: [(u32, i32, usize, usize); 3] = [(8, 1, 0, 13), (12, 13, 13, 32), (16, 32, 32, 60)];

/// Por caso (fonte × tamanho): (rótulo, células de tinta do Trial, células que diferem em tinta/sem
/// tinta, maior |Δalfa|, soma |Δalfa|).
fn compare(antialias: bool) -> Vec<(String, usize, usize, u8, u32)> {
    let doc: Value = serde_json::from_str(FIXTURE).unwrap();
    let (x0, x1) = if antialias { (60, 120) } else { (0, 60) };
    let mut out = Vec::new();
    for (family, info) in doc["fonts"].as_object().unwrap() {
        let path = info["file"].as_str().unwrap();
        let data = std::fs::read(path)
            .unwrap_or_else(|e| panic!("{path}: {e} (a fonte do Trial tem de estar instalada)"));
        let rows: Vec<Vec<u8>> = doc["cases"][family]["cells_r"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                r.as_str()
                    .unwrap()
                    .split(' ')
                    .map(|v| v.parse().unwrap())
                    .collect()
            })
            .collect();
        for (px, anchor_y, y0, y1) in SIZES {
            let mut style = TextStyle::new(FontChoice::System(path.into(), 0), px);
            style.antialias = antialias;
            let m = rasterize_outline(&data, 0, "Ag1", &style).unwrap();
            let ascent = face_metrics(&face(&data, 0).unwrap(), px).ascent;
            let (ox, oy) = (x0 as i32 + 2 - m.origin_x, anchor_y + ascent - m.baseline);
            let (mut ink, mut diff, mut max, mut sum) = (0, 0, 0u8, 0u32);
            for (y, row) in rows.iter().enumerate().take(y1).skip(y0) {
                for (x, &r) in row.iter().enumerate().take(x1).skip(x0) {
                    let theirs = 255 - r;
                    let (mx, my) = (x as i32 - ox, y as i32 - oy);
                    let ours = if mx >= 0 && my >= 0 && (mx as u32) < m.w && (my as u32) < m.h {
                        m.cover[(my as u32 * m.w + mx as u32) as usize]
                    } else {
                        0
                    };
                    ink += usize::from(theirs != 0);
                    diff += usize::from((theirs != 0) != (ours != 0));
                    let d = theirs.abs_diff(ours);
                    max = max.max(d);
                    sum += u32::from(d);
                }
            }
            out.push((format!("{family} {px}"), ink, diff, max, sum));
        }
    }
    out
}

fn report(rows: &[(String, usize, usize, u8, u32)]) -> String {
    rows.iter()
        .map(|(l, i, d, mx, s)| {
            format!("{l}: {i} células de tinta, {d} diferem, Δalfa máx {mx}, soma {s}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Mono (as omissões do Aseprite), MEDIDO em 02/11, na ordem Sans·Sans Mono·Serif × 8·12·16. Os 4
/// a 16 px (a última linha da barriga do «g») foram a 0 com `box_bottom_cut`. Os 2 de Sans 8 são cantos
/// do «g» que acendemos e a FreeType não — empates de centro sobre a curva, não dropout — os mesmos 2
/// do gate `freetype_oracle`.
const MEASURED_MONO: [usize; 9] = [2, 0, 0, 0, 0, 0, 0, 0, 0];

#[test]
fn trial_text_mono_hinted_matches_cell_for_cell() {
    let rows = compare(false);
    eprintln!("{}", report(&rows));
    assert_eq!(
        rows.iter().map(|r| r.2).collect::<Vec<_>>(),
        MEASURED_MONO,
        "\n{}",
        report(&rows)
    );
}

/// AA: suporte (que células têm tinta) e a maior diferença de alfa, MEDIDOS em 02/11. Antes: suporte
/// 0–11 e Δ 54–253 (a cobertura não era a da FreeType, faltava o tom e a caixa não cortava). Agora a
/// cobertura é a da FreeType em escala de cinza, passa pela LUT `TRIAL_TEXT_TONE` e a caixa corta como
/// o Trial: suporte 0 em todos, Δ ≤ 5 salvo um ponto de Sans 8 (14).
const MEASURED_AA: [(usize, u8); 9] = [
    (0, 14),
    (0, 3),
    (0, 5),
    (0, 2),
    (0, 3),
    (0, 2),
    (0, 2),
    (0, 2),
    (0, 2),
];

#[test]
fn trial_text_antialias_support_and_tone_are_measured() {
    let rows = compare(true);
    eprintln!("{}", report(&rows));
    assert_eq!(
        rows.iter().map(|r| (r.2, r.3)).collect::<Vec<_>>(),
        MEASURED_AA,
        "\n{}",
        report(&rows)
    );
}

const BOX: &str = include_str!("../fixtures/oracle_trial_box.json");

/// Por caso da rodada 3 («Ag1|» DejaVu Sans, âncora (10,10), lona 160×100): (rótulo, células que
/// diferem em tinta/sem tinta, maior |Δalfa|).
fn compare_box() -> Vec<(String, usize, u8)> {
    let doc: Value = serde_json::from_str(BOX).unwrap();
    let text = &doc["text"];
    let mut out = Vec::new();
    // (rótulo, texto, px, antialias, caso): a chave `text` («Ag1|», mono por px, depois AA) e a
    // `text_b` («A»/«Ag» sem AA).
    let mut cases: Vec<(String, &str, u32, bool, &Value)> = Vec::new();
    for (key, case) in text["cases"].as_object().unwrap() {
        let mut p = key.split(':').skip(1);
        let px: u32 = p.next().unwrap().parse().unwrap();
        let aa = p.next().unwrap() == "aa";
        let label = format!("Ag1| {px} {}", if aa { "aa" } else { "mono" });
        cases.push((label, text["text"].as_str().unwrap(), px, aa, case));
    }
    cases.sort_by_key(|c| (c.3, c.2));
    let mut b: Vec<_> = doc["text_b"]["cases"].as_object().unwrap().iter().collect();
    b.sort_by_key(|(k, _)| (k.len(), *k));
    for (key, case) in b {
        let (t, px) = key.split_once(':').unwrap();
        cases.push((
            format!("{t} {px} mono"),
            t,
            px.parse().unwrap(),
            false,
            case,
        ));
    }
    let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    for (label, txt, px, antialias, case) in cases {
        let anchor = case["anchor_cell"].as_array().unwrap();
        let (ax, ay) = (
            anchor[0].as_i64().unwrap() as i32,
            anchor[1].as_i64().unwrap() as i32,
        );
        let mut style = TextStyle::new(FontChoice::System(path.into(), 0), px);
        style.antialias = antialias;
        let m = rasterize_outline(&data, 0, txt, &style).unwrap();
        let ascent = face_metrics(&face(&data, 0).unwrap(), px).ascent;
        let (ox, oy) = (ax - m.origin_x, ay + ascent - m.baseline);
        let (mut diff, mut max) = (0, 0u8);
        for (y, row) in case["rows_hexR"].as_array().unwrap().iter().enumerate() {
            let row = row.as_str().unwrap();
            for x in 0..row.len() / 2 {
                let theirs = 255 - u8::from_str_radix(&row[2 * x..2 * x + 2], 16).unwrap();
                let (mx, my) = (x as i32 - ox, y as i32 - oy);
                let ours = if mx >= 0 && my >= 0 && (mx as u32) < m.w && (my as u32) < m.h {
                    m.cover[(my as u32 * m.w + mx as u32) as usize]
                } else {
                    0
                };
                diff += usize::from((theirs != 0) != (ours != 0));
                max = max.max(theirs.abs_diff(ours));
            }
        }
        out.push((label, diff, max));
    }
    out
}

/// A caixa do Aseprite (`outline::box_bottom_cut`) contra a rodada 3, MEDIDA em 02/11: «Ag1|» mono
/// 9·10·11·13·14·15·17·18·20·24·32, AA 12·16·24·32, depois «A» 14·15·16·18 e «Ag» 14·16 (`text_b`):
/// (células que diferem, maior |Δalfa|). Sem o corte 14..20 e 32 tinham a última linha da descida a
/// mais; a regra só «última linha de tinta» cortava o pé do «A» a 14..18 (o Trial deixa-o inteiro).
/// O 1–2 restantes a 14, 15, 24 e 32 mono não são da caixa: a FreeType mono casa o Trial nesses casos
/// fora do corte — são células nossas contra a FreeType (como os empates do «g» Sans 8).
const MEASURED_BOX: [(usize, u8); 21] = [
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (1, 255),
    (1, 255),
    (0, 0),
    (0, 0),
    (0, 0),
    (2, 255),
    (2, 255),
    (0, 3),
    (0, 5),
    (0, 4),
    (0, 7),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (1, 255),
    (0, 0),
];

#[test]
fn trial_text_box_cuts_like_the_trial() {
    let rows = compare_box();
    let rep = rows
        .iter()
        .map(|(l, d, mx)| format!("{l}: {d} diferem, Δalfa máx {mx}"))
        .collect::<Vec<_>>()
        .join("\n");
    eprintln!("{rep}");
    assert_eq!(
        rows.iter().map(|r| (r.1, r.2)).collect::<Vec<_>>(),
        MEASURED_BOX,
        "\n{rep}"
    );
}
