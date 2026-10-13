//! ⭐ W18 — o gate do `<text>` contra o ORÁCULO: o Chromium sobre os SVG `t01..t16`
//! (`excalidraw_oracle/sonda_w18.mjs`, fixture `saidas/w18/w18_sondas_offline.json`). Por caso: a
//! origem de cada letra (`getStartPositionOfChar`, espaço do utilizador) contra a nossa disposição, e
//! a caixa da tinta de cada cor-chave no SVG desenhado a 1× e 2× contra a nossa rasterização (o
//! mesmo classificador da sonda). Tolerâncias e porquê: `docs/MiroClone/09_texto_dos_svg.md`.

use serde_json::Value;

use super::{LaidText, installed, lay_out_texts};
use crate::raster;

const ORACLE: &str = "../../docs/MiroClone/ferramentas/excalidraw_oracle";
/// Letra: 0,05 unidade do utilizador. Medido: o pior caso dá 0,020; o próprio Chromium discorda de
/// si em 1/64 (o fim de uma letra contra o início da seguinte). Perder o kerning custa 0,71 (o `rg`
/// de «Hamburg») — uma tolerância de 1 unidade deixá-lo-ia passar.
const LETTER_TOL: f32 = 0.05;
/// Caixa da tinta: 2 píxeis (o anti-aliasing de dois rasterizadores sob o limiar de 0,35 da sonda).
const BOX_TOL: i64 = 2;
const KEYS: [(&str, [f64; 3]); 3] = [
    ("vermelho", [220.0, 0.0, 0.0]),
    ("verde", [0.0, 160.0, 0.0]),
    ("azul", [0.0, 0.0, 220.0]),
];

/// As famílias em que o Chromium desta máquina desenhou cada caso (o doc mede-as).
fn needs(case: &str) -> &'static [&'static str] {
    match case {
        "t02_familia_ausente" | "t03_sem_familia" => &["Liberation Serif"],
        "t04_genericas" => &["Liberation Sans", "Liberation Serif", "Noto Sans Mono"],
        "t05_lista_familias" => &["DejaVu Serif"],
        _ => &["DejaVu Sans"],
    }
}

fn manifest(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// A caixa (x0, y0, x1, y1) de cada cor-chave: o mesmo classificador de `sonda_w18.mjs` sobre o
/// desenho composto em branco.
fn boxes(rgba: &[u8], w: usize) -> [Option<[i64; 4]>; 3] {
    let mut out = [None; 3];
    for (i, p) in rgba.as_chunks::<4>().0.iter().enumerate() {
        let a = f64::from(p[3]) / 255.0;
        let c: [f64; 3] =
            std::array::from_fn(|j| (f64::from(p[j]) * a + 255.0 * (1.0 - a)).round());
        if c.iter().all(|v| *v > 245.0) {
            continue;
        }
        let ink = 1.0 - c.iter().copied().fold(255.0, f64::min) / 255.0;
        if ink <= 0.35 {
            continue;
        }
        let (x, y) = ((i % w) as i64, (i / w) as i64);
        for (k, (_, key)) in KEYS.iter().enumerate() {
            let d = (0..3).map(|j| (c[j] - key[j]).powi(2)).sum::<f64>().sqrt();
            if d < 140.0 {
                let b = out[k].get_or_insert([x, y, x, y]);
                *b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
            }
        }
    }
    out
}

/// Os caracteres do elemento `idx` (ele ou um descendente é o dono).
fn chars_of(t: &LaidText, idx: usize) -> Vec<[f32; 2]> {
    let inside = |mut o: usize| loop {
        if o == idx {
            return true;
        }
        match t.parents[o] {
            Some(p) => o = p,
            None => return false,
        }
    };
    (0..t.at.len())
        .filter(|&k| inside(t.owner[k]))
        .map(|k| t.at[k])
        .collect()
}

#[test]
fn the_texts_match_chromium() {
    let fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(manifest(&format!(
            "{ORACLE}/saidas/w18/w18_sondas_offline.json"
        )))
        .expect("fixture"),
    )
    .expect("JSON");
    let mut fails = Vec::new();
    let mut ran = 0;
    eprintln!("caso                    letras(máx)  caixa1x  caixa2x");
    for s in fixture["sondas"].as_array().expect("sondas") {
        let Some(case) = s["sonda"].as_str().and_then(|n| n.strip_prefix("T_")) else {
            continue;
        };
        if let Some(f) = needs(case)
            .iter()
            .find(|f| installed(f).as_deref() != Some(**f))
        {
            eprintln!("⚠️ {case}: SALTADO — a família «{f}» não está instalada nesta máquina");
            continue;
        }
        ran += 1;
        let step = &s["passos"][0];
        let svg =
            std::fs::read_to_string(manifest(&format!("{ORACLE}/entradas/w18_texto/{case}.svg")))
                .expect("entrada");
        let ours = lay_out_texts(&svg);
        let mut worst = 0.0f32;
        for g in step["geometria"].as_array().expect("geometria") {
            let id = g["id"].as_str().expect("id");
            let found = ours.iter().find_map(|t| {
                let i = t.ids.iter().position(|x| x.as_deref() == Some(id))?;
                Some(chars_of(t, i))
            });
            let Some(mine) = found else {
                fails.push(format!("{case}#{id}: elemento não disposto"));
                continue;
            };
            let letters = g["letras"].as_array().expect("letras");
            if mine.len() != letters.len() {
                fails.push(format!(
                    "{case}#{id}: {} letras, o Chromium {}",
                    mine.len(),
                    letters.len()
                ));
                continue;
            }
            for (l, m) in letters.iter().zip(&mine) {
                let de = [
                    l["de"][0].as_f64().unwrap() as f32,
                    l["de"][1].as_f64().unwrap() as f32,
                ];
                let dev = (de[0] - m[0]).abs().max((de[1] - m[1]).abs());
                worst = worst.max(dev);
                if dev > LETTER_TOL {
                    fails.push(format!("{case}#{id} letra {}: {m:?} ≠ {de:?}", l["i"]));
                }
            }
        }
        let mut box_worst = [0i64; 2];
        for (n, k) in ["1x", "2x"].iter().enumerate() {
            let px = &step["pixeis"][k];
            let dim = [
                px["dim"][0].as_u64().unwrap() as u32,
                px["dim"][1].as_u64().unwrap() as u32,
            ];
            let tree = raster::parse(svg.as_bytes()).expect("SVG");
            let rgba = raster::rasterize(&tree, dim).expect("desenho");
            let mine = boxes(&rgba, dim[0] as usize);
            for (j, (key, _)) in KEYS.iter().enumerate() {
                let b = &px["caixas"][key];
                let theirs = (b["n"].as_u64().unwrap() > 0)
                    .then(|| ["x0", "y0", "x1", "y1"].map(|c| b[c].as_i64().unwrap()));
                match (theirs, mine[j]) {
                    (None, None) => {}
                    (Some(t), Some(m)) => {
                        let dev = (0..4).map(|i| (t[i] - m[i]).abs()).max().unwrap_or(0);
                        box_worst[n] = box_worst[n].max(dev);
                        if dev > BOX_TOL {
                            fails.push(format!("{case} {k} {key}: {m:?} ≠ {t:?}"));
                        }
                    }
                    (t, m) => fails.push(format!("{case} {k} {key}: nosso {m:?}, Chromium {t:?}")),
                }
            }
        }
        eprintln!(
            "{case:<24}{worst:>10.3}{:>9}{:>9}",
            box_worst[0], box_worst[1]
        );
    }
    // Nenhum caso correu = nenhuma família do oráculo nesta máquina (o CI): SALTA, dito; com uma só
    // presente, os casos dela têm de passar.
    if ran == 0 {
        eprintln!(
            "⚠️ gate SALTADO — nenhuma família do oráculo instalada nesta máquina (ver acima)"
        );
        return;
    }
    assert!(
        fails.is_empty(),
        "{} desvios:\n{}",
        fails.len(),
        fails.join("\n")
    );
}
