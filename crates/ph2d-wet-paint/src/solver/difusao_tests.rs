//! B13.7: a difusão Jacobi dá o mesmo bit em série e em paralelo, conserva a massa e espalha.
use super::*;
use crate::painter::Engine;

fn cena() -> (Engine, Params) {
    let mut e = Engine::new(96, 64);
    e.sim.lei = crate::assenta::Lei::Cobre;
    e.sliders.water = 1.0;
    e.tuning.set(Knob::ExtDiffusion, 0.25);
    for (k, cor) in [[200.0, 40.0, 40.0], [40.0, 70.0, 200.0]]
        .into_iter()
        .enumerate()
    {
        e.color = cor;
        let y = 24.0 + 10.0 * k as f64;
        e.pointer_down(12.0, y, None);
        let mut x = 12.0;
        while x < 84.0 {
            x += 6.0;
            e.dispatch_pressure_dab(x, y, 6.0, 40.0);
        }
        e.pointer_up();
    }
    while e.release_frame(84.0, 34.0) {}
    super::super::rebuild_active_region(&mut e.layers[0].grid);
    let p = e.sim.gather_params(&e.tuning);
    (e, p)
}

fn bits(g: &Grid) -> (Vec<u32>, Vec<[u32; 3]>) {
    (
        g.susp.iter().map(|v| v.to_bits()).collect(),
        g.susp_rgb.iter().map(|c| c.map(f32::to_bits)).collect(),
    )
}

#[test]
fn a_difusao_jacobi_e_a_mesma_em_serie_e_em_paralelo() {
    let (mut e, p) = cena();
    let g = &mut e.layers[0].grid;
    let snap = crate::grid::snapshot_grid(g);
    let massa = |g: &Grid| g.susp.iter().map(|&v| f64::from(v)).sum::<f64>();
    let m0 = massa(g);
    let antes = bits(g);
    let vivas = (0..g.cells)
        .filter(|&i| g.active[i] != 0 && g.film[i] > 0.1)
        .count();
    assert!(vivas > 500, "a cena tem {vivas} donas");
    let mut r = Vec::new();
    for mode in [Rows::Serial, Rows::Parallel, Rows::Medida(0)] {
        crate::grid::restore_grid(g, &snap);
        for _ in 0..8 {
            diffusion_pass_jacobi_rows(g, &p, mode);
        }
        assert!(g.susp.iter().all(|&v| v >= 0.0), "{mode:?}: massa negativa");
        let m1 = massa(g);
        assert!((m1 - m0).abs() <= 1e-5 * m0, "{mode:?}: massa {m0} → {m1}");
        r.push(bits(g));
    }
    assert_ne!(r[0], antes, "a difusão não mexeu");
    assert_eq!(r[0], r[1], "série × paralelo");
    assert_eq!(r[0], r[2], "série × medida");
}
