//! **Quanto custa perguntar a silhueta de uma forma com traço** — o número que decide se o campo
//! de distância exato pode alcançar as formas TRAÇADAS (hoje elas caem no caminho do raster, cuja
//! semente discreta desenha o pente no bevel).
//!
//! O FX re-cozinha por forma quando ela MUDA (não por frame parado), então a barra é *cabe num
//! frame de autoria*, não *é grátis*.
//!
//! Rodar: `cargo test -p ph2d-vec-boolean --test silhouette_cost --release -- --ignored --nocapture`

use ph2d_vec_scene::{Rgba8, ShapeKind, StrokeSpec, VecPath, cook};

fn bbox(p: &VecPath) -> [f64; 4] {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in p
        .verts
        .iter()
        .chain(p.subpaths.iter().flat_map(|c| c.verts.iter()))
    {
        for a in 0..2 {
            lo[a] = lo[a].min(v.anchor[a]);
            hi[a] = hi[a].max(v.anchor[a]);
        }
    }
    [lo[0], lo[1], hi[0], hi[1]]
}

fn star(points: f64) -> VecPath {
    cook(
        ShapeKind::Star,
        [-2.0, -2.0],
        [2.0, 2.0],
        &[points, 0.45, 0.0],
    )
}

#[test]
#[ignore = "medicao; roda com --ignored --nocapture"]
fn what_a_silhouette_costs() {
    for points in [5.0_f64, 12.0, 40.0] {
        let mut p = star(points);
        p.fill = Some(ph2d_vec_scene::Paint::Solid(Rgba8::new(160, 40, 200, 255)));
        p.stroke = Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.10));
        // Aquece: o 1o sweep paga a alocacao de arena.
        let _ = ph2d_vec_boolean::silhouette_paths(&p);
        let before = ph2d_vec_boolean::__sweep_calls();
        let t = std::time::Instant::now();
        let n = 20u32;
        let mut out = 0usize;
        for _ in 0..n {
            out = ph2d_vec_boolean::silhouette_paths(&p).len();
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0 / f64::from(n);
        let sweeps = (ph2d_vec_boolean::__sweep_calls() - before) / u64::from(n);
        let sil = ph2d_vec_boolean::silhouette_paths(&p);
        let verts: usize = sil
            .iter()
            .map(|q| q.verts.len() + q.subpaths.iter().map(|c| c.verts.len()).sum::<usize>())
            .sum();
        let subs: usize = sil.iter().map(|q| q.subpaths.len()).sum();
        let bs = bbox(&sil[0]);
        let bf = bbox(&p);
        eprintln!(
            "[silhueta] estrela {points} pontas: {ms:.3} ms - {sweeps} sweeps - {out} path(s), {subs} subpath(s), {verts} verts\n\
             \x20          bbox fonte [{:.4},{:.4}]..[{:.4},{:.4}] - silhueta [{:.4},{:.4}]..[{:.4},{:.4}] (cresceu {:.4})",
            bf[0],
            bf[1],
            bf[2],
            bf[3],
            bs[0],
            bs[1],
            bs[2],
            bs[3],
            bf[0] - bs[0]
        );
        assert!(out > 0, "a silhueta saiu vazia");
    }
    // O CONTROLE: sem traco nao ha sweep nenhum, e o caminho de hoje fica byte-identico.
    let plain = star(5.0);
    let before = ph2d_vec_boolean::__sweep_calls();
    let out = ph2d_vec_boolean::silhouette_paths(&plain);
    assert_eq!(
        ph2d_vec_boolean::__sweep_calls(),
        before,
        "uma forma SEM traco nao pode custar um sweep"
    );
    assert_eq!(out.len(), 1, "sem traco a silhueta e o proprio path");
    eprintln!("[silhueta] sem traco: 0 sweeps");
}
