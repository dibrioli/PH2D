//! F4_2 E7: o escoamento leva as somas com a massa (a secagem fica desligada — assentar é da E8).

use crate::grid::Grid;
use crate::painter::Engine;
use crate::sim::LeiDaTinta;
use ph2d_paint_tinta::{Soma, Tinta};

const W: usize = 96;
const H: usize = 64;
/// O factor fixo massa → volume das somas pousadas à mão (qualquer um serve: a lei é linear).
const K: f32 = 0.37;

fn traco(e: &mut Engine, x0: f64, x1: f64, y: f64) {
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 6.0f64).min(x1);
        e.dispatch_pressure_dab(x, y, 6.0, 40.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

/// Pinta com a lei velha, pousa as somas `cor(x)` coerentes com a massa e corre `passos` com `Pigmento`.
fn escorre(cor: impl Fn(usize) -> Tinta, passos: usize) -> (Engine, Vec<f32>) {
    let mut e = Engine::new(W, H);
    e.sim.lei = crate::assenta::Lei::Cobre;
    e.sliders.water = 1.0;
    e.sim.gravity_override = Some([0.0, 0.04]);
    traco(&mut e, 16.0, 80.0, 22.0);
    traco(&mut e, 16.0, 80.0, 28.0);
    let g = &mut e.layers[0].grid;
    crate::grid::tinta::garante(g);
    for (i, c) in g.tinta.iter_mut().enumerate() {
        let tc = cor(i % g.s);
        c.filme = Soma::de(tc, g.susp[i].max(0.0) * K);
        c.aberto = Soma::de(tc, g.sett[i].max(0.0) * K);
    }
    let antes = g.susp.clone();
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    for n in 1..=passos {
        passo_do_escoamento(&mut e, n);
    }
    (e, antes)
}

/// Os estágios 0, 2–5 do `sim_step_stage` (sem o 1, a secagem: assentar é da E8; o anel de dreno fica longe).
fn passo_do_escoamento(e: &mut Engine, n: usize) {
    use crate::solver as s;
    let p = e.sim.gather_params(&e.tuning);
    let [gx, gy] = e.sim.gravity(&e.tuning);
    let g = &mut e.layers[0].grid;
    if n.is_multiple_of(2) {
        s::rebuild_active_region(g);
    }
    if n.is_multiple_of(4) {
        s::build_flow_field_jacobi(g, &p, gx, gy, false);
    } else {
        s::smooth_velocity(g, &p);
        s::diffusion_pass(g, &p);
    }
    s::advect_jacobi(g, &p, gx, gy);
    s::sedimenta(g, 0.2, gx, gy, false);
    s::apply_boundaries(g, false);
    // ⚠️ A porta do dreno vive em `project.rs` (fora da E7): aqui chama-se à mão — ver E7_escoamento.md.
    super::drena_o_anel(g);
    if n.is_multiple_of(3) {
        s::project(g, &p);
        s::apply_boundaries(g, true);
    }
}

fn grade(e: &Engine) -> &Grid {
    &e.layers[0].grid
}

#[test]
fn a_advecao_conserva_somas() {
    let azul = Tinta::de_srgb([0.1, 0.2, 0.9], 0.3);
    let (e, antes) = escorre(|_| azul, 60);
    let g = grade(&e);
    let mexeu = antes
        .iter()
        .zip(&g.susp)
        .filter(|(a, d)| (*a - *d).abs() > 1e-3)
        .count();
    assert!(
        mexeu > 50,
        "a água tem de escorrer: {mexeu} células mudaram"
    );
    let massa: f64 = (0..g.cells)
        .map(|i| f64::from(g.susp[i].max(0.0)) + f64::from(g.sett[i].max(0.0)))
        .sum();
    let vol: f64 = g
        .tinta
        .iter()
        .map(|c| f64::from(c.filme.v) + f64::from(c.aberto.v))
        .sum();
    let rel = (vol / f64::from(K) - massa).abs() / massa;
    assert!(
        rel < 1e-4,
        "Σv/K = {} × massa {massa}: rel {rel:e}",
        vol / f64::from(K)
    );
    let topo = g.susp.iter().copied().fold(0.0f32, f32::max);
    let mut pior = 0.0f64;
    for i in 0..g.cells {
        if g.susp[i] > 1e-2 * topo {
            let r = f64::from(g.tinta[i].filme.v) / f64::from(g.susp[i]) / f64::from(K);
            pior = pior.max((r - 1.0).abs());
        }
    }
    assert!(pior < 1e-4, "filme.v/susp fugiu da constante: rel {pior:e}");
}

#[test]
fn a_cor_nao_muda_quando_a_tinta_e_uma_so() {
    let cor = Tinta::de_srgb([0.8, 0.3, 0.1], 0.5);
    let (e, _) = escorre(|_| cor, 60);
    let mut n = 0;
    for c in &grade(&e).tinta {
        if c.filme.v > 1e-6 {
            let t = c.filme.cor().unwrap();
            for k in 0..3 {
                assert!((t.lab[k] - cor.lab[k]).abs() < 1e-4, "{t:?} × {cor:?}");
            }
            assert!((t.transparencia - cor.transparencia).abs() < 1e-4);
            n += 1;
        }
    }
    assert!(n > 100, "{n} células com tinta");
}

#[test]
fn duas_cores_misturam_pela_soma() {
    let azul = Tinta::de_srgb([0.1, 0.2, 0.9], 0.2);
    let amarelo = Tinta::de_srgb([0.95, 0.85, 0.1], 0.8);
    let (e, _) = escorre(|x| if x < W / 2 { azul } else { amarelo }, 80);
    let mut mistas = 0;
    for c in &grade(&e).tinta {
        let Some(t) = c.filme.cor() else { continue };
        if c.filme.v < 1e-6 {
            continue;
        }
        for k in 0..3 {
            let (lo, hi) = (
                azul.lab[k].min(amarelo.lab[k]),
                azul.lab[k].max(amarelo.lab[k]),
            );
            assert!(
                t.lab[k] >= lo - 1e-4 && t.lab[k] <= hi + 1e-4,
                "{t:?} fora da mistura"
            );
        }
        if (t.lab[0] - azul.lab[0]).abs() > 1e-3 && (t.lab[0] - amarelo.lab[0]).abs() > 1e-3 {
            mistas += 1;
        }
    }
    assert!(mistas > 0, "nenhuma célula misturou");
}
