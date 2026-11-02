//! **A ferramenta Gradient contra o Trial** (`oracle_trial_gradient.json`): 32 arrastos — linear
//! (horizontal, vertical, diagonal, curto, a começar fora) e radial, Solid e Bayer 2/4/8 — célula a
//! célula, fg `#000000` → bg `#ff0000` sobre um canvas branco (a região do balde é o canvas).

use ph2d_pixel_core::dither::Dither;
use ph2d_pixel_core::gradient::{Gradient, Shape};
use serde_json::Value;

fn cell(v: &Value) -> (i32, i32) {
    (
        v[0].as_i64().expect("x") as i32,
        v[1].as_i64().expect("y") as i32,
    )
}

#[test]
fn every_trial_drag_gives_the_same_cells() {
    let d: Value = serde_json::from_str(include_str!("../fixtures/oracle_trial_gradient.json"))
        .expect("fixtura");
    let cases = d["cases"].as_object().expect("casos");
    assert_eq!(cases.len(), 32);
    for (name, c) in cases {
        let g = Gradient {
            shape: match c["shape"].as_str() {
                Some("radial") => Shape::Radial,
                _ => Shape::Linear,
            },
            from: cell(&c["from_cell"]),
            to: cell(&c["to_cell"]),
            fg: [0, 0, 0, 255],
            bg: [255, 0, 0, 255],
            dither: match c["fill"].as_str() {
                Some("bayer2") => Dither::Bayer2,
                Some("bayer4") => Dither::Bayer4,
                Some("bayer8") => Dither::Bayer8,
                _ => Dither::None,
            },
        };
        let mut bad = Vec::new();
        for (y, row) in c["grid"].as_array().expect("grid").iter().enumerate() {
            for (x, hex) in row.as_str().expect("linha").split(' ').enumerate() {
                let want: [u8; 3] = std::array::from_fn(|k| {
                    u8::from_str_radix(&hex[k * 2..k * 2 + 2], 16).expect("hex")
                });
                let got = g.color_at((x as i32, y as i32));
                if got[..3] != want {
                    bad.push((x, y, want, got));
                }
            }
        }
        assert!(
            bad.is_empty(),
            "{name}: {} células, 1.ª {:?}",
            bad.len(),
            bad[0]
        );
    }
}
