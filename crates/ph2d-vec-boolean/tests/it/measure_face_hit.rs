//! MEDIÇÃO: quanto custa achar a face sob o cursor? (nao e um gate — e o numero que decide
//! se o `Topology` do linesweeper vale a pena)
use ph2d_vec_boolean::Arrangement;
use ph2d_vec_scene::{VecPath, VecVertex};
use std::time::Instant;

/// Um circulo aproximado por `n` vertices — geometria REALISTA (nao um quadrado de 4).
fn circle(cx: f64, cy: f64, r: f64, n: usize) -> VecPath {
    VecPath {
        verts: (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / n as f64;
                VecVertex::corner([cx + r * a.cos(), cy + r * a.sin()])
            })
            .collect(),
        closed: true,
        ..VecPath::default()
    }
}

#[test]
fn how_expensive_is_a_face_hit() {
    for n_shapes in [2usize, 4, 6, 8] {
        let shapes: Vec<VecPath> = (0..n_shapes)
            .map(|i| circle(i as f64 * 6.0, (i % 2) as f64 * 4.0, 10.0, 64))
            .collect();

        // FRIO: cada pertinencia nova paga a booleana.
        let t = Instant::now();
        let mut arr = Arrangement::new(shapes.clone());
        let mut cold = 0;
        for i in 0..40 {
            let p = [i as f64 * 0.9, 2.0];
            if arr.face_at(p).is_some() {
                cold += 1;
            }
        }
        let cold_ms = t.elapsed().as_secs_f64() * 1e3;

        // QUENTE: o memo ja tem as classes visitadas (e o caso do arrasto real).
        let t = Instant::now();
        let mut hot = 0;
        for _ in 0..10 {
            for i in 0..40 {
                if arr.face_at([i as f64 * 0.9, 2.0]).is_some() {
                    hot += 1;
                }
            }
        }
        let per_hit_us = t.elapsed().as_secs_f64() * 1e6 / 400.0;

        println!(
            "{n_shapes} formas x 64 verts | FRIO {cold} hits em {cold_ms:.2} ms \
             | QUENTE {per_hit_us:.1} us/hit ({hot} hits)"
        );
    }
}
