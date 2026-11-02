//! **Outline, Fill/Stroke da selecção e Blur/Jumble contra o Trial** (`oracle_trial_outline.json`,
//! `oracle_trial_fill_stroke.json`, `oracle_trial_blur.json`), célula a célula.

use std::collections::BTreeMap;

use ph2d_pixel_core::filter::outline::{
    Background, CIRCLE, HORIZONTAL, Matrix, Outline, Place, SQUARE, VERTICAL,
};
use ph2d_pixel_core::filter::smudge::{Kind, Smudge};
use ph2d_pixel_core::filter::{Filter, Source};
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::{Background as Bg, ColorMode, NewSprite, Op, Patch, Sprite, apply};
use serde_json::Value;

fn fixture(text: &str) -> Value {
    serde_json::from_str(text).expect("fixtura")
}

/// As cores lidas; o xadrez da vista (cinzentos `80`/`c0`) é o transparente.
fn grid(v: &Value) -> Vec<Vec<[u8; 4]>> {
    v.as_array()
        .expect("grid")
        .iter()
        .map(|r| {
            r.as_str()
                .expect("linha")
                .split(' ')
                .map(|h| {
                    let c: [u8; 3] = std::array::from_fn(|k| {
                        u8::from_str_radix(&h[k * 2..k * 2 + 2], 16).unwrap()
                    });
                    let chess = c[0] == c[1] && c[1] == c[2] && matches!(c[0], 0x80 | 0xc0);
                    if chess {
                        [0; 4]
                    } else {
                        [c[0], c[1], c[2], 255]
                    }
                })
                .collect()
        })
        .collect()
}

fn sprite_from(rows: &[Vec<[u8; 4]>]) -> Sprite {
    let (w, h) = (rows[0].len() as u32, rows.len() as u32);
    let mut s = Sprite::new(NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: Bg::Transparent,
    })
    .expect("sprite");
    let r = s.bounds();
    let pixels = rows.iter().flatten().flatten().copied().collect();
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    apply(
        &mut s,
        Op::Patch(Patch {
            layer,
            frame,
            bbox: Some(r),
            rect: r,
            pixels,
        }),
    )
    .expect("patch");
    s
}

fn diff(name: &str, got: &[Vec<[u8; 4]>], want: &[Vec<[u8; 4]>]) {
    let bad: Vec<_> = (0..want.len())
        .flat_map(|y| (0..want[0].len()).map(move |x| (x, y)))
        .filter(|&(x, y)| got[y][x] != want[y][x])
        .collect();
    assert!(
        bad.is_empty(),
        "{name}: {} células, 1.ª {:?}",
        bad.len(),
        bad[0]
    );
}

#[test]
fn outline_inside_and_outside_with_the_four_presets() {
    let d = fixture(include_str!("../fixtures/oracle_trial_outline.json"));
    let shape = [(6, 6), (7, 6), (8, 6), (6, 7), (6, 8), (7, 8)];
    for (name, case) in d["cases"].as_object().expect("casos") {
        let parts: Vec<&str> = name.split('/').collect();
        let fill = if parts[0] == "shape_w" {
            [255; 4]
        } else {
            [0; 4]
        };
        let mut rows = vec![vec![fill; 16]; 16];
        for (x, y) in shape {
            rows[y][x] = [0, 0, 255, 255];
        }
        let src = Source::from_rgba(16, 16, rows.into_iter().flatten().collect());
        let matrix: Matrix = match parts[1] {
            "circle" => CIRCLE,
            "square" => SQUARE,
            "horizontal" => HORIZONTAL,
            _ => VERTICAL,
        };
        let read: Matrix = std::array::from_fn(|r| {
            std::array::from_fn(|k| case["matrix_read"][r][k].as_str() == Some("on"))
        });
        assert_eq!(read, matrix, "{name}: o preset do diálogo");
        let f = Outline {
            color: [0, 0, 0, 255],
            background: Background::Mask,
            matrix,
            place: if parts[2] == "inside" {
                Place::Inside
            } else {
                Place::Outside
            },
            tiled: false,
        };
        let got: Vec<Vec<_>> = (0..16)
            .map(|y| (0..16).map(|x| f.at(&src, x, y)).collect())
            .collect();
        diff(name, &got, &grid(&case["grid"]));
    }
}

#[test]
fn fill_paints_the_selection_and_stroke_its_inner_four_border() {
    let d = fixture(include_str!("../fixtures/oracle_trial_fill_stroke.json"));
    let rect = Mask::marquee(32, 16, (2, 2), (11, 9));
    let ellipse = Mask::ellipse(32, 16, (16, 2), (27, 13));
    for (name, case) in d["cases"].as_object().expect("casos") {
        let stroke = case["op"].as_str() == Some("Stroke");
        let mut got = vec![vec![[255u8; 4]; 32]; 16];
        for m in [&rect, &ellipse] {
            let m = if stroke { m.edge4() } else { m.clone() };
            for (x, y) in m.cells() {
                got[y as usize][x as usize] = [0, 0, 0, 255];
            }
        }
        diff(name, &got, &grid(&case["grid"]));
    }
}

fn cross(c: (i32, i32), size: u64) -> Vec<(i32, i32)> {
    if size == 1 {
        return vec![c];
    }
    vec![
        c,
        (c.0 - 1, c.1),
        (c.0 + 1, c.1),
        (c.0, c.1 - 1),
        (c.0, c.1 + 1),
    ]
}

fn checker() -> Vec<Vec<[u8; 4]>> {
    (0..16)
        .map(|y| {
            (0..16)
                .map(|x| {
                    if (x + y) % 2 == 0 {
                        [0, 0, 0, 255]
                    } else {
                        [255; 4]
                    }
                })
                .collect()
        })
        .collect()
}

#[test]
fn blur_is_the_floor_of_the_three_by_three_mean_dab_after_dab() {
    let d = fixture(include_str!("../fixtures/oracle_trial_blur.json"));
    for name in ["blur_1", "blur_3", "blur_1_x2"] {
        let case = &d["cases"][name];
        let s = sprite_from(&checker());
        let key = (s.layers()[0].id, s.frames()[0].id);
        let mut g = Smudge::new(&s, key, Kind::Blur, 0);
        for click in case["clicks"].as_array().expect("clicks") {
            let c = (
                click[0][0].as_i64().unwrap() as i32,
                click[0][1].as_i64().unwrap() as i32,
            );
            g.dab(&cross(c, case["size"].as_u64().unwrap()));
        }
        let got: Vec<Vec<_>> = (0..16)
            .map(|y| (0..16).map(|x| g.color_at(x, y).unwrap()).collect())
            .collect();
        diff(name, &got, &grid(&case["grid"]));
    }
}

#[test]
fn jumble_only_moves_colours_that_exist_inside_the_brush_band() {
    let s = sprite_from(&checker());
    let key = (s.layers()[0].id, s.frames()[0].id);
    let mut g = Smudge::new(&s, key, Kind::Jumble, 7);
    for _ in 0..3 {
        for x in 2..=13 {
            g.dab(&cross((x, 8), 3));
        }
    }
    let vals: BTreeMap<_, _> = g.values(&s).into_iter().collect();
    assert!(!vals.is_empty());
    for (&(x, y), &v) in &vals {
        assert!(
            (7..=9).contains(&y) && (1..=14).contains(&x),
            "({x},{y}) fora da faixa"
        );
        assert!(matches!(
            s.rgba_of_value(v),
            [0, 0, 0, 255] | [255, 255, 255, 255]
        ));
    }
}

/// Rodada 3 `B3_outline_bg_color`: `shape15` (128 com o quadrado 255 em 5..9), contorno `#FF0000`,
/// fundo = a COR `#808080`, matriz círculo. Fora: o anel sem os cantos; dentro: a borda do quadrado
/// com os cantos. (O Trial não lembra as cores ao reabrir; a nossa sessão lembra — de propósito.)
#[test]
fn outline_with_a_background_colour_matches_round3() {
    let d = fixture(include_str!("../fixtures/oracle_trial_round3.json"));
    let rows: Vec<[u8; 4]> = (0..15)
        .flat_map(|y| {
            (0..15).map(move |x| {
                if (5..=9).contains(&x) && (5..=9).contains(&y) {
                    [255; 4]
                } else {
                    [128, 128, 128, 255]
                }
            })
        })
        .collect();
    let src = Source::from_rgba(15, 15, rows);
    for (key, place) in [
        ("outside_circle", Place::Outside),
        ("inside_circle", Place::Inside),
    ] {
        let f = Outline {
            color: [255, 0, 0, 255],
            background: Background::Color([128, 128, 128, 255]),
            matrix: CIRCLE,
            place,
            tiled: false,
        };
        let got: Vec<Vec<_>> = (0..15)
            .map(|y| (0..15).map(|x| f.at(&src, x, y)).collect())
            .collect();
        let want: Vec<Vec<[u8; 4]>> = d["B3_outline_bg_color"]["data"][key]
            .as_array()
            .expect("linhas")
            .iter()
            .map(|r| {
                r.as_array()
                    .expect("linha")
                    .iter()
                    .map(|c| {
                        let a = c.get(3).and_then(Value::as_u64).unwrap_or(255) as u8;
                        std::array::from_fn(|k| {
                            if k == 3 {
                                a
                            } else {
                                c[k].as_u64().unwrap() as u8
                            }
                        })
                    })
                    .collect()
            })
            .collect();
        diff(key, &got, &want);
    }
}
