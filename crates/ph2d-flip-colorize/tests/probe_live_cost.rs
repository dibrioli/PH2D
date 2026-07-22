//! **A régua que decidiu o §7.2**: quanto custa UM tique do slider Trap/Bleed depois do
//! Apply, na escala do PRODUTO e com o número de quadros que a fatia C3 introduziu.
//!
//! Fica no repo porque a decisão que ela sustenta ("o Colorize não é síncrono") é a que uma
//! próxima rodada teria vontade de reverter — e porque a hipótese oposta (*"basta cachear o
//! raster, que não muda entre tiques"*) é intuitiva demais para não ser reconstruída. Ela é
//! **falsa**: o split medido é `solve` **76%** · vetorização 18% · raster+setup **4%**.
//!
//! `cargo test -p ph2d-flip-colorize --release --test probe_live_cost -- --ignored --nocapture`
use ph2d_core::Vec2;
use ph2d_flip_colorize::{Scribble, colorize_with, squeeze_from_bleed};
use std::time::Instant;

fn hand(pts: &[Vec2], seed: usize) -> Vec<Vec2> {
    let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    pts.iter()
        .enumerate()
        .map(|(i, p)| Vec2::new(p.x + h(i + seed) * 0.05, p.y + h(i + seed + 91) * 0.05))
        .collect()
}
fn seg(a: Vec2, b: Vec2, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
        })
        .collect()
}
/// A arte do smoke, com o divisor deslocado de `dx` — a pose de um quadro.
fn art(dx: f32) -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    [
        (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5), 0usize, 24usize),
        (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5), 7, 24),
        (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5), 13, 24),
        (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5), 29, 24),
        (Vec2::new(1.0 + dx, -2.5), Vec2::new(1.0 + dx, -0.6), 41, 41),
        (Vec2::new(1.0 + dx, 0.6), Vec2::new(1.0 + dx, 2.5), 53, 53),
    ]
    .into_iter()
    .map(|(a, b, s, n)| {
        let pts = hand(&seg(a, b, n), s);
        let m = pts.len();
        (pts, vec![0.13; m], false)
    })
    .collect()
}

#[test]
#[ignore = "régua: roda sob demanda, com --release"]
fn measure_the_live_slider_tick_cost() {
    let seeds = vec![
        Scribble {
            label: 0,
            points: seg(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5), 8),
            width: 0.15,
        },
        Scribble {
            label: 1,
            points: seg(Vec2::new(2.6, -1.5), Vec2::new(2.6, 1.5), 8),
            width: 0.15,
        },
    ];
    // 172,8 é a escala do PRODUTO: câmera default (10 unidades de mundo em 1080p) e
    // `DEFAULT_PRECISION` 1,6 ⇒ 1.6/(10/1080). As vizinhas são meio zoom e o dobro dele.
    eprintln!("  precisao      grade   1 quadro   3 quadros (C3)   x orcamento (16 ms)");
    for precision in [86.4f32, 172.8, 345.6] {
        let mut per_frame = Vec::new();
        for dx in [0.0f32, 0.6, 1.2] {
            let strokes = art(dx);
            let t = Instant::now();
            let r = colorize_with(&strokes, &seeds, precision, 0.0, squeeze_from_bleed(0.5));
            assert!(
                !r.is_empty(),
                "controle positivo: o corte tem de produzir região"
            );
            per_frame.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let one = per_frame[0];
        let three: f64 = per_frame.iter().sum();
        let grid = (8.0 * precision) as usize + 40;
        eprintln!(
            "  {precision:>8.1}  ~{grid:>5}px  {one:>7.1} ms   {three:>10.1} ms   \
             {:>4.0}x / {:.0}x",
            one / 16.0,
            three / 16.0
        );
    }
}
