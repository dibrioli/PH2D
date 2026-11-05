//! E11 (F4_2 §6): a lei `Pigmento` na placa, meio e pasta incluídos. A régua é POR PLACA
//! (`project_same_bit_law_is_per_device_2026_10_24`): a placa dá o mesmo bit entre corridas e a resposta casa com a CPU
//! canónica dentro das tolerâncias medidas abaixo; bit cruzado entre fabricantes não se exige.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-wet-gpu --test tinta -- --ignored --nocapture
//! ```

#[path = "tinta/ciclo.rs"]
mod ciclo;
#[path = "tinta/comum.rs"]
mod comum;
#[path = "tinta/paridade.rs"]
mod paridade;

use comum::{CENAS, COR_MAX, contexto, cor, da_cpu, da_placa, desvio, pinta, traco};
use ph2d_wet_gpu::passo::PassoPlaca;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::sim::{LeiDaTinta, sim_step};
use ph2d_wet_paint::tuning::Knob;

/// O Smooth Edges é da célula e a composição é da CPU: a placa não o sobe nem o desce, e o valor não se perde na ida e volta
/// nem quando a CPU pousa um traço novo com a placa a meio (o espelho por hash).
#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn o_suave_da_celula_sobrevive_a_placa() {
    let Some(gpu) = contexto() else { return };
    let c = &CENAS[0];
    let pintada = || {
        let mut e = Engine::new(320, 200);
        e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
        e.sim.pickup_assentado = 0.0;
        e.sim.set_tinta(LeiDaTinta::Pigmento);
        e.set_knob(Knob::Medium, c.medium);
        e.sliders.water = 1.0;
        e.sliders.size = 1.0;
        e.sim.suave = 0.6;
        traco(&mut e, 32.0, 288.0, 60.0);
        e
    };
    let mut ec = pintada();
    let mut eg = pintada();
    let idx = eg.active_layer;
    let antes = eg.layers[idx].grid.aparencia.clone();
    assert!(
        antes.iter().any(|a| a.suave > 0.5),
        "a cena pousou o suave (senão a prova é muda)"
    );
    let teto = eg.layers[idx].grid.suave_teto;
    assert_eq!(teto, 0.6, "o tecto do suave");

    let mut p = {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = &eg;
        PassoPlaca::new(&gpu, &layers[idx].grid, sim, tuning)
    };
    let anda = |p: &mut PassoPlaca, e: &mut Engine, n: usize| {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = e;
        for _ in 0..n {
            p.passo(&gpu, sim, &mut layers[idx].grid, tuning);
        }
    };
    let anda_cpu = |e: &mut Engine, n: usize| {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = e;
        for _ in 0..n {
            sim_step(sim, &mut layers[idx].grid, tuning);
        }
    };
    anda(&mut p, &mut eg, 30);
    anda_cpu(&mut ec, 30);
    p.desce_suja(&gpu, &mut eg.layers[idx].grid);
    assert_eq!(
        eg.layers[idx].grid.aparencia, antes,
        "a descida suja mexeu na aparência"
    );
    assert_eq!(eg.layers[idx].grid.suave_teto, teto);

    // Um traço novo, com outro Smooth Edges, pousado na CPU com a placa a meio: o espelho sobe-o e a placa continua.
    for e in [&mut ec, &mut eg] {
        e.sim.suave = 0.2;
        e.color = [0.1, 0.3, 0.9];
        traco(e, 32.0, 288.0, 130.0);
    }
    let gg = &eg.layers[idx].grid;
    assert!(p.sobe_sujo(&gpu, gg).is_some(), "o traço novo sobe");
    anda(&mut p, &mut eg, 30);
    anda_cpu(&mut ec, 30);
    p.desce(&gpu, &mut eg.layers[idx].grid);

    let (gc, gg) = (&ec.layers[idx].grid, &eg.layers[idx].grid);
    assert_eq!(gg.suave_teto, gc.suave_teto, "o tecto do suave");
    let (mx, _) = desvio(gg, |i| {
        f64::from(gg.aparencia[i].suave) - f64::from(gc.aparencia[i].suave)
    });
    let n02 = gg
        .aparencia
        .iter()
        .filter(|a| (a.suave - 0.2).abs() < 0.05)
        .count();
    let n06 = gg
        .aparencia
        .iter()
        .filter(|a| (a.suave - 0.6).abs() < 0.05)
        .count();
    let (cm, _) = desvio(gg, |i| {
        let (a, b) = (cor(gc, i), cor(gg, i));
        (a[0] - b[0])
            .abs()
            .max((a[1] - b[1]).abs())
            .max((a[2] - b[2]).abs())
    });
    eprintln!(
        "suave: Δ máx placa × CPU {mx:.2e} · células 0,6 {n06} · 0,2 {n02} · cor máx {cm:.4}"
    );
    assert!(n06 > 0 && n02 > 0, "os dois valores continuam na tela");
    assert!(mx < 0.02, "o suave da célula afastou-se da CPU: {mx}");
    assert!(cm <= COR_MAX, "a cor depois do traço novo: {cm}");
}

/// A cura anda na placa e o fecho é o da CPU: as mesmas células fecham, com o mesmo fundo, e o `sett` sai da grade.
#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn a_cura_fecha_na_placa_como_na_cpu() {
    let Some(gpu) = contexto() else { return };
    let mut e = pinta(&CENAS[1], 320, 200);
    let idx = e.active_layer;
    {
        let g = &mut e.layers[idx].grid;
        for y in 170..190 {
            for x in 200..240 {
                let i = x + y * g.s;
                g.film[i] = 0.0;
                g.susp[i] = 0.0;
                g.sett[i] = 500.0;
                g.tinta[i].filme = Default::default();
                g.tinta[i].aberto = ph2d_wet_paint::grid::tinta::soma_da_massa(
                    [200.0, 60.0, 30.0],
                    0.2,
                    1.0,
                    500.0,
                );
                g.tinta[i].o = 0.010_002;
            }
        }
    }
    let (_, gc) = da_cpu(&e, 80);
    let (_, gg) = da_placa(&gpu, &e, 80);
    let fechadas = |g: &Grid| g.tinta.iter().filter(|c| c.altura_fechada > 0.0).count();
    let (nc, ng) = (fechadas(&gc), fechadas(&gg));
    let (dr, _) = desvio(&gc, |i| {
        f64::from(gc.tinta[i].r_g[0]) - f64::from(gg.tinta[i].r_g[0])
    });
    let (dh, _) = desvio(&gc, |i| {
        f64::from(gc.tinta[i].altura_fechada) - f64::from(gg.tinta[i].altura_fechada)
    });
    eprintln!("cura: fechadas CPU {nc} placa {ng} · r_g Δ máx {dr:.2e} · altura Δ máx {dh:.2e}");
    assert!(
        nc >= 800,
        "a cena tem de fechar (senão a prova é muda): {nc}"
    );
    assert_eq!(nc, ng, "as mesmas células fecham");
    assert!(
        dr < 1e-4 && dh < 1e-4,
        "o fundo fechado: r_g {dr:.2e}, altura {dh:.2e}"
    );
}

/// O custo do passo com a lei nova contra a velha, na mesma placa e no mesmo processo, em blocos intercalados; conta o MÍNIMO.
#[test]
#[ignore = "placa — medida: PH2D_GPU=1, corra com --ignored --nocapture"]
fn mede_o_passo_da_tinta_a_4k() {
    let Some(gpu) = contexto() else { return };
    let (w, h) = (4096, 2560);
    let mut velha = {
        let mut e = pinta(&CENAS[0], w, h);
        e.sim.set_tinta(LeiDaTinta::Velha);
        let idx = e.active_layer;
        e.layers[idx].grid.tinta = Vec::new();
        e
    };
    let mut nova = pinta(&CENAS[0], w, h);
    let idx = nova.active_layer;
    let mut pv = PassoPlaca::new(&gpu, &velha.layers[idx].grid, &velha.sim, &velha.tuning);
    let mut pn = PassoPlaca::new(&gpu, &nova.layers[idx].grid, &nova.sim, &nova.tuning);
    let bloco = |p: &mut PassoPlaca, e: &mut Engine| {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = e;
        let t = std::time::Instant::now();
        for _ in 0..6 {
            p.passo(&gpu, sim, &mut layers[idx].grid, tuning);
        }
        t.elapsed().as_secs_f64() * 1000.0 / 6.0
    };
    let (mut mv, mut mn) = (f64::MAX, f64::MAX);
    for r in 0..8 {
        if r % 2 == 0 {
            mv = mv.min(bloco(&mut pv, &mut velha));
            mn = mn.min(bloco(&mut pn, &mut nova));
        } else {
            mn = mn.min(bloco(&mut pn, &mut nova));
            mv = mv.min(bloco(&mut pv, &mut velha));
        }
    }
    eprintln!(
        "4096×2560: lei velha {mv:.2} ms/passo · Pigmento {mn:.2} ms/passo (mínimo de 8 blocos de 6)"
    );
}
