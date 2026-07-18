//! **Critério de morte do §8 do plano: > 100 ms num quadro de 200 traços ⇒ a wave morre.**
//!
//! O balde é operação de CLIQUE, não de frame — mas 100 ms é o limite acima do qual o
//! clique deixa de parecer instantâneo, e o passo de interseções é O(segmentos²).
//!
//! `cargo test -p ph2d-flip-fill --test probe_arrange_perf --release -- --nocapture`

use ph2d_core::Vec2;
use ph2d_flip_fill::region_at;

/// Um quadro realista: uma grade de traços de mão cruzados.
fn frame(strokes: usize, pts_per_stroke: usize) -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    let wob = |i: usize| ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    (0..strokes)
        .map(|s| {
            let horiz = s % 2 == 0;
            let off = (s / 2) as f32 * (400.0 / (strokes.max(2) / 2) as f32);
            let pts: Vec<Vec2> = (0..pts_per_stroke)
                .map(|i| {
                    let t = i as f32 / pts_per_stroke as f32 * 420.0 - 10.0;
                    let j = wob(i + s * 31) * 4.0;
                    if horiz {
                        Vec2::new(t, off + j)
                    } else {
                        Vec2::new(off + j, t)
                    }
                })
                .collect();
            let n = pts.len();
            (pts, vec![0.5; n], false)
        })
        .collect()
}

#[test]
fn probe_arrange_cost() {
    println!("traços | pontos/traço | segmentos | ms");
    for (strokes, per) in [(20usize, 30usize), (50, 30), (100, 30), (200, 30)] {
        let lines = frame(strokes, per);
        let segs: usize = lines.iter().map(|(p, _, _)| p.len() - 1).sum();
        let t0 = std::time::Instant::now();
        let r = region_at(&lines, Vec2::new(200.0, 200.0));
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!(
            "{strokes:>6} | {per:>12} | {segs:>9} | {ms:>7.1}  (região: {})",
            if r.is_some() { "sim" } else { "não" }
        );
    }
}
