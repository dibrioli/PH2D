//! **A VALÊNCIA decide a estabilidade do Sharpen?** — a sonda que separa um
//! defeito da LEI de um defeito da FIXTURE.
//!
//! O gather do `calc_sharpen_filter` é `Σ_vizinhos (p[n] − p[i])·f[n]` e **não é
//! normalizado pela contagem**. Com `f ≈ 1` isso é `valência × laplaciano`, e um
//! passo de alisamento de fator maior que um OVERSHOOTA. Numa esfera UV os polos
//! têm valência igual ao número de segmentos (64 na fixture) — a malha do
//! produto é um cubo subdividido, valência 4 quase em toda parte.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --test measure_sharpen_valence --release -- --ignored --nocapture`

use ph2d_mesh::{Mesh, shapes};
use ph2d_sculpt3d::{Brush, FilterKind, SculptStroke, Verb};

fn norm(d: [f32; 3]) -> f32 {
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn ridged(mut m: Mesh) -> Mesh {
    for p in m.positions_mut() {
        let r = norm(*p);
        if r <= f32::EPSILON {
            continue;
        }
        let t = p[1] / r;
        let bump = (-(t * t) / (2.0 * 0.15 * 0.15)).exp() * 0.12;
        let s = (r + bump) / r;
        *p = [p[0] * s, p[1] * s, p[2] * s];
    }
    m
}

fn valence_stats(m: &Mesh) -> (usize, f32) {
    let adj = m.adjacency();
    let mut max = 0usize;
    let mut sum = 0usize;
    for v in 0..m.vert_count() {
        let n = adj.vert_verts.neighbours(v).len();
        max = max.max(n);
        sum += n;
    }
    (max, sum as f32 / m.vert_count() as f32)
}

fn max_step(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let mut worst = 0.0f32;
    for v in 0..pos.len() {
        let r = norm(pos[v]);
        for &nb in adj.vert_verts.neighbours(v) {
            worst = worst.max((norm(pos[nb as usize]) - r).abs());
        }
    }
    worst
}

fn sweep(label: &str, build: impl Fn() -> Mesh) {
    let base = build();
    let (vmax, vavg) = valence_stats(&base);
    let step0 = max_step(&base);
    println!(
        "\n=== {label} — {} vértices | valência máx {vmax}, média {vavg:.2} | degrau {step0:.6} ===",
        base.vert_count()
    );
    println!("  força | excursão | degrau    | Δdegrau");
    for f in [0.25f32, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 8.0] {
        let mut m = build();
        let pre = m.positions().to_vec();
        let mut s = SculptStroke::default();
        s.filter_begin(&m);
        let b = Brush {
            verb: Verb::Smooth,
            ..Brush::default()
        };
        s.filter(&mut m, &b, FilterKind::Sharpen, f);
        let exc = pre
            .iter()
            .zip(m.positions())
            .map(|(a, c)| norm([c[0] - a[0], c[1] - a[1], c[2] - a[2]]))
            .fold(0.0f32, f32::max);
        let st = max_step(&m);
        println!("  {f:>5.2} | {exc:>8.5} | {st:>9.6} | {:>8.3}×", st / step0);
    }
}

#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_whether_valence_decides_the_stability() {
    sweep("ESFERA UV (polos de valência alta)", || {
        ridged(shapes::uv_sphere(48, 64, 1.0))
    });
    sweep("A MALHA DO PRODUTO (cubo subdividido)", || {
        ridged(shapes::sculpt_sphere(1.0))
    });
}
