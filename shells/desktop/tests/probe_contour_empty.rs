//! Conta VAZIOS diretamente (no 0.3.0 todo vazio era um pânico apanhado) — medição honesta.
#[test]
#[ignore = "sonda"]
fn probe_empty() {
    for (name, sh) in [
        (
            "hex",
            ph2d_vec_scene::cook(
                ph2d_vec_scene::ShapeKind::Polygon,
                [1.0, -1.2],
                [3.4, 1.2],
                &[6.0],
            ),
        ),
        (
            "star",
            ph2d_vec_scene::cook(
                ph2d_vec_scene::ShapeKind::Star,
                [-3.6, -1.4],
                [-0.8, 1.4],
                &[5.0, 0.45, 0.0],
            ),
        ),
    ] {
        for (jn, join) in [
            ("Miter", ph2d_vec_scene::LineJoin::Miter),
            ("Round", ph2d_vec_scene::LineJoin::Round),
            ("Bevel", ph2d_vec_scene::LineJoin::Bevel),
        ] {
            let empty = (1..=400)
                .filter(|&i| {
                    let d = f64::from(i) * 0.01;
                    ph2d_vec_boolean::offset_path(&sh, d, join, ph2d_vec_scene::OffsetSide::Outer)
                        .is_empty()
                })
                .count();
            println!("{name:5} {jn:5}: {empty:3}/400 vazios");
        }
    }
}
