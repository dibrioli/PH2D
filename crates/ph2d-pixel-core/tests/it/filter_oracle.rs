//! **Os filtros contra o Aseprite Trial** (P8_plano §5, `trial_efeitos_menus.py`): Despeckle
//! (`oracle_trial_despeckle.json`) e Invert (`oracle_trial_adjust.json`), célula a célula.

use ph2d_pixel_core::filter::adjust::Invert;
use ph2d_pixel_core::filter::median::Median;
use ph2d_pixel_core::filter::{Edge, Filter, Source};
use serde_json::Value;

fn fixture(text: &str) -> Value {
    serde_json::from_str(text).expect("fixtura")
}

fn gray_rows(v: &Value) -> Vec<Vec<u8>> {
    v.as_array()
        .expect("linhas")
        .iter()
        .map(|r| {
            r.as_str()
                .expect("linha")
                .split_whitespace()
                .map(|n| n.parse().expect("u8"))
                .collect()
        })
        .collect()
}

fn hex_rows(v: &Value) -> Vec<Vec<[u8; 3]>> {
    v.as_array()
        .expect("linhas")
        .iter()
        .map(|r| {
            let s = r.as_str().expect("linha");
            (0..s.len() / 6)
                .map(|i| {
                    std::array::from_fn(|k| {
                        u8::from_str_radix(&s[i * 6 + k * 2..i * 6 + k * 2 + 2], 16).expect("hex")
                    })
                })
                .collect()
        })
        .collect()
}

fn run(f: &dyn Filter, src: &Source) -> Vec<Vec<[u8; 4]>> {
    (0..src.height() as i32)
        .map(|y| (0..src.width() as i32).map(|x| f.at(src, x, y)).collect())
        .collect()
}

#[test]
fn despeckle_is_the_median_per_channel_with_the_trials_edges() {
    let d = fixture(include_str!("../fixtures/oracle_trial_despeckle.json"));
    let input = gray_rows(&d["input"]);
    let (w, h) = (input[0].len() as u32, input.len() as u32);
    let src = Source::from_rgba(
        w,
        h,
        input.iter().flatten().map(|&g| [g, g, g, 255]).collect(),
    );
    for (name, case) in d["cases"].as_object().expect("casos") {
        let f = Median {
            w: case["w"].as_u64().expect("w") as u32,
            h: case["h"].as_u64().expect("h") as u32,
            edge: if case["tiled"].as_bool().expect("tiled") {
                Edge::Wrap
            } else {
                Edge::Clamp
            },
        };
        let want = gray_rows(&case["out"]);
        let got = run(&f, &src);
        let mut bad = 0;
        for y in 0..h as usize {
            for x in 0..w as usize {
                if got[y][x] != [want[y][x], want[y][x], want[y][x], 255] {
                    bad += 1;
                }
            }
        }
        assert_eq!(bad, 0, "{name}: {bad} células diferem do Trial");
    }
}

#[test]
fn invert_is_255_minus_each_colour_channel() {
    let d = fixture(include_str!("../fixtures/oracle_trial_adjust.json"));
    let input = hex_rows(&d["input"]["in"]);
    let (w, h) = (input[0].len() as u32, input.len() as u32);
    let src = Source::from_rgba(
        w,
        h,
        input
            .iter()
            .flatten()
            .map(|c| [c[0], c[1], c[2], 255])
            .collect(),
    );
    let got = run(&Invert, &src);
    for (y, row) in input.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            assert_eq!(
                got[y][x],
                [255 - c[0], 255 - c[1], 255 - c[2], 255],
                "({x},{y})"
            );
        }
    }
}
