//! **Os filtros contra o Aseprite Trial** (P8_plano §5, `trial_efeitos_menus.py`): Despeckle
//! (`oracle_trial_despeckle.json`) e Invert (`oracle_trial_adjust.json`), célula a célula.

use ph2d_pixel_core::filter::adjust::{BrightnessContrast, Curve, HueMode, HueSaturation, Invert};
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

fn lut_of(f: &dyn Filter) -> Vec<u8> {
    let src = Source::from_rgba(256, 1, (0..=255u8).map(|v| [v, v, v, 255]).collect());
    (0..256).map(|x| f.at(&src, x, 0)[0]).collect()
}

fn lut_from(v: &Value) -> Vec<u8> {
    let v = match v {
        Value::String(s) => serde_json::from_str(s).expect("lut"),
        v => v.clone(),
    };
    v.as_array()
        .expect("lut")
        .iter()
        .map(|n| n.as_u64().expect("u8") as u8)
        .collect()
}

#[test]
fn brightness_contrast_gives_the_trials_tables() {
    let d = fixture(include_str!("../fixtures/oracle_trial_adjust.json"));
    // Rodada 1 (9 casos) + rodada 2 (43 casos cinzentos, 11 008 células): `⌊y·255,5⌋` fecha a 0.
    let r2 = &d["round2_brightness_contrast"]["cases_gray_lut"];
    let mut n = 0;
    for (name, lut) in d["brightness_contrast"]["cases"]
        .as_object()
        .expect("casos")
        .iter()
        .chain(r2.as_object().expect("casos r2"))
    {
        n += 1;
        let (b, c) = name.split_once(',').expect("b,c");
        let f = BrightnessContrast::new(b.parse().expect("b"), c.parse().expect("c"));
        assert_eq!(lut_of(&f), lut_from(lut), "B/C {name}");
    }
    assert_eq!(n, 9 + 43, "as duas rodadas inteiras");
}

#[test]
fn color_curve_is_straight_lines_between_the_points() {
    let d = fixture(include_str!("../fixtures/oracle_trial_adjust.json"));
    let cc = &d["color_curve"];
    assert_eq!(
        lut_of(&Curve::new(&[(126, 129)])),
        lut_from(&cc["lut_point_128_130"])
    );
    assert_eq!(
        lut_of(&Curve::new(&[(126, 192)])),
        lut_from(&cc["lut_point_128_192"])
    );
}

#[test]
fn hue_saturation_matches_the_trial_except_on_exact_half_ties() {
    let d = fixture(include_str!("../fixtures/oracle_trial_adjust.json"));
    let input = hex_rows(&d["input"]["in"]);
    let mut ties = 0;
    for (name, rows) in d["hue_saturation"]["cases"].as_object().expect("casos") {
        let (mode, v) = name.split_once(':').expect("modo");
        let v: Vec<i32> = v.split(',').map(|n| n.parse().expect("n")).collect();
        let f = HueSaturation {
            mode: match mode {
                "HSL" => HueMode::Hsl,
                "HSV" => HueMode::Hsv,
                "HSL+" => HueMode::HslPlus,
                _ => HueMode::HsvPlus,
            },
            hue: v[0],
            saturation: v[1],
            lightness: v[2],
        };
        for (y, row) in hex_rows(rows).iter().enumerate() {
            for (x, want) in row.iter().enumerate() {
                let c = input[y][x];
                let (got, raw) = (f.rgb(c), f.scaled(c));
                for k in 0..3 {
                    if got[k] == want[k] {
                        continue;
                    }
                    let tie = ((raw[k] - raw[k].floor()) - 0.5).abs() < 1e-6;
                    assert!(
                        tie && got[k].abs_diff(want[k]) == 1,
                        "{name} ({x},{y}) canal {k}: {got:?} × {want:?} (cru {:.6})",
                        raw[k]
                    );
                    ties += 1;
                }
            }
        }
    }
    assert!(
        ties > 0,
        "a fixtura tem empates: o desempate do Trial é ruído de vírgula flutuante"
    );
}
