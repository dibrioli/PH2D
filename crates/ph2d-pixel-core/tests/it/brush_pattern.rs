//! **O pincel de padrão (Ctrl+B) contra o Aseprite Trial observado** — a fixtura
//! `oracle_trial_brush_pattern.json` (`docs/PixelArt/ferramentas/trial_pinceis.py`, 02/11). Cada caso
//! refaz-se pelo NOSSO núcleo — o mesmo sprite, a mesma selecção, os mesmos cliques e traços — e tem
//! de dar a grelha 64×48 inteira, célula a célula.

use std::sync::Arc;

use ph2d_pixel_core::brush::Brush;
use ph2d_pixel_core::pattern::{PatternAlign, PatternBrush};
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::{
    Background, ColorMode, NewSprite, Op, Paint, Patch, Rect, Sprite, Stroke, apply,
};
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/oracle_trial_brush_pattern.json")).expect("json")
}

const W: u32 = 64;
const H: u32 = 48;

fn rgb(letter: char) -> [u8; 4] {
    match letter {
        'R' => [255, 0, 0, 255],
        'G' => [0, 160, 0, 255],
        'B' => [0, 0, 255, 255],
        'Y' => [255, 220, 0, 255],
        'C' => [0, 200, 220, 255],
        'P' => [160, 0, 200, 255],
        '.' => [255, 255, 255, 255],
        'k' => [90, 60, 30, 255],
        't' => [0, 0, 0, 0],
        c => panic!("letra {c}"),
    }
}

fn letter(c: [u8; 4]) -> char {
    if c[3] == 0 {
        return 't';
    }
    "RGBYCP.k".chars().find(|&l| rgb(l) == c).unwrap_or('?')
}

/// O sprite de entrada pelo nome do `.ase` que o script gerou (a legenda `sprites` da fixtura).
fn entrada(nome: &str) -> Sprite {
    let branco = !matches!(nome, "pad_t.ase" | "pad_buraco.ase");
    let mut g = vec![vec![if branco { '.' } else { 't' }; W as usize]; H as usize];
    if matches!(nome, "pad_t.ase" | "pad_wb.ase" | "pad_buraco.ase") {
        for row in &mut g[14..=26] {
            for c in &mut row[24..=37] {
                *c = 'k';
            }
        }
    }
    let (px, py) = if nome == "pad_b.ase" { (13, 9) } else { (8, 8) };
    for (j, linha) in ["RGB", "YCP"].iter().enumerate() {
        for (i, l) in linha.chars().enumerate() {
            g[py + j][px + i] = l;
        }
    }
    if nome == "pad_buraco.ase" {
        g[8][9] = 't';
    }
    let mut s = Sprite::new(NewSprite {
        width: W,
        height: H,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let r = Rect::new(0, 0, W, H);
    let pixels = g.iter().flatten().flat_map(|&c| rgb(c)).collect();
    apply(
        &mut s,
        Op::Patch(Patch {
            layer: l,
            frame: f,
            bbox: Some(r),
            rect: r,
            pixels,
        }),
    )
    .expect("patch");
    s
}

fn grelha(s: &Sprite) -> Vec<String> {
    let key = (s.layers()[0].id, s.frames()[0].id);
    (0..H as i32)
        .map(|y| {
            (0..W as i32)
                .map(|x| letter(s.rgba_of_value(s.value_at(key, x, y))))
                .collect()
        })
        .collect()
}

fn xy(v: &Value) -> (i32, i32) {
    (
        v[0].as_i64().expect("x") as i32,
        v[1].as_i64().expect("y") as i32,
    )
}

/// Refaz um caso: selecção → pincel, e cada passo é um traço próprio (como no Trial).
fn refaz(c: &Value) -> Vec<String> {
    let mut s = entrada(c["sprite"].as_str().expect("sprite"));
    let key = (s.layers()[0].id, s.frames()[0].id);
    let sel = &c["selecao"];
    let (a, b) = (
        (
            sel[0].as_i64().unwrap() as i32,
            sel[1].as_i64().unwrap() as i32,
        ),
        (
            sel[2].as_i64().unwrap() as i32,
            sel[3].as_i64().unwrap() as i32,
        ),
    );
    let mask = match c["forma_selecao"].as_str() {
        Some("elipse") => Mask::ellipse(W, H, a, b),
        _ => Mask::marquee(W, H, a, b),
    };
    let pattern = PatternBrush::from_selection(&s, key, &mask).expect("selecção");
    let align = match c["modo"].as_str() {
        Some("source") => PatternAlign::Source,
        Some("destination") => PatternAlign::Destination,
        _ => PatternAlign::Paint,
    };
    let paint = Paint {
        brush: Brush {
            pattern: Some(Arc::new(pattern)),
            pattern_align: align,
            ..Brush::default()
        },
        ..Paint::default()
    };
    for passo in c["passos"].as_array().expect("passos") {
        let (de, ate) = match passo[0].as_str() {
            Some("clique") => (xy(&passo[1]), xy(&passo[1])),
            _ => (xy(&passo[1]), xy(&passo[2])),
        };
        let mut st = Stroke::begin(&s, key.0, key.1, 0, paint.clone(), de);
        st.to(&s, ate.0, ate.1);
        if let Some(op) = st.op(&s) {
            apply(&mut s, op).expect("op");
        }
    }
    grelha(&s)
}

/// Os casos que o núcleo refaz (os rb2/rb3/rb5 medem a interface; o rb4_sem_selecao não cria pincel).
const CASOS: [&str; 13] = [
    "rb1_source",
    "rb1_destination",
    "rb1_paint",
    "rb1_source_b",
    "rb1_destination_b",
    "rb1_ancora_4x2",
    "rb1_ancora_4x3",
    "rb4_elipse",
    "rb4_rect_maior",
    "rb4_buraco",
    "rb4_buraco_source",
    "rb4_elipse_branco",
    "rb4_apara_6x2",
];

#[test]
fn every_pattern_brush_case_paints_the_trials_grid_cell_by_cell() {
    let f = fx();
    let mut falhas = Vec::new();
    for nome in CASOS {
        let c = &f["casos"][nome];
        let quer: Vec<String> = c["grelha"]
            .as_array()
            .expect("grelha")
            .iter()
            .map(|r| r.as_str().expect("linha").to_owned())
            .collect();
        let tem = refaz(c);
        let erradas: Vec<(usize, usize)> = (0..H as usize)
            .flat_map(|y| (0..W as usize).map(move |x| (x, y)))
            .filter(|&(x, y)| quer[y].as_bytes()[x] != tem[y].as_bytes()[x])
            .collect();
        if !erradas.is_empty() {
            let (x, y) = erradas[0];
            falhas.push(format!(
                "{nome}: {} células, 1.ª ({x},{y}) quer {:?} tem {:?}\n quer {}\n tem  {}",
                erradas.len(),
                quer[y].as_bytes()[x] as char,
                tem[y].as_bytes()[x] as char,
                quer[y],
                tem[y]
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// A âncora é a do rectângulo da selecção, NUNCA a do recorte opaco: uma selecção 6×2 com o padrão
/// só nas 3 colunas da esquerda carimba a partir de `cursor − 3` (rb4_apara_6x2).
#[test]
fn the_anchor_is_the_selections_and_transparent_margins_are_not_trimmed() {
    let s = entrada("pad_t.ase");
    let key = (s.layers()[0].id, s.frames()[0].id);
    let p =
        PatternBrush::from_selection(&s, key, &Mask::marquee(W, H, (8, 8), (13, 9))).expect("p");
    assert_eq!(p.size(), (6, 2));
    assert_eq!(p.anchor(), (3, 1));
}
