//! A descida suja não pode perder o que a placa escreve fora da faixa suja: a `t_cura` escreve a abertura na folha inteira.

use super::comum::{CENAS, contexto, dif_bit_sem_paredes, pinta};
use ph2d_gpu::GpuContext;
use ph2d_wet_gpu::passo::PassoPlaca;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::grid::tinta::soma_da_massa;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::sim::sim_step;

const FAIXA_Y: (usize, usize) = (170, 190);
const FAIXA_X: (usize, usize) = (40, 160);

/// Óleo diluído com uma faixa SECA, de abertura 1 e com meio, abaixo da caixa de água.
fn com_faixa_seca(w: usize, h: usize) -> Engine {
    let mut e = pinta(&CENAS[1], w, h);
    let idx = e.active_layer;
    let g = &mut e.layers[idx].grid;
    for y in FAIXA_Y.0..FAIXA_Y.1 {
        for x in FAIXA_X.0..FAIXA_X.1 {
            let i = x + y * g.s;
            g.film[i] = 0.0;
            g.susp[i] = 0.0;
            g.sett[i] = 500.0;
            g.tinta[i].filme = Default::default();
            g.tinta[i].aberto = soma_da_massa([200.0, 60.0, 30.0], 0.2, 1.0, 500.0);
            g.tinta[i].o = 1.0;
        }
    }
    e
}

fn traco_vertical(e: &mut Engine, x: f64, y0: f64, y1: f64) {
    e.pointer_down(x, y0, None);
    let mut y = y0;
    while y < y1 {
        y = (y + 6.0f64).min(y1);
        e.dispatch_pressure_dab(x, y, 6.0, 40.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

fn anda_cpu(e: &mut Engine, n: usize) {
    let idx = e.active_layer;
    let Engine {
        sim,
        layers,
        tuning,
        ..
    } = e;
    for _ in 0..n {
        sim_step(sim, &mut layers[idx].grid, tuning);
    }
}

fn anda(gpu: &GpuContext, p: &mut PassoPlaca, e: &mut Engine, n: usize) {
    let idx = e.active_layer;
    let Engine {
        sim,
        layers,
        tuning,
        ..
    } = e;
    for _ in 0..n {
        p.passo(gpu, sim, &mut layers[idx].grid, tuning);
    }
}

/// `(média da abertura na faixa, máx |a − b|)`.
fn abertura_na_faixa(a: &Grid, b: &Grid) -> (f64, f64) {
    let (mut soma, mut mx, mut n) = (0.0f64, 0.0f64, 0.0f64);
    for y in FAIXA_Y.0..FAIXA_Y.1 {
        for x in FAIXA_X.0..FAIXA_X.1 {
            let i = x + y * a.s;
            soma += f64::from(a.tinta[i].o);
            mx = mx.max(f64::from((a.tinta[i].o - b.tinta[i].o).abs()));
            n += 1.0;
        }
    }
    (soma / n, mx)
}

/// ⭐ D2: uma cura vale `2,78e-4`; a régua é 30× menor que isso e 100× maior que o arredondamento de 5 multiplicações em f32.
const ABERTURA_MAX: f64 = 1e-5;

/// A faixa seca cura; o ciclo real do worker (`desce_suja`/`sobe_sujo`) devolve a abertura à CPU, e um traço noutra coluna, que sobe as
/// linhas da faixa, não apaga a cura na placa.
#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn a_cura_fora_da_faixa_suja_desce_e_um_traco_nao_a_apaga() {
    let Some(gpu) = contexto() else { return };
    let (w, h) = (320, 200);
    let (mut ec, mut eg) = (com_faixa_seca(w, h), com_faixa_seca(w, h));
    let idx = eg.active_layer;
    let mut p = {
        let g = &eg.layers[idx].grid;
        PassoPlaca::new(&gpu, g, &eg.sim, &eg.tuning)
    };
    for _ in 0..5 {
        anda(&gpu, &mut p, &mut eg, 40);
        anda_cpu(&mut ec, 40);
        p.desce_suja(&gpu, &mut eg.layers[idx].grid);
        let subiu = p.sobe_sujo(&gpu, &eg.layers[idx].grid);
        assert!(subiu.is_none(), "a CPU não mudou nada e subiu {subiu:?}");
    }
    let gg = &eg.layers[idx].grid;
    assert!(
        gg.has_fluid && gg.by1 < FAIXA_Y.0 as i32,
        "a faixa tem de ficar fora da caixa (senão a prova é muda): caixa até {}",
        gg.by1
    );
    let (mc, d1) = abertura_na_faixa(&ec.layers[idx].grid, gg);
    assert!(mc < 0.9995, "a faixa curou (senão a prova é muda): {mc}");
    let mut inteira = gg.clone();
    p.desce(&gpu, &mut inteira);
    let dif0 = dif_bit_sem_paredes(&inteira, gg);

    for e in [&mut ec, &mut eg] {
        traco_vertical(e, 300.0, 10.0, 195.0);
    }
    let faixa = p.sobe_sujo(&gpu, &eg.layers[idx].grid);
    assert!(
        faixa.is_some_and(|(a, b)| a <= FAIXA_Y.0 && b >= FAIXA_Y.1),
        "o traço tem de subir as linhas da faixa (senão a prova é muda): {faixa:?}"
    );
    anda(&gpu, &mut p, &mut eg, 40);
    anda_cpu(&mut ec, 40);
    p.desce_suja(&gpu, &mut eg.layers[idx].grid);
    let (mc2, d2) = abertura_na_faixa(&ec.layers[idx].grid, &eg.layers[idx].grid);
    let mut inteira = eg.layers[idx].grid.clone();
    p.desce(&gpu, &mut inteira);
    let (_, d3) = abertura_na_faixa(&inteira, &eg.layers[idx].grid);
    let dif = dif_bit_sem_paredes(&inteira, &eg.layers[idx].grid);
    eprintln!(
        "cura fora da faixa: CPU {mc:.6} → {mc2:.6} · Δ antes do traço {d1:.2e} · depois {d2:.2e} · descida inteira × suja {d3:.2e} (células ≠: {dif0} antes do traço, {dif} depois)"
    );
    assert!(mc2 < mc, "a faixa continua a curar depois do traço");
    assert!(d1 <= ABERTURA_MAX, "a descida suja perdeu a cura: {d1:.2e}");
    assert!(d2 <= ABERTURA_MAX, "o traço apagou a cura: {d2:.2e}");
    assert_eq!(
        dif0, 0,
        "antes do traço, a suja não trouxe o que a inteira traz"
    );
    assert_eq!(dif, 0, "a descida suja não trouxe o que a inteira traz");
}

/// O custo da cura e da descida dela a 4K, faixa cheia, no mesmo processo; conta o MÍNIMO.
#[test]
#[ignore = "placa — medida: PH2D_GPU=1, corra com --ignored --nocapture"]
fn mede_a_descida_da_cura_a_4k() {
    let Some(gpu) = contexto() else { return };
    let (w, h) = (4096, 2560);
    let mut e = pinta(&CENAS[1], w, h);
    let idx = e.active_layer;
    {
        // A pior cena: água só na faixa de cima, e toda a folha abaixo dela com aberto e meio.
        let g = &mut e.layers[idx].grid;
        for y in (h / 2)..h {
            for x in 1..=w {
                let i = x + y * g.s;
                g.film[i] = 0.0;
                g.susp[i] = 0.0;
                g.sett[i] = 500.0;
                g.tinta[i].filme = Default::default();
                g.tinta[i].aberto = soma_da_massa([200.0, 60.0, 30.0], 0.2, 1.0, 500.0);
            }
        }
    }
    let mut p = {
        let g = &e.layers[idx].grid;
        PassoPlaca::new(&gpu, g, &e.sim, &e.tuning)
    };
    let (mut com, mut sem, mut passada) = (f64::MAX, f64::MAX, f64::MAX);
    for _ in 0..6 {
        // Até ao quadro da cura; a descida seguinte é a que paga a abertura.
        loop {
            anda(&gpu, &mut p, &mut e, 1);
            if e.sim.frame.is_multiple_of(40) {
                break;
            }
        }
        // Quadro de rebuild sem secagem: rebuild + porta + `t_cura`, nessa ordem de gravação.
        if !e.sim.frame.is_multiple_of(e.sim.dry_every) {
            let k = ph2d_wet_gpu::PLANO_REBUILD.len() + 1;
            passada = passada.min(p.ms_por_passada().get(k).copied().unwrap_or(f64::MAX));
        }
        let t = std::time::Instant::now();
        p.desce_suja(&gpu, &mut e.layers[idx].grid);
        com = com.min(t.elapsed().as_secs_f64() * 1000.0);
        anda(&gpu, &mut p, &mut e, 1);
        let t = std::time::Instant::now();
        p.desce_suja(&gpu, &mut e.layers[idx].grid);
        sem = sem.min(t.elapsed().as_secs_f64() * 1000.0);
    }
    let g = &e.layers[idx].grid;
    eprintln!(
        "4096×2560 (caixa até {}): descida depois da cura {com:.2} ms · sem cura {sem:.2} ms · `t_cura` {passada:.3} ms de placa",
        g.by1
    );
}
