//! Medida (ignorada): a distância, em células, das NOSSAS rotações não-90°, da inclinação e do
//! Sprite Size RotSprite fora de 2× às tabelas do Trial (`oracle_trial_selection.json`) —
//! `cargo test --profile smoke -p ph2d-pixel-core mede_rotacao -- --ignored --nocapture`.
//! O Trial não deu fórmula para estes casos; nunca se copia a tabela para «acertar».

use ph2d_pixel_core::affine::{Affine, Linear, Resample};
use ph2d_pixel_core::canvas::{self, SizeMethod};
use ph2d_pixel_core::edit;
use ph2d_pixel_core::transform::lift;

use crate::selection_oracle::{cases, fx, rgba_sprite};
use crate::transform_oracle::{lifted_box, run, sz_diff, xf_ids, xf_sprite};

#[test]
#[ignore = "medida, não lei"]
fn mede_rotacao() {
    let f = fx();
    let mut n = 0;
    println!("caso | células diferentes de 384");
    for (k, c) in cases(&f, "xform_") {
        let fields = &c["fields"];
        let angle = fields["angle"].as_f64();
        let skew = fields["skew"].as_f64();
        if angle == Some(90.0) || fields["w"].is_u64() {
            continue; // fixados em transform_oracle
        }
        let method = if c["algorithm"] == "rotsprite" {
            Resample::RotSprite
        } else {
            Resample::Fast
        };
        let want: Vec<Vec<i64>> = serde_json::from_value(c["ids"].clone()).expect("ids");
        let mut best = usize::MAX;
        for sign in [1.0, -1.0] {
            let mut s = xf_sprite(&f);
            let key = (s.layers()[0].id, s.frames()[0].id);
            let original = lifted_box(&s);
            let fl = lift(&s, key, Some(&original)).expect("lift");
            let lin = match (angle, skew) {
                (Some(a), _) => Linear::from_screen_ccw(a, 1.0, 1.0, 0.0),
                (_, Some(k)) => Linear::from_screen_ccw(0.0, 1.0, 1.0, sign * k),
                _ => unreachable!(),
            };
            let t = Affine::around_center(lin, fl.rect());
            let moved = edit::transform(&fl, &t, method, 0).expect("t");
            let op = edit::commit(&s, key, &original, &moved, 0);
            run(&mut s, op);
            let got = xf_ids(&f, &s);
            let d = got
                .iter()
                .flatten()
                .zip(want.iter().flatten())
                .filter(|(a, b)| a != b)
                .count();
            best = best.min(d);
            if angle.is_some() {
                break;
            }
        }
        println!("{k} | {best}");
        n += 1;
    }
    for (k, c) in cases(&f, "size_rotsprite_") {
        let (w, h) = (
            c["w"].as_u64().expect("w") as u32,
            c["h"].as_u64().expect("h") as u32,
        );
        if w % 24 == 0 && h % 16 == 0 {
            continue;
        }
        let mut s = rgba_sprite(f["inputs"]["sz_rows_rgba"].as_array().expect("sz"));
        let op = canvas::sprite_size(&s, w, h, SizeMethod::RotSprite);
        run(&mut s, op);
        println!("{k} | {} de {}", sz_diff(&f, &s, c), w * h);
        n += 1;
    }
    assert_eq!(n, 20, "17 rotações/inclinações + 3 Sprite Size RotSprite");
}
