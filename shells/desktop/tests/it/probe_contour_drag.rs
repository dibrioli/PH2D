//! Sonda do ARRASTO: `d` varrido em passos finos, como o slider faz. Mede quantas vezes a
//! CONTAGEM de anéis desenhados troca — que é o que o artista vê como "piscar".
use std::time::Instant;

fn hex() -> ph2d_vec_scene::VecPath {
    ph2d_vec_scene::cook(
        ph2d_vec_scene::ShapeKind::Polygon,
        [1.0, -1.2],
        [3.4, 1.2],
        &[6.0],
    )
}

/// Sem a guarda: conta os anéis que o `offset_path` devolve cru.
#[test]
#[ignore = "sonda"]
fn probe_drag_raw() {
    let h = hex();
    let (mut prev, mut flips, mut worst) = (usize::MAX, 0, 0.0_f64);
    for i in 0..=240 {
        let d = f64::from(i) * 0.005;
        let spec = ph2d_ecs::VecContour {
            steps: 6,
            d,
            accel: 1.0,
            ..ph2d_ecs::VecContour::default()
        };
        let t = Instant::now();
        let alive = (1..=6u16)
            .filter(|&k| {
                !ph2d_vec_boolean::offset_path(
                    &h,
                    spec.ring_distance(k),
                    ph2d_vec_scene::LineJoin::Round,
                    ph2d_vec_scene::OffsetSide::Outer,
                )
                .is_empty()
            })
            .count();
        worst = worst.max(t.elapsed().as_secs_f64() * 1000.0);
        if alive != prev {
            if prev != usize::MAX {
                flips += 1;
            }
            prev = alive;
        }
    }
    println!("CRU: {flips} trocas de contagem no arrasto | pior frame {worst:.2} ms");
}
