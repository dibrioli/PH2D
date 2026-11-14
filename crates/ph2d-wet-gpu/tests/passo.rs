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

/// Três traços de aguada que se TOCAM molhados (a parede entre traços saiu na E10: o `Knob::Body` a substitui).
fn aguada_que_se_toca(w: usize, h: usize) -> Engine {
    let mut e = Engine::new(w, h);
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.pickup_assentado = 0.0;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let (wf, hf) = (w as f64, h as f64);
    let t = traco;
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
        "a cena tem de estar no que a placa sabe: inv {} · pend {} · km {} · oleo {} · drip {} · \
         goleo {} · bypass {} · ext {:?} · ecra {:?}",
        e.sim.order_invariant,
        e.sim.step_pending(),
        pp.km_mixing,
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
    let mut e = aguada_que_se_toca(w, h);
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

/// Degrau 6 (PLACA_degrau6.md): as submissões do passo cortadas pelo orçamento × um submit por passo (o de antes),
/// intercaladas no mesmo processo a 4096×2560. Afirma o MESMO bit (film, susp, cor, sett) e imprime, por variante, o
/// número de submissões por passo e os ms de placa da maior (carimbos), e a maior passada sozinha.
#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn mede_as_submissoes_a_4k() {
    use ph2d_wet_gpu::passo::{ORCAMENTO_GRUPOS, TETO_MS};
    let Some(gpu) = contexto() else { return };
    let (w, h) = (4096usize, 2560usize);
    let e = aguada(w, h);
    let idx = e.active_layer;
    let (sim0, g0) = (e.sim.clone(), e.layers[idx].grid.clone());
    let mut p = PassoPlaca::new(&gpu, &g0, &sim0, &e.tuning);
    espera(&gpu);
    const N: usize = 12;
    let variantes = [("um submit", u64::MAX), ("fatiado", ORCAMENTO_GRUPOS)];
    // por variante: (min ms/passo de relógio, máx submissões/passo, máx ms da maior submissão, máx ms de uma passada)
    let mut res = [(f64::INFINITY, 0usize, 0.0f64, 0.0f64); 2];
    for r in 0..4 {
        let mut finais = Vec::new();
        for k in [r % 2, 1 - r % 2] {
            p.afina(variantes[k].1, TETO_MS);
            p.sobe(&gpu, &g0);
            espera(&gpu);
            let (mut s, mut g) = (sim0.clone(), g0.clone());
            let t = Instant::now();
            for _ in 0..N {
                p.passo(&gpu, &mut s, &mut g, &e.tuning);
                let subs = p.ms_por_submissao();
                let pas = p.ms_por_passada();
                let o = &mut res[k];
                o.1 = o.1.max(subs.len());
                o.2 = subs.iter().copied().fold(o.2, f64::max);
                o.3 = pas.iter().copied().fold(o.3, f64::max);
            }
            let ms = t.elapsed().as_secs_f64() * 1e3 / N as f64;
            if r == 3 {
                let v: Vec<String> = p
                    .ms_por_passada()
                    .iter()
                    .map(|m| format!("{m:.2}"))
                    .collect();
                let u: Vec<String> = p
                    .ms_por_submissao()
                    .iter()
                    .map(|m| format!("{m:.2}"))
                    .collect();
                eprintln!(
                    "PASSADAS {} (passo 12): [{}] · submissões [{}]",
                    variantes[k].0,
                    v.join(" "),
                    u.join(" ")
                );
            }
            if r > 0 {
                res[k].0 = res[k].0.min(ms);
            }
            p.desce(&gpu, &mut g);
            finais.push((k, g));
        }
        finais.sort_by_key(|f| f.0);
        let (a, b) = (&finais[0].1, &finais[1].1);
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&a.film), bits(&b.film), "film ao bit");
        assert_eq!(bits(&a.susp), bits(&b.susp), "susp ao bit");
        assert_eq!(bits(&a.sett), bits(&b.sett), "sett ao bit");
        assert!(
            a.susp_rgb == b.susp_rgb && a.sett_rgb == b.sett_rgb,
            "cores ao bit"
        );
    }
    let i = gpu.adapter.get_info();
    for (k, (nome, orc)) in variantes.iter().enumerate() {
        let (ms, n, sub, pas) = res[k];
        eprintln!(
            "SUBMISSOES {nome} (orçamento {orc}): {ms:.2} ms/passo (mín de 3) · até {n} submissões/passo · maior submissão \
             {sub:.2} ms · maior passada {pas:.2} ms · {} · loadavg {}",
            i.name,
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
    }
}

/// F4-2 E2: a lei `Pigmento` da tinta fica na CPU até a paridade por placa (E11) — sem placa nenhuma.
#[test]
fn a_placa_aceita_a_lei_pigmento_e_recusa_o_que_nao_sabe() {
    use ph2d_wet_paint::sim::LeiDaTinta;
    let mut e = aguada(64, 48);
    let idx = e.active_layer;
    let sabe = |e: &Engine| placa_sabe(&e.sim, &e.layers[idx].grid, &e.tuning);
    assert!(sabe(&e), "a aguada é da placa com a lei velha");
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    assert!(sabe(&e), "E11: a placa sabe a lei Pigmento");

    // O meio e o corpo pousados: só a lei nova os leva à placa.
    e.layers[idx].grid.oleo =
        ph2d_wet_paint::grid::oleo_celula::campo_de(&vec![
            Default::default();
            e.layers[idx].grid.cells
        ]);
    assert!(sabe(&e), "o óleo da célula é da placa com a lei Pigmento");
    e.sim.set_tinta(LeiDaTinta::Velha);
    assert!(!sabe(&e), "a lei velha não leva o óleo da célula à placa");
    e.layers[idx].grid.oleo = ph2d_wet_paint::grid::oleo_celula::sem_campo();
    e.sim.set_tinta(LeiDaTinta::Pigmento);

    // O que fica na CPU, um a um.
    e.sim.pigment_mix = 0.5;
    assert!(!sabe(&e), "o Pigment mix (o lado espectral) fica na CPU");
    e.sim.pigment_mix = 0.0;
    e.layers[idx].grid.chao = vec![0.0; e.layers[idx].grid.cells];
    assert!(!sabe(&e), "o chão como força fica na CPU");
    e.layers[idx].grid.chao = Vec::new();
    e.set_knob(Knob::Dissolve, 0.5);
    assert!(!sabe(&e), "a água parada que dissolve fica na CPU");
    e.set_knob(Knob::Dissolve, 0.0);
    e.set_knob(Knob::EdgeSoftMass, 100.0);
    assert!(!sabe(&e), "a borda macia fica na CPU");
    e.set_knob(Knob::EdgeSoftMass, 0.0);
    e.set_knob(Knob::Opaque, 0.5);
    assert!(!sabe(&e), "a tinta opaca fica na CPU");
    e.set_knob(Knob::Opaque, 0.0);
    assert!(sabe(&e));
}

/// O estado restaurável da camada, ao bit (o `Debug` de um `f32` é o round-trip exacto, com `-0.0`).
fn estado(g: &Grid) -> Vec<String> {
    let s = ph2d_wet_paint::grid::snapshot_grid(g);
    let caixa = (s.bx0, s.by0, s.bx1, s.by1, s.has_fluid);
    vec![
        format!("{:?}", s.film),
        format!("{:?}", s.susp),
        format!("{:?}", s.susp_rgb),
        format!("{:?}", s.sett),
        format!("{:?}", s.sett_rgb),
        format!("{:?}", s.tinta),
        format!("{:?}", (&s.vel_x, &s.vel_y)),
        format!("{:?}", (&s.wet, &s.active, &s.bloom)),
        format!("{:?}", (&s.row_lo, &s.row_hi, caixa)),
    ]
}

/// **B44.9 — a história da placa marca só as linhas que a descida escreve.** O ciclo do worker (`offthread/placa.rs`):
/// passos na placa, `desce_suja` a cada 7 (a devolução), `marca_linhas(linhas_da_descida)` antes de cada descida e
/// `marca_tudo` só quando `cura_neste_passo`. Oráculo: a MESMA corrida com o `marca_tudo` de antes ao receber — o
/// undo dá o mesmo bit nas duas leis da tinta; e fora da cura nenhuma marca de tela inteira.
#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn a_historia_da_placa_marca_so_as_linhas_da_descida_e_o_undo_e_o_mesmo() {
    use ph2d_wet_paint::sim::LeiDaTinta;
    let Some(gpu) = contexto() else { return };
    for lei in [LeiDaTinta::Velha, LeiDaTinta::Pigmento] {
        let (w, h) = (320usize, 200usize);
        let mut e = Engine::new(w, h);
        e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
        e.sim.set_tinta(lei);
        e.sliders.water = 1.0;
        e.sliders.size = 1.0;
        traco(&mut e, 0.1 * w as f64, 0.9 * w as f64, 0.4 * h as f64);
        e.color = [0.8, 0.2, 0.1];
        traco(&mut e, 0.2 * w as f64, 0.6 * w as f64, 0.45 * h as f64);
        let idx = e.active_layer;
        assert!(placa_sabe(&e.sim, &e.layers[idx].grid, &e.tuning));
        // O oráculo: o registo aberto com a grade inteira marcada ao receber (o de antes).
        let mut a = e.clone();
        let _ = a.active_grid_mut();
        let mut p = PassoPlaca::new(&gpu, &e.layers[idx].grid, &e.sim, &e.tuning);
        let (mut curas, mut fora) = (0u64, 0u64);
        let marca = |p: &PassoPlaca, e: &mut Engine| {
            for (y0, y1) in p.linhas_da_descida(e.active_grid()) {
                e.marca_linhas(y0, y1);
            }
        };
        for k in 1..=95usize {
            let antes = e.marcas_da_tela_inteira();
            let cura = p.cura_neste_passo(&e.sim);
            if cura {
                let _ = e.active_grid_mut();
                curas += 1;
            }
            let Engine {
                sim,
                layers,
                tuning,
                ..
            } = &mut e;
            p.passo(&gpu, sim, &mut layers[idx].grid, tuning);
            if !cura {
                fora += e.marcas_da_tela_inteira() - antes;
            }
            if k % 7 == 0 {
                marca(&p, &mut e);
                p.desce_suja(&gpu, &mut e.layers[idx].grid);
            }
        }
        marca(&p, &mut e);
        p.desce_suja(&gpu, &mut e.layers[idx].grid);
        assert_eq!(
            fora, 0,
            "{lei:?}: {fora} passos fora da cura marcaram a tela inteira"
        );
        if lei == LeiDaTinta::Pigmento {
            assert!(
                curas >= 2,
                "controlo: a corrida não passou pela cura ({curas})"
            );
        }
        a.layers[idx].grid = e.layers[idx].grid.clone();
        a.sim = e.sim.clone();
        assert!(e.undo() && a.undo(), "{lei:?}: nada a desfazer");
        let (eb, ea) = (estado(e.active_grid()), estado(a.active_grid()));
        for (i, (x, y)) in eb.iter().zip(&ea).enumerate() {
            assert!(
                x == y,
                "{lei:?}: o undo difere do de marca_tudo no plano {i}"
            );
        }
        eprintln!("B44.9 {lei:?}: undo ao bit, {curas} curas, 0 marcas de tela inteira fora delas");
    }
}
