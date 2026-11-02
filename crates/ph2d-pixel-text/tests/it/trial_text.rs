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

/// Mono (as omissões do Aseprite), MEDIDO em 02/11, na ordem Sans·Sans Mono·Serif × 8·12·16. Os 4 a
/// 16 são a última linha da barriga do «g»: o Trial corta a caixa 17 linhas abaixo da âncora (a
/// FreeType mono hintada desenha-a). Os 2 de Sans 8 são cantos do «g» que acendemos e a FreeType não
/// — empates de centro sobre a curva, não dropout — os mesmos 2 do gate `freetype_oracle`.
const MEASURED_MONO: [usize; 9] = [2, 0, 4, 0, 0, 4, 0, 0, 4];

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
/// 0–1 a 8/12 px e Δ 54–60 (a cobertura não era a da FreeType e faltava o tom). Agora a cobertura é a
/// da FreeType em escala de cinza e passa pela LUT `TRIAL_TEXT_TONE`: 8/12 px com suporte 0 e Δ ≤ 14;
/// a 16 px os 6/6/7 e o Δ ~220 são a linha de baixo que o Trial corta (a caixa).
const MEASURED_AA: [(usize, u8); 9] = [
    (0, 14),
    (0, 3),
    (6, 214),
    (0, 2),
    (0, 3),
    (6, 221),
    (0, 2),
    (0, 2),
    (7, 216),
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
