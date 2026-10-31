//! F4-2 E6b: a ponta e as ferramentas movem somas. Doc: `docs/Painter/Arquitetura/E6b_pickup_e_ferramentas.md`.

use super::super::*;
use crate::grid::tinta::{M_UM, garante, soma_da_massa};
use crate::painter::Engine;
use crate::sim::LeiDaTinta;
use crate::tuning::Knob;
use ph2d_paint_tinta::{Soma, Tinta};

const AZUL: [f32; 3] = [20.0, 40.0, 200.0];
const AMARELO: [f64; 3] = [240.0, 210.0, 20.0];

/// Uma grade com um quadrado de tinta azul (filme e aberto) debaixo de uma ponta pronta a recolher.
fn cena(o: f32) -> (Grid, Trail, Params) {
    let mut e = Engine::new(64, 64);
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    e.tuning.set(Knob::Pickup, 0.5);
    let mut p = e.sim.gather_params(&e.tuning);
    p.pickup_assentado = 1.0;
    let mut g = e.layers.swap_remove(0).grid;
    garante(&mut g);
    for y in 20..44 {
        for x in 20..44 {
            let i = y * g.s + x;
            (g.susp[i], g.sett[i]) = (600.0, 900.0);
            g.tinta[i].filme = soma_da_massa(AZUL, 0.0, 600.0);
            g.tinta[i].aberto = soma_da_massa(AZUL, 0.0, 900.0);
            g.tinta[i].o = o;
        }
    }
    let mut t = Trail::default();
    t.start_stroke(32.0, 32.0, AMARELO, TrailMode::Paint);
    (t.lx0, t.ly0, t.lx1, t.ly1) = (0, 0, t.size - 1, t.size - 1);
    (g, t, p)
}

fn tela(g: &Grid) -> (f64, f64) {
    let f = g.tinta.iter().map(|c| f64::from(c.filme.v)).sum();
    let a = g.tinta.iter().map(|c| f64::from(c.aberto.v)).sum();
    (f, a)
}

fn ponta(t: &Trail) -> f64 {
    t.ponta_tinta.iter().map(|s| f64::from(s.v)).sum()
}

fn invariante(g: &Grid) {
    for (i, c) in g.tinta.iter().enumerate() {
        let tol = 1e-4 * (1.0 + f64::from(g.susp[i]) + f64::from(g.sett[i]));
        assert!(
            (f64::from(c.filme.v * M_UM) - f64::from(g.susp[i])).abs() < tol,
            "filme {i}"
        );
        assert!(
            (f64::from(c.aberto.v * M_UM) - f64::from(g.sett[i])).abs() < tol,
            "aberto {i}"
        );
    }
}

#[test]
fn o_pick_up_leva_o_aberto_pela_abertura() {
    let mut perdas = Vec::new();
    for o in [1.0, 0.5] {
        let (mut g, mut t, p) = cena(o);
        let (f0, a0) = tela(&g);
        let antes = f0 + a0 + ponta(&t);
        t.tip_rows(&mut g, &p, None);
        let (f1, a1) = tela(&g);
        let depois = f1 + a1 + ponta(&t);
        assert!(ponta(&t) > 0.0, "a ponta levou tinta");
        assert!(
            ((depois - antes) / antes).abs() < 1e-5,
            "conserva: {antes} → {depois}"
        );
        invariante(&g);
        perdas.push(((f0 - f1) / f0, (a0 - a1) / a0));
    }
    let ((f1, a1), (f5, a5)) = (perdas[0], perdas[1]);
    assert!(
        (f1 - f5).abs() < 1e-6,
        "o filme não depende de o: {f1} × {f5}"
    );
    assert!(
        (a5 / a1 - 0.5).abs() < 1e-3,
        "o aberto pela abertura: {a1} × {a5}"
    );
}

#[test]
fn a_ponta_suja_pousa_a_mistura_pela_soma() {
    let (_, mut t, p) = cena(1.0);
    let mut g = Grid::new(64, 64);
    garante(&mut g);
    let azul = soma_da_massa(AZUL, 0.0, 600.0);
    t.ponta_tinta = vec![azul; (t.size * t.size) as usize];
    let c = (t.half + t.half * t.size) as usize;
    t.pig[c] = 600.0;
    // Com água: a ponta SECA (janela sem água) pousa no aberto, não no filme.
    t.water[c] = 300.0;
    t.transfer_paint(&mut g, &p);
    // A célula onde o texel central da ponta pousou (o mapeamento rasto→grade é do `transfer_paint`).
    let pousou = g
        .tinta
        .iter()
        .map(|c| c.filme)
        .max_by(|a, b| a.v.total_cmp(&b.v))
        .expect("grade");
    assert!(pousou.v > 0.0, "nada pousou");
    let l = pousou.lab[0] / pousou.v;
    let la = azul.lab[0] / azul.v;
    let ly = Tinta::de_srgb(AMARELO.map(|c| (c / 255.0) as f32), 0.0).lab[0];
    assert!(la < l && l < ly, "L entre os dois: {la} < {l} < {ly}");
    invariante(&g);
}

#[test]
fn o_smear_conserva_somas() {
    let (mut g, _, p) = cena(1.0);
    for x in 20..44 {
        let i = 30 * g.s + x;
        g.tinta[i].filme = soma_da_massa([240.0, 210.0, 20.0], 0.0, 600.0);
    }
    let foto = crate::tools::tinta::Foto::tira(&mut g, &p, [10, 10, 50, 50]).unwrap();
    let massa0: f64 = g.susp.iter().map(|&m| f64::from(m)).sum();
    // Uma relaxação para a média que conserva a massa (a forma do Blend/Smear).
    let n = 41.0 * 41.0;
    let media = (massa0 / n) as f32;
    for y in 10..=50 {
        for x in 10..=50 {
            let i = y * g.s + x;
            g.susp[i] += (media - g.susp[i]) * 0.5;
        }
    }
    foto.segue(&mut g);
    invariante(&g);
    let v: f64 = g.tinta.iter().map(|c| f64::from(c.filme.v)).sum::<f64>() * f64::from(M_UM);
    let massa1: f64 = g.susp.iter().map(|&m| f64::from(m)).sum();
    assert!(
        ((v - massa1) / massa1).abs() < 1e-5,
        "Σ soma {v} × Σ massa {massa1}"
    );
    assert!(((massa1 - massa0) / massa0).abs() < 1e-5);
    let _ = Soma::default();
}
