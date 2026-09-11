//! Sonda da cena `PH2D_BUILD_SMOKE=25` + varredura de distâncias que fazem o offset entrar em
//! pânico (achado ao medir os números da cena).
use std::time::Instant;

fn hex() -> ph2d_vec_scene::VecPath {
    ph2d_vec_scene::cook(
        ph2d_vec_scene::ShapeKind::Polygon,
        [1.0, -1.2],
        [3.4, 1.2],
        &[6.0],
    )
}

#[test]
#[ignore = "sonda de medição"]
fn probe_offset_panic_sweep() {
    let h = hex();
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    for (jn, join) in [
        ("Miter", ph2d_vec_scene::LineJoin::Miter),
        ("Round", ph2d_vec_scene::LineJoin::Round),
        ("Bevel", ph2d_vec_scene::LineJoin::Bevel),
    ] {
        for (sn, shape) in [
            ("hex", hex()),
            (
                "star",
                ph2d_vec_scene::cook(
                    ph2d_vec_scene::ShapeKind::Star,
                    [-3.6, -1.4],
                    [-0.8, 1.4],
                    &[5.0, 0.45, 0.0],
                ),
            ),
            (
                "rect",
                ph2d_vec_scene::cook(
                    ph2d_vec_scene::ShapeKind::Rectangle,
                    [0.0, 0.0],
                    [2.0, 1.0],
                    &[],
                ),
            ),
        ] {
            let mut bad = 0;
            let mut first = String::new();
            for i in 1..=200 {
                let d = f64::from(i) * 0.02;
                let sh = shape.clone();
                let r = std::panic::catch_unwind(move || {
                    ph2d_vec_boolean::offset_path(&sh, d, join, ph2d_vec_scene::OffsetSide::Outer)
                        .len()
                });
                if r.is_err() {
                    bad += 1;
                    if first.is_empty() {
                        first = format!("{d:.2}");
                    }
                }
            }
            println!("{sn:5} {jn:5}: {bad:3}/200 distancias panicam (1a em d={first})");
        }
    }
    let _ = h;
    std::panic::set_hook(prev);
}

#[test]
#[ignore = "sonda de medição"]
fn probe_contour_scene() {
    let h = hex();
    for accel in [1.0_f32, 1.3, 1.6] {
        for d in [0.10_f64, 0.12, 0.14] {
            let spec = ph2d_ecs::VecContour {
                steps: 6,
                d,
                accel,
                ..ph2d_ecs::VecContour::default()
            };
            let dists: Vec<String> = (1..=6)
                .map(|k| format!("{:.2}", spec.ring_distance(k)))
                .collect();
            let t = Instant::now();
            let mut total = 0;
            for k in 1..=6u16 {
                total += ph2d_vec_boolean::offset_path(
                    &h,
                    spec.ring_distance(k),
                    ph2d_vec_scene::LineJoin::Round,
                    ph2d_vec_scene::OffsetSide::Outer,
                )
                .len();
            }
            println!(
                "accel {accel} d {d}: [{}] externo {:.2} | {:.2} ms ({total} paths)",
                dists.join(", "),
                spec.ring_distance(6),
                t.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
}
