//! Quanto custa a secagem na placa (degrau 5), sobre a aguada de `sonda_secagem::wash` em 4096×2560:
//! ms por secagem e do passo inteiro com secagem 1 em 6 (o `plano_passo` do degrau 4 + `seca`),
//! mínimo de `RONDAS`. `SECA_COR=1`: o gate de cor (150 secagens, placa × CPU) nesta cena.
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo run --profile smoke -p ph2d-wet-gpu --example mede_seca
//! ```
//! (Radeon: `env VK_ICD_FILENAMES=… VK_DRIVER_FILES=…` do README §2b de docs/Painter/49_realshader/.)

use std::time::Instant;

use ph2d_gpu::GpuContext;
use ph2d_wet_gpu::seca::{SecaGpu, SecaU};
use ph2d_wet_gpu::{AdvectGpu, Knobs, Rect, Uniforms, plano_passo};
use ph2d_wet_paint::assenta::{Ecra, sobre_branco};
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::opacity::alpha_of_mass;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::Rows;
use ph2d_wet_paint::sim::Params;
use ph2d_wet_paint::tuning::Knob;

const RONDAS: usize = 7;
const BLOCO: u32 = 6;

fn faixa(e: &mut Engine, w: usize, y: f64) {
    let (x0, x1) = (-64.0, w as f64 + 64.0);
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 16.0f64).min(x1);
        e.dispatch_pressure_dab(x, y, 6.0, 60.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

fn wash(w: usize, h: usize) -> (Grid, Params, f64, f64, bool) {
    let mut e = Engine::new(w, h);
    e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA;
    e.sim.glaze_filme = ph2d_wet_paint::colorops::VELATURA_DO_FILME;
    e.sim.pickup_assentado = 0.0;
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    e.wet_canvas_now();
    let mut y = -32.0;
    while y < h as f64 + 32.0 {
        faixa(&mut e, w, y);
        y += 12.0;
    }
    let p = e.sim.gather_params(&e.tuning);
    let evap = e.sim.evap_scale * p.k(Knob::Evaporation);
    let rewet = e.sim.rewet_base * p.k(Knob::Rewet);
    let by = e.sim.ext_bypass;
    (e.active_grid_mut().clone(), p, evap, rewet, by)
}

fn espera(gpu: &GpuContext) {
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
}

fn ver(g: &Grid, p: &Params) -> Vec<[f64; 3]> {
    (0..g.film.len())
        .map(|i| {
            ph2d_wet_paint::assenta::sobre_branco(
                p.ecra.com_vela(g.vela.get(i).copied().unwrap_or(0.0)),
                g.sett_rgb[i].map(f64::from),
                g.susp_rgb[i].map(f64::from),
                alpha_of_mass(f64::from(g.sett[i])),
                alpha_of_mass(f64::from(g.susp[i])),
            )
        })
        .collect()
}

fn main() {
    let w: usize = std::env::var("SECA_W")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4096);
    let h: usize = std::env::var("SECA_H")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2560);
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("GpuContext");
    let info = gpu.adapter.get_info();
    let t0 = Instant::now();
    let (g, p, evap, rewet, by) = wash(w, h);
    eprintln!(
        "placa: {} · aguada {w}×{h} montada em {:.1} s · loadavg {}",
        info.name,
        t0.elapsed().as_secs_f64(),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let mut s = SecaGpu::new(&gpu, &g, SecaU::new(&g, &p, evap, rewet, by));
    let a = AdvectGpu::new(&gpu, &g);
    let u = Uniforms::from_grid(&g, p.k(Knob::MaxVelocity), Rect::full(&g));
    a.upload(&gpu, &g, &u);
    a.set_knobs(&gpu, &Knobs::from_params(&p, g.flow.rf));
    espera(&gpu);

    let (mut ts, mut tp, mut ta) = (Vec::new(), Vec::new(), Vec::new());
    for r in 0..=RONDAS {
        // (a) só secagens.
        s.upload(&gpu, &g);
        espera(&gpu);
        let t = Instant::now();
        s.step(&gpu, BLOCO);
        espera(&gpu);
        let ms_s = t.elapsed().as_secs_f64() * 1e3 / f64::from(BLOCO);
        // (b) o passo sem secagem (12 passos do ciclo).
        let t = Instant::now();
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        for k in 0..12u64 {
            a.encode_plano(&mut enc, &u, 1, &plano_passo(k, false), None);
        }
        gpu.queue.submit([enc.finish()]);
        espera(&gpu);
        let ms_a = t.elapsed().as_secs_f64() * 1e3 / 12.0;
        // (c) o passo inteiro: 12 passos, a secagem no 1.º de cada 6.
        s.upload(&gpu, &g);
        espera(&gpu);
        let t = Instant::now();
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        for k in 0..12u64 {
            if k % 6 == 0 {
                s.encode(&mut enc, 1);
            }
            a.encode_plano(&mut enc, &u, 1, &plano_passo(k, false), None);
        }
        gpu.queue.submit([enc.finish()]);
        espera(&gpu);
        let ms_p = t.elapsed().as_secs_f64() * 1e3 / 12.0;
        if r > 0 {
            ts.push(ms_s);
            ta.push(ms_a);
            tp.push(ms_p);
        }
    }
    let mn = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
    eprintln!(
        "ms (parede, mínimo de {RONDAS}): secagem {:.2} · passo sem secagem {:.2} · PASSO com secagem 1 em 6 {:.2}",
        mn(&ts),
        mn(&ta),
        mn(&tp)
    );

    if std::env::var_os("SECA_COR").is_none() {
        return;
    }
    let passes: usize = std::env::var("SECA_PASSES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(150);
    for k in 0..2 {
        let (mut gc, mut pc) = (g.clone(), p.clone());
        if k == 1 {
            if let ph2d_wet_paint::assenta::Ecra::Cobre {
                esconde,
                molhada,
                oleo,
                ..
            } = pc.ecra
            {
                pc.ecra = ph2d_wet_paint::assenta::Ecra::Cobre {
                    filme: 0.3,
                    esconde,
                    molhada,
                    vela: 0.0,
                    oleo,
                };
            }
            let n = gc.vela.len();
            for (i, x) in gc.vela.iter_mut().enumerate() {
                *x = if i < n / 2 { 0.8 } else { 0.0 };
            }
        }
        s.set_u(&gpu, SecaU::new(&gc, &pc, evap, rewet, by));
        s.upload(&gpu, &gc);
        let t = Instant::now();
        s.step(&gpu, passes as u32);
        let mut gg = gc.clone();
        s.readback(&gpu, &mut gg);
        let tg = t.elapsed().as_secs_f64();
        let t = Instant::now();
        for _ in 0..passes {
            ph2d_wet_paint::drying::drying_pass_jacobi_rows(
                &mut gc,
                &pc,
                evap,
                rewet,
                by,
                Rows::Parallel,
            );
        }
        let tc = t.elapsed().as_secs_f64();
        let (bc, bg) = (ver(&gc, &pc), ver(&gg, &pc));
        let mut dif: Vec<f64> = Vec::with_capacity(w * h * 3);
        let mut q8 = 0i32;
        for y in 1..=h {
            for x in 1..=w {
                let i = x + y * g.s;
                for ch in 0..3 {
                    dif.push((bc[i][ch] - bg[i][ch]).abs());
                    q8 = q8.max((bc[i][ch].round() as i32 - bg[i][ch].round() as i32).abs());
                }
            }
        }
        dif.sort_by(f64::total_cmp);
        let n = dif.len();
        eprintln!(
            "COR cena {k} {w}×{h} · {passes} secagens (placa {tg:.1} s, CPU {tc:.1} s): |Δ| máx {:.4} · p99.9 {:.4} · p99 {:.4} · u8 máx {q8} · Δ>0,5 {}",
            dif[n - 1],
            dif[(n as f64 * 0.999) as usize],
            dif[(n as f64 * 0.99) as usize],
            dif.iter().filter(|&&d| d > 0.5).count()
        );
        if k == 1 && std::env::var_os("SECA_DIAG").is_some() {
            diag(&gpu, &mut s, &g, &pc, &bc, &bg, &gc, &gg, w, h, passes, evap, rewet, by);
        }
    }
}

/// O `resolve` da CPU em f64 com o rasto de cada derivada contra o `cego` (só diagnóstico).
fn rasto(e: Ecra, x0: [f64; 3], uc: [f64; 3], alvo: [f64; 3], a_s2: f64, a_f2: f64, cego: f64) {
    let ec = |x: [f64; 3]| sobre_branco(e, x, uc, a_s2, a_f2);
    let mut x = x0;
    let mut y = ec(x);
    for it in 0..4 {
        if (0..3).all(|c| (y[c] - alvo[c]).abs() < 0.01) {
            eprintln!("      it {it}: convergiu");
            break;
        }
        let mut mexeu = false;
        for c in 0..3 {
            let mut xh = x;
            xh[c] = if x[c] + 0.5 <= 255.0 { x[c] + 0.5 } else { x[c] - 0.5 };
            let d = (ec(xh)[c] - y[c]) / (xh[c] - x[c]);
            eprintln!(
                "      it {it} c {c}: x {:.4} y {:.4} alvo {:.4} d {d:.6} (cego {cego}) {}",
                x[c],
                y[c],
                alvo[c],
                if d.abs() < cego { "CEGO" } else { "mexe" }
            );
            if d.abs() < cego {
                continue;
            }
            let passo = ((y[c] - alvo[c]) / d).clamp(-48.0, 48.0);
            x[c] = (x[c] - passo).clamp(0.0, 255.0);
            y = ec(x);
            mexeu = true;
        }
        if !mexeu {
            break;
        }
    }
    eprintln!("      x final {x:?}");
}

#[allow(clippy::too_many_arguments)]
fn diag(
    gpu: &GpuContext,
    s: &mut SecaGpu,
    g0: &Grid,
    pc: &Params,
    bc: &[[f64; 3]],
    bg: &[[f64; 3]],
    gc: &Grid,
    gg: &Grid,
    w: usize,
    h: usize,
    passes: usize,
    evap: f64,
    rewet: f64,
    by: bool,
) {
    let mut cel: Vec<(f64, usize)> = Vec::new();
    for y in 1..=h {
        for x in 1..=w {
            let i = x + y * g0.s;
            let d = (0..3).map(|c| (bc[i][c] - bg[i][c]).abs()).fold(0.0, f64::max);
            if d > 0.9 {
                cel.push((d, i));
            }
        }
    }
    cel.sort_by(|a, b| b.0.total_cmp(&a.0));
    cel.truncate(1);
    for &(d, i) in &cel {
        eprintln!(
            "MAX ({}, {}) Δ {d:.4} vela {} · CPU sett {} susp {} film {} rgb {:?}/{:?} · placa sett {} susp {} film {} rgb {:?}/{:?}",
            i % g0.s, i / g0.s, gc.vela[i], gc.sett[i], gc.susp[i], gc.film[i], gc.sett_rgb[i], gc.susp_rgb[i],
            gg.sett[i], gg.susp[i], gg.film[i], gg.sett_rgb[i], gg.susp_rgb[i]
        );
    }
    // Passo a passo, re-sincronizado: a placa parte do estado da CPU em cada secagem.
    let mut cpu = g0.clone();
    let n = cpu.vela.len();
    for (j, x) in cpu.vela.iter_mut().enumerate() {
        *x = if j < n / 2 { 0.8 } else { 0.0 };
    }
    let mut gp = cpu.clone();
    let mut livre = cpu.clone();
    for k in 0..passes {
        s.upload(gpu, &livre);
        s.step(gpu, 1);
        s.readback(gpu, &mut livre);
        let pre = cpu.clone();
        s.upload(gpu, &cpu);
        s.step(gpu, 1);
        s.readback(gpu, &mut gp);
        ph2d_wet_paint::drying::drying_pass_jacobi_rows(&mut cpu, pc, evap, rewet, by, Rows::Parallel);
        for &(_, i) in &cel {
            let dr = (0..3)
                .map(|c| {
                    (cpu.sett_rgb[i][c] - gp.sett_rgb[i][c])
                        .abs()
                        .max((cpu.susp_rgb[i][c] - gp.susp_rgb[i][c]).abs())
                })
                .fold(0.0f32, f32::max);
            let gap = (0..3)
                .map(|c| (cpu.sett_rgb[i][c] - livre.sett_rgb[i][c]).abs())
                .fold(0.0f32, f32::max);
            if dr <= 0.002 {
                eprintln!("  secagem {k}: lockstep ≤ 0,002 · livre sett_rgb gap {gap:.4} sett {} susp {} film {}", cpu.sett[i], cpu.susp[i], cpu.film[i]);
                continue;
            }
            eprintln!("  livre gap {gap:.4}");
            let (t, u, f) = (pre.sett[i], pre.susp[i], pre.film[i]);
            eprintln!(
                "  secagem {k} célula ({}, {}): Δrgb {dr:.3} · antes sett {t} susp {u} film {f} rgb {:?}/{:?} · depois CPU sett {} susp {} rgb {:?}/{:?} · placa sett {} susp {} rgb {:?}/{:?}",
                i % g0.s, i / g0.s, pre.sett_rgb[i], pre.susp_rgb[i],
                cpu.sett[i], cpu.susp[i], cpu.sett_rgb[i], cpu.susp_rgb[i],
                gp.sett[i], gp.susp[i], gp.sett_rgb[i], gp.susp_rgb[i]
            );
            // O `assenta` da CPU, re-traçado (massa da CPU: dm = sett depois − sett antes).
            let dm = f64::from(cpu.sett[i]) - f64::from(t);
            let e = pc.ecra.com_vela(pre.vela[i]).seco();
            let (sc, uc) = (pre.sett_rgb[i].map(f64::from), pre.susp_rgb[i].map(f64::from));
            let (a_s, a_f) = (alpha_of_mass(f64::from(t)), alpha_of_mass(f64::from(u)));
            let (a_s2, a_f2) = (alpha_of_mass(f64::from(t) + dm), alpha_of_mass(f64::from(u) - dm));
            let alvo = sobre_branco(e, sc, uc, a_s, a_f);
            eprintln!("    assenta a_s {a_s} a_f {a_f} a_s2 {a_s2} a_f2 {a_f2} alvo {alvo:?} (palpite x0 = CPU não exposto; rasto a partir da saída da placa e da CPU)");
            let a_dm = if a_s < 1.0 { (1.0 - (1.0 - a_s2) / (1.0 - a_s)).clamp(0.0, 1.0) } else { 0.0 };
            let mut x0 = if a_s <= 0.0 || a_s2 <= 0.0 {
                uc
            } else {
                sobre_branco(e, sc, uc, a_s, a_dm).map(|v| (255.0 - (255.0 - v) / a_s2).clamp(0.0, 255.0))
            };
            let vela = f64::from(pre.vela[i]);
            if vela > 0.0 && a_s > 0.0 && a_s2 > 0.0 {
                for q in 0..3 {
                    let t_s = 255.0 - a_s * (255.0 - sc[q]);
                    let (f1, f2) = (255.0 - a_f * (255.0 - uc[q]), 255.0 - a_f2 * (255.0 - uc[q]));
                    let t_s2 = if f2 > 0.0 { t_s * f1 / f2 } else { t_s };
                    let produto = (255.0 - (255.0 - t_s2) / a_s2).clamp(0.0, 255.0);
                    x0[q] += vela * (produto - x0[q]);
                }
            }
            rasto(e, x0, uc, alvo, a_s2, a_f2, if vela > 0.0 { 0.01 } else { 0.1 });
        }
    }
}
