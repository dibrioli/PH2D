//! Gate (2) de docs/Painter/49_realshader/LIGACAO_PLACA.md: o [`PassoPlaca`] direto sobre o `Engine`
//! depois de um traço de aguada (sem Glaze), N passos, placa × `sim_step` da CPU, cor final sobre branco.
//!
//! ```text
//! bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo test --profile smoke -p ph2d-wet-gpu --test passo -- --ignored --nocapture
//! ```
//! (Radeon: o `env VK_ICD_FILENAMES=… VK_DRIVER_FILES=…` do README §2b de docs/Painter/49_realshader/.)

use std::time::Instant;

use ph2d_gpu::GpuContext;
use ph2d_wet_gpu::passo::{PassoPlaca, placa_sabe};
use ph2d_wet_paint::assenta::sobre_branco;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::opacity::alpha_of_mass;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::sim::{Params, sim_step};
use ph2d_wet_paint::tuning::Knob;

/// Cor final, níveis de 0–255: máx e p99.9 (o briefing da linha).
const MAX_NIVEIS: f64 = 1.0;
const P999_NIVEIS: f64 = 0.5;

fn contexto() -> Option<GpuContext> {
    match GpuContext::new(GpuContext::default_instance(), None) {
        Ok(g) => {
            let i = g.adapter.get_info();
            eprintln!("placa: {} · {:?} · {}", i.name, i.backend, i.driver_info);
            Some(g)
        }
        Err(e) => {
            eprintln!("⛔ sem placa ({e}) — nada medido");
            None
        }
    }
}

fn traco(e: &mut Engine, x0: f64, x1: f64, y: f64) {
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 8.0f64).min(x1);
        e.dispatch_pressure_dab(x, y, 6.0, 40.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

/// Três traços de aguada (lei Cobre, sem Glaze) sobre papel seco.
fn aguada(w: usize, h: usize) -> Engine {
    let mut e = Engine::new(w, h);
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.pickup_assentado = 0.0;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let (wf, hf) = (w as f64, h as f64);
    traco(&mut e, 0.1 * wf, 0.9 * wf, 0.3 * hf);
    e.color = [0.8, 0.2, 0.1];
    traco(&mut e, 0.2 * wf, 0.8 * wf, 0.5 * hf);
    e.color = [0.1, 0.3, 0.9];
    traco(&mut e, 0.15 * wf, 0.7 * wf, 0.62 * hf);
    e
}

/// A aguada com as PAREDES entre traços: o pincel com a lei (`LevelingOwnStroke`) marca o `traco` de cada célula e o
/// passo tem `sim.paredes` ligado (o pincel normal) — o caminho que antes caía na CPU.
fn aguada_com_paredes(w: usize, h: usize) -> Engine {
    let mut e = Engine::new(w, h);
    e.tuning.set(Knob::LevelingOwnStroke, 1.0);
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.pickup_assentado = 0.0;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let (wf, hf) = (w as f64, h as f64);
    // `pointer_down` não numera o traço (só `begin_direct_stroke`, a porta do produto): numera-se à mão.
    let mut t = |e: &mut Engine, x0: f64, x1: f64, y: f64| {
        e.sim.traco_actual += 1;
        traco(e, x0, x1, y);
    };
    t(&mut e, 0.1 * wf, 0.9 * wf, 0.3 * hf);
    e.color = [0.8, 0.2, 0.1];
    t(&mut e, 0.2 * wf, 0.8 * wf, 0.34 * hf);
    e.color = [0.1, 0.3, 0.9];
    t(&mut e, 0.15 * wf, 0.7 * wf, 0.38 * hf);
    e
}

fn ver(g: &Grid, p: &Params) -> Vec<[f64; 3]> {
    (0..g.film.len())
        .map(|i| {
            sobre_branco(
                p.ecra.com_vela(g.vela.get(i).copied().unwrap_or(0.0)),
                g.sett_rgb[i].map(f64::from),
                g.susp_rgb[i].map(f64::from),
                alpha_of_mass(f64::from(g.sett[i])),
                alpha_of_mass(f64::from(g.susp[i])),
            )
        })
        .collect()
}

/// `(máx, p99.9, Δ>0,5)` da cor sobre branco, só no interior.
fn desvio_cor(g: &Grid, a: &[[f64; 3]], b: &[[f64; 3]]) -> (f64, f64, usize) {
    let mut d = Vec::with_capacity(g.w * g.h * 3);
    for y in 1..=g.h {
        for x in 1..=g.w {
            let i = x + y * g.s;
            for c in 0..3 {
                d.push((a[i][c] - b[i][c]).abs());
            }
        }
    }
    d.sort_by(f64::total_cmp);
    let n = d.len();
    (
        d[n - 1],
        d[(n as f64 * 0.999) as usize],
        d.iter().filter(|&&v| v > 0.5).count(),
    )
}

#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn o_passo_placa_sobre_o_engine_casa_com_a_cpu_na_cor() {
    let Some(gpu) = contexto() else { return };
    compara(&gpu, aguada(512, 320));
}

/// O mesmo gate com as PAREDES entre traços ligadas (`sim.paredes` do pincel normal + `traco` marcado): três
/// traços que se tocam molhados, a parede tem de existir (células separadas > 0) e a cor casar ao mesmo limiar.
#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn o_passo_placa_com_paredes_casa_com_a_cpu_na_cor() {
    let Some(gpu) = contexto() else { return };
    let e = aguada_com_paredes(512, 320);
    let g = &e.layers[e.active_layer].grid;
    assert!(e.sim.paredes, "o pincel normal liga as paredes");
    assert!(!g.traco.is_empty(), "o pincel com a lei marcou o traço");
    let sep = |a: usize, b: usize| ph2d_wet_paint::grid::traco::separa(&g.traco, &g.susp, a, b);
    let viz = (1..g.h)
        .flat_map(|y| (1..g.w).map(move |x| x + y * g.s))
        .filter(|&i| sep(i, i + 1) || sep(i, i + g.s))
        .count();
    let mut ids: Vec<u32> = g.traco.clone();
    ids.sort_unstable();
    ids.dedup();
    eprintln!("PAREDES: {viz} células com vizinho separado · traços {ids:?}");
    assert!(viz > 0, "a cena tem de ter paredes");
    compara(&gpu, e);
}

fn compara(gpu: &GpuContext, mut e: Engine) {
    let gpu = gpu.clone();
    let passos = 60usize;
    let (w, h) = (
        e.layers[e.active_layer].grid.w,
        e.layers[e.active_layer].grid.h,
    );
    let idx = e.active_layer;
    let pp = e.sim.gather_params(&e.tuning);
    assert!(
        placa_sabe(&e.sim, &e.layers[idx].grid, &e.tuning),
        "a cena tem de estar no que a placa sabe: inv {} · pend {} · km {} · paredes {} · oleo {} · drip {} · \
         goleo {} · bypass {} · ext {:?} · ecra {:?}",
        e.sim.order_invariant,
        e.sim.step_pending(),
        pp.km_mixing,
        pp.paredes,
        pp.oleo,
        e.sim.drip_head,
        e.layers[idx].grid.oleo.len(),
        e.sim.ext_bypass,
        [
            pp.k(Knob::ExtFingering),
            pp.k(Knob::ExtBackrun),
            pp.k(Knob::ExtDiffusion)
        ],
        matches!(pp.ecra, ph2d_wet_paint::assenta::Ecra::Cobre { .. }),
    );
    assert!(e.layers[idx].grid.has_fluid, "o traço deixou água");
    // A CPU anda numa cópia; a placa anda sobre o próprio Engine.
    let (mut sc, mut gc) = (e.sim.clone(), e.layers[idx].grid.clone());
    let t = Instant::now();
    for _ in 0..passos {
        sim_step(&mut sc, &mut gc, &e.tuning);
    }
    let tc = t.elapsed().as_secs_f64();
    let t = Instant::now();
    let mut p = PassoPlaca::new(&gpu, &e.layers[idx].grid, &e.sim, &e.tuning);
    {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = &mut e;
        for _ in 0..passos {
            p.passo(&gpu, sim, &mut layers[idx].grid, tuning);
        }
        p.desce(&gpu, &mut layers[idx].grid);
    }
    let tg = t.elapsed().as_secs_f64();
    let gg = &e.layers[idx].grid;
    assert_eq!(e.sim.frame, sc.frame, "o relógio do passo");
    assert_eq!(
        e.sim.dry_every, sc.dry_every,
        "a cadência da secagem (vmax)"
    );
    let pr = sc.gather_params(&e.tuning);
    let (mx, p999, n05) = desvio_cor(&gc, &ver(&gc, &pr), &ver(gg, &pr));
    let massa = |g: &Grid| -> f64 {
        g.susp.iter().map(|&v| f64::from(v)).sum::<f64>()
            + g.sett.iter().map(|&v| f64::from(v)).sum::<f64>()
    };
    let wet_dif = gc.wet.iter().zip(&gg.wet).filter(|(a, b)| a != b).count();
    eprintln!(
        "COR {w}×{h} · {passos} passos (placa {tg:.2} s, CPU {tc:.2} s): máx {mx:.4} · p99.9 {p999:.4} · Δ>0,5 {n05} · \
         massa CPU {:.4} placa {:.4} · wet ≠ {wet_dif} · caixa CPU {:?} placa {:?}",
        massa(&gc),
        massa(gg),
        (gc.bx0, gc.by0, gc.bx1, gc.by1, gc.has_fluid),
        (gg.bx0, gg.by0, gg.bx1, gg.by1, gg.has_fluid),
    );
    assert!(mx <= MAX_NIVEIS, "cor máx {mx} > {MAX_NIVEIS}");
    assert!(p999 <= P999_NIVEIS, "cor p99.9 {p999} > {P999_NIVEIS}");
    // A devolução: depois do `desce` a CPU continua (faixas vivas, caixa, cadência) — 12 passos dos dois lados.
    let mut gd = gg.clone();
    let mut sd = e.sim.clone();
    for _ in 0..12 {
        sim_step(&mut sc, &mut gc, &e.tuning);
        sim_step(&mut sd, &mut gd, &e.tuning);
    }
    let (mx2, p2, _) = desvio_cor(&gc, &ver(&gc, &pr), &ver(&gd, &pr));
    eprintln!("DEVOLVIDO +12 passos na CPU: máx {mx2:.4} · p99.9 {p2:.4}");
    assert!(mx2 <= MAX_NIVEIS, "devolvido: cor máx {mx2} > {MAX_NIVEIS}");
    assert!(
        p2 <= P999_NIVEIS,
        "devolvido: cor p99.9 {p2} > {P999_NIVEIS}"
    );
}

fn espera(gpu: &GpuContext) {
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
}

/// Medida (4) de LIGACAO_PLACA.md: ms por passo do [`PassoPlaca`] sobre o Engine a 4096×2560 contra o
/// `sim_step` da CPU, e o custo de subir/descer a grade (o que o worker paga por traço). Mínimo de rondas.
#[test]
#[ignore = "placa — medida, corra com --ignored --nocapture"]
fn mede_o_passo_placa_a_4k() {
    let Some(gpu) = contexto() else { return };
    let (w, h) = (4096usize, 2560usize);
    let t = Instant::now();
    let e = aguada(w, h);
    let idx = e.active_layer;
    eprintln!("aguada {w}×{h} em {:.1} s", t.elapsed().as_secs_f64());
    assert!(placa_sabe(&e.sim, &e.layers[idx].grid, &e.tuning));
    let (sim0, g0) = (e.sim.clone(), e.layers[idx].grid.clone());
    let mut p = PassoPlaca::new(&gpu, &g0, &sim0, &e.tuning);
    espera(&gpu);
    let mn = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
    let (mut sobe, mut desce, mut placa, mut cpu) = (vec![], vec![], vec![], vec![]);
    let (mut sobe_s, mut desce_s, mut faixa) = (vec![], vec![], None);
    const N: usize = 12;
    for r in 0..4 {
        let t = Instant::now();
        p.sobe(&gpu, &g0);
        espera(&gpu);
        let ms_sobe = t.elapsed().as_secs_f64() * 1e3;
        let (mut s, mut g) = (sim0.clone(), g0.clone());
        let t = Instant::now();
        for _ in 0..N {
            p.passo(&gpu, &mut s, &mut g, &e.tuning);
        }
        let ms_p = t.elapsed().as_secs_f64() * 1e3 / N as f64;
        let t = Instant::now();
        p.desce(&gpu, &mut g);
        let ms_desce = t.elapsed().as_secs_f64() * 1e3;
        // A descida SUJA (a união das caixas) e a subida SUJA de um traço de 64 linhas (o que o worker paga).
        p.sobe(&gpu, &g0);
        let (mut s, mut g) = (sim0.clone(), g0.clone());
        for _ in 0..N {
            p.passo(&gpu, &mut s, &mut g, &e.tuning);
        }
        let t = Instant::now();
        p.desce_suja(&gpu, &mut g);
        desce_s.push(t.elapsed().as_secs_f64() * 1e3);
        let y = g.by1 as usize + 100;
        for i in y * g.s..(y + 64) * g.s {
            g.film[i] += 0.5;
        }
        let t = Instant::now();
        let fx = p.sobe_sujo(&gpu, &g);
        espera(&gpu);
        sobe_s.push(t.elapsed().as_secs_f64() * 1e3);
        faixa = fx;
        let (mut s, mut g) = (sim0.clone(), g0.clone());
        let t = Instant::now();
        for _ in 0..N {
            sim_step(&mut s, &mut g, &e.tuning);
        }
        let ms_c = t.elapsed().as_secs_f64() * 1e3 / N as f64;
        if r > 0 {
            sobe.push(ms_sobe);
            desce.push(ms_desce);
            placa.push(ms_p);
            cpu.push(ms_c);
        }
    }
    eprintln!(
        "MEDE {w}×{h} (mínimo de 3, {N} passos): placa {:.2} ms/passo · CPU {:.2} ms/passo · sobe {:.1} ms · desce {:.1} ms · \
         sobe SUJO {:.1} ms (faixa {faixa:?}) · desce SUJA {:.1} ms · caixa {:?} · loadavg {}",
        mn(&placa),
        mn(&cpu),
        mn(&sobe),
        mn(&desce),
        mn(&sobe_s),
        mn(&desce_s),
        (g0.bx0, g0.by0, g0.bx1, g0.by1),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// Item 2 de LIGACAO_PLACA.md — o ciclo do WORKER com subida/descida só das linhas sujas: N passos na placa,
/// `desce_suja`, um traço NOVO na CPU (com as paredes), `sobe_sujo` (só a faixa do traço), N passos, `desce_suja`;
/// contra a CPU que faz o mesmo. Cor ao mesmo limiar, e a faixa subida é menor que a folha.
#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn a_subida_e_a_descida_so_das_linhas_sujas_casam_com_a_cpu() {
    let Some(gpu) = contexto() else { return };
    let (w, h, n) = (512usize, 320usize, 30usize);
    let mut e = aguada_com_paredes(w, h);
    let idx = e.active_layer;
    let mut c = e.clone();
    let mut p = PassoPlaca::new(&gpu, &e.layers[idx].grid, &e.sim, &e.tuning);
    let anda = |p: &mut PassoPlaca, e: &mut Engine| {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = e;
        for _ in 0..n {
            p.passo(&gpu, sim, &mut layers[idx].grid, tuning);
        }
        p.desce_suja(&gpu, &mut layers[idx].grid);
    };
    let anda_cpu = |c: &mut Engine| {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = c;
        for _ in 0..n {
            sim_step(sim, &mut layers[idx].grid, tuning);
        }
    };
    anda(&mut p, &mut e);
    anda_cpu(&mut c);
    for m in [&mut e, &mut c] {
        m.sim.traco_actual += 1;
        m.color = [0.2, 0.7, 0.2];
        traco(m, 0.3 * w as f64, 0.6 * w as f64, 0.7 * h as f64);
    }
    let faixa = p.sobe_sujo(&gpu, &e.layers[idx].grid);
    eprintln!(
        "SUJO: faixa subida {faixa:?} de {} linhas",
        e.layers[idx].grid.rows
    );
    let (y0, y1) = faixa.expect("o traço mudou linhas");
    assert!(
        y1 - y0 + 1 < e.layers[idx].grid.rows,
        "a faixa é a folha inteira"
    );
    anda(&mut p, &mut e);
    anda_cpu(&mut c);
    let (gg, gc) = (&e.layers[idx].grid, &c.layers[idx].grid);
    assert_eq!(e.sim.frame, c.sim.frame, "o relógio do passo");
    let pr = c.sim.gather_params(&c.tuning);
    let (mx, p999, n05) = desvio_cor(gc, &ver(gc, &pr), &ver(gg, &pr));
    eprintln!("SUJO COR: máx {mx:.4} · p99.9 {p999:.4} · Δ>0,5 {n05}");
    assert!(mx <= MAX_NIVEIS, "cor máx {mx} > {MAX_NIVEIS}");
    assert!(p999 <= P999_NIVEIS, "cor p99.9 {p999} > {P999_NIVEIS}");
    // A devolução: a CPU continua dos dois lados.
    for m in [&mut e, &mut c] {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = m;
        for _ in 0..12 {
            sim_step(sim, &mut layers[idx].grid, tuning);
        }
    }
    let (gg, gc) = (&e.layers[idx].grid, &c.layers[idx].grid);
    let (mx2, p2, _) = desvio_cor(gc, &ver(gc, &pr), &ver(gg, &pr));
    eprintln!("SUJO DEVOLVIDO +12: máx {mx2:.4} · p99.9 {p2:.4}");
    assert!(
        mx2 <= MAX_NIVEIS && p2 <= P999_NIVEIS,
        "devolvido: {mx2} / {p2}"
    );
}
