//! Gate da secagem na placa (degrau 5) contra `drying_pass_jacobi_rows` da CPU, sobre a aguada de
//! `sonda_secagem.rs` (`nucleos_wash`, `Lei::Cobre`) reduzida. Números: PLACA_degrau5.md.

use ph2d_gpu::GpuContext;
use ph2d_wet_gpu::seca::{SecaGpu, SecaU};
use ph2d_wet_paint::assenta::Ecra;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::opacity::alpha_of_mass;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::Rows;
use ph2d_wet_paint::sim::Params;
use ph2d_wet_paint::tuning::Knob;

const W: usize = 512;
const H: usize = 320;

fn faixa(e: &mut Engine, y: f64) {
    let (x0, x1) = (-64.0, W as f64 + 64.0);
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 16.0f64).min(x1);
        e.dispatch_pressure_dab(x, y, 6.0, 60.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

/// A montagem de `nucleos_wash.rs` / `sonda_secagem::wash`, em W×H.
fn wash() -> (Grid, Params, f64, f64, bool) {
    let mut e = Engine::new(W, H);
    e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA;
    e.sim.glaze_filme = ph2d_wet_paint::colorops::VELATURA_DO_FILME;
    e.sim.pickup_assentado = 0.0;
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    e.wet_canvas_now();
    let mut y = -32.0;
    while y < H as f64 + 32.0 {
        faixa(&mut e, y);
        y += 12.0;
    }
    let p = e.sim.gather_params(&e.tuning);
    let evap = e.sim.evap_scale * p.k(Knob::Evaporation);
    let rewet = e.sim.rewet_base * p.k(Knob::Rewet);
    let by = e.sim.ext_bypass;
    (e.active_grid_mut().clone(), p, evap, rewet, by)
}

fn cena(p: &mut Params, g: &mut Grid, k: usize) {
    if k == 1 {
        if let Ecra::Cobre {
            esconde,
            molhada,
            oleo,
            ..
        } = p.ecra
        {
            p.ecra = Ecra::Cobre {
                opaca: 0.0,
                filme: 0.3,
                esconde,
                molhada,
                vela: 0.0,
                oleo,
            };
        }
        let n = g.vela.len();
        for (i, x) in g.vela.iter_mut().enumerate() {
            *x = if i < n / 2 { 0.8 } else { 0.0 };
        }
    }
}

fn ver(g: &Grid, p: &Params) -> Vec<[f64; 3]> {
    (0..g.film.len())
        .map(|i| {
            ph2d_wet_paint::assenta::sobre_branco(
                p.ecra.com_vela(g.vela[i]),
                g.sett_rgb[i].map(f64::from),
                g.susp_rgb[i].map(f64::from),
                alpha_of_mass(f64::from(g.sett[i])),
                alpha_of_mass(f64::from(g.susp[i])),
            )
        })
        .collect()
}

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

fn maxdif(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| f64::from((x - y).abs()))
        .fold(0.0, f64::max)
}

/// Massa (film, susp, sett) placa × CPU após 1 secagem, relativa a `max(|v|, 1)`.
/// Medido 31/10: RTX 2,3e-5, Radeon 1,9e-5 (o arredondamento f32 de `susp − dm` com `dm ≈ susp`).
const EPS_MASSA_REL: f64 = 1e-4;
/// Células cujo rgb (a cor GUARDADA) diverge > 1 nível após 1 secagem: no máximo 1 em `RAMOS_POR`.
/// Medido 31/10, cena Glaze: RTX 148, Radeon 126 em 163 840 (≈ 1 em 1 100); cena 0: 0. O ramo é
/// o `assenta` com `a_s = 0` e `a_s2` pequeno sob Glaze: a cor guardada mal se vê no ecrã
/// (`∂ecrã/∂x ≈ CEGO_COM_GLAZE = 0,01`), o Newton pára a 0,01 nível do ecrã e o canal cego vira por
/// f32 × f64 — a cor guardada difere, o ecrã não (`EPS_ECRA_1`).
const RAMOS_POR: usize = 400;
/// O ecrã seco placa × CPU após 1 secagem, em níveis: UM degrau da tabela de opacidade (0,002 × 255);
/// a massa que cai noutro índice truncado de `alpha_of_mass` (f32 × f64) é o limiar. Medido RTX 0,18, Radeon 0,22 (cena 0); cena Glaze 0,64 nas DUAS (o ramo, visto com a molhada da vista) — tecto = o critério invisível (1 nível).
const EPS_ECRA_1: f64 = 1.0;

fn trunc_idx(m: f64) -> i32 {
    if m < 1.0 { 0 } else { (m as i32).min(3000) }
}

/// Conta as células cujo rgb diverge > 1 nível e, para cada uma, refaz a decisão de massa da CPU
/// (f64) e da placa (f32): o índice truncado de `alpha_of_mass` em `sett+dm` / `susp−dm`.
fn ramos_da_secagem(g: &Grid, c: &Grid, p: &Grid, u: &SecaU) -> (usize, f64) {
    let mut rel = 0.0f64;
    let mut ramos = 0;
    for y in 1..=H {
        for x in 1..=W {
            let i = x + y * g.s;
            for (a, b) in [
                (c.film[i], p.film[i]),
                (c.susp[i], p.susp[i]),
                (c.sett[i], p.sett[i]),
            ] {
                rel = rel.max(f64::from((a - b).abs()) / f64::from(a.abs()).max(1.0));
            }
            let d = (0..3)
                .map(|k| {
                    (c.susp_rgb[i][k] - p.susp_rgb[i][k])
                        .abs()
                        .max((c.sett_rgb[i][k] - p.sett_rgb[i][k]).abs())
                })
                .fold(0.0f32, f32::max);
            if d <= 1.0 {
                continue;
            }
            ramos += 1;
            if ramos > 4 {
                continue;
            }
            let mut cnt = 0;
            for dy in [-1i64, 0, 1] {
                for dx in [-1i64, 0, 1] {
                    let j = (i as i64 + dx + dy * g.s as i64) as usize;
                    cnt += u32::from(g.susp[j] > 10.0);
                }
            }
            let ev = if cnt == 9 { 1.0 } else { f64::from(cnt) / 9.0 };
            let f = f64::from(g.film[i]);
            let paper = f64::from(g.paper[i]);
            let (s, t) = (f64::from(g.susp[i]), f64::from(g.sett[i]));
            let nf = |r: f64, eb: f64, ed: f64, be: f64| {
                let v = f * r - eb * ((1.0 - ev) * ed + be);
                if v < 0.0001 { 0.0 } else { v }
            };
            let n64 = nf(
                f64::from(u.retention),
                f64::from(u.evap_base),
                f64::from(u.edge_dark),
                f64::from(u.base_evap),
            );
            let bias = (1.0 + f64::from(u.ext_gran) * 0.6 * (0.5 - paper) * 2.0).clamp(0.3, 1.7);
            let dm64 = (s * (1.0 - (n64 / f).clamp(0.0, 1.0)) * bias).min(s);
            let (f3, s3, t3) = (g.film[i], g.susp[i], g.sett[i]);
            let ev3 = ev as f32;
            let mut n32 =
                f3 * u.retention - u.evap_base * ((1.0 - ev3) * u.edge_dark + u.base_evap);
            if n32 < 0.0001 {
                n32 = 0.0;
            }
            let b3 = (1.0 + u.ext_gran * 0.6 * (0.5 - g.paper[i]) * 2.0).clamp(0.3, 1.7);
            let dm32 = (s3 * (1.0 - (n32 / f3).clamp(0.0, 1.0)) * b3).min(s3);
            eprintln!(
                "  ramo ({x},{y}) Δ {d:.1}: sett {t:.4} susp {s:.4} · f64 sett+dm {:.6} [{}] susp−dm {:.6} [{}] · f32 sett+dm {:.6} [{}] susp−dm {:.6} [{}] · rgb CPU {:?}/{:?} placa {:?}/{:?} antes {:?}/{:?} vela {}",
                t + dm64,
                trunc_idx(t + dm64),
                s - dm64,
                trunc_idx(s - dm64),
                t3 + dm32,
                trunc_idx(f64::from(t3 + dm32)),
                s3 - dm32,
                trunc_idx(f64::from(s3 - dm32)),
                c.sett_rgb[i],
                c.susp_rgb[i],
                p.sett_rgb[i],
                p.susp_rgb[i],
                g.sett_rgb[i],
                g.susp_rgb[i],
                g.vela[i]
            );
        }
    }
    (ramos, rel)
}

#[test]
fn a_secagem_na_placa_casa_com_a_cpu() {
    let Some(gpu) = contexto() else { return };
    let (g0, p0, evap, rewet, by) = wash();
    let passes: usize = std::env::var("SECA_PASSES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(150);
    let cross: u32 = std::env::var("SECA_CROSS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let mut piores = Vec::new();
    for k in 0..2 {
        let (mut g, mut p) = (g0.clone(), p0.clone());
        cena(&mut p, &mut g, k);
        let mut u = SecaU::new(&g, &p, evap, rewet, by);
        u.cross = cross;
        let a = SecaGpu::new(&gpu, &g, u);
        // 1 secagem: campos.
        let mut c1 = g.clone();
        ph2d_wet_paint::drying::drying_pass_jacobi_rows(&mut c1, &p, evap, rewet, by, Rows::Serial);
        a.step(&gpu, 1);
        let mut g1 = g.clone();
        a.readback(&gpu, &mut g1);
        let rgb = |v: &[[f32; 3]]| v.iter().flatten().copied().collect::<Vec<f32>>();
        eprintln!(
            "cena {k} · 1 secagem: film {:.2e} · susp {:.2e} · sett {:.2e} · susp_rgb {:.2e} · sett_rgb {:.2e}",
            maxdif(&c1.film, &g1.film),
            maxdif(&c1.susp, &g1.susp),
            maxdif(&c1.sett, &g1.sett),
            maxdif(&rgb(&c1.susp_rgb), &rgb(&g1.susp_rgb)),
            maxdif(&rgb(&c1.sett_rgb), &rgb(&g1.sett_rgb)),
        );
        let (ramos, rel) = ramos_da_secagem(&g, &c1, &g1, &u);
        let (e1c, e1g) = (ver(&c1, &p), ver(&g1, &p));
        let ecra1 = e1c
            .iter()
            .zip(&e1g)
            .flat_map(|(a, b)| (0..3).map(move |ch| (a[ch] - b[ch]).abs()))
            .fold(0.0f64, f64::max);
        eprintln!(
            "cena {k} · 1 secagem: massa rel. máx {rel:.2e} · células com Δrgb > 1 nível: {ramos} · ecrã Δ máx {ecra1:.4}"
        );
        assert!(rel <= EPS_MASSA_REL, "cena {k}: massa relativa {rel:.2e}");
        assert!(
            ecra1 <= EPS_ECRA_1,
            "cena {k}: ecrã após 1 secagem Δ {ecra1:.4}"
        );
        assert!(
            ramos * RAMOS_POR <= W * H,
            "cena {k}: {ramos} células viram de ramo (tecto 1 em {RAMOS_POR})"
        );
        // A aguada inteira: `passes` secagens, cor final.
        let mut cn = c1;
        for _ in 1..passes {
            ph2d_wet_paint::drying::drying_pass_jacobi_rows(
                &mut cn,
                &p,
                evap,
                rewet,
                by,
                Rows::Parallel,
            );
        }
        a.step(&gpu, passes as u32 - 1);
        let mut gn = g.clone();
        a.readback(&gpu, &mut gn);
        let molh = cn.film.iter().filter(|&&f| f > 0.0).count();
        let (bc, bg) = (ver(&cn, &p), ver(&gn, &p));
        let mut dif: Vec<f64> = Vec::new();
        let mut q8 = 0i32;
        for y in 1..=H {
            for x in 1..=W {
                let i = x + y * g.s;
                for ch in 0..3 {
                    dif.push((bc[i][ch] - bg[i][ch]).abs());
                    q8 = q8.max((bc[i][ch].round() as i32 - bg[i][ch].round() as i32).abs());
                }
            }
        }
        dif.sort_by(f64::total_cmp);
        let n = dif.len();
        let (mx, p999, p99) = (
            dif[n - 1],
            dif[(n as f64 * 0.999) as usize],
            dif[(n as f64 * 0.99) as usize],
        );
        eprintln!(
            "cena {k} · {passes} secagens (molhadas CPU {molh}): |Δ| máx {mx:.4} · p99.9 {p999:.4} · p99 {p99:.4} · u8 máx {q8} · Δ>0,5 {}",
            dif.iter().filter(|&&d| d > 0.5).count()
        );
        piores.push((k, mx, p999));
    }
    for (k, mx, p999) in piores {
        assert!(
            mx <= 1.0 && p999 <= 0.5,
            "cena {k}: a cor final da placa sai do invisível (máx {mx:.3}, p99.9 {p999:.3})"
        );
    }
}

/// Bug #69: a placa lê a borda da CÉLULA como a CPU, e a célula sem borda própria lê o fundo.
#[test]
fn a_borda_de_cada_celula_casa_com_a_cpu() {
    use ph2d_wet_paint::grid::aparencia::AparenciaCelula;
    let Some(gpu) = contexto() else { return };
    let (mut g, p, evap, rewet, by) = wash();
    g.borda_fundo = 4.0;
    g.aparencia = (0..g.cells)
        .map(|i| AparenciaCelula {
            borda: if i < g.cells / 2 { 60.0 } else { -1.0 },
            ..AparenciaCelula::NEUTRA
        })
        .collect();
    let a = SecaGpu::new(&gpu, &g, SecaU::new(&g, &p, evap, rewet, by));
    let mut cpu = g.clone();
    ph2d_wet_paint::drying::drying_pass_jacobi_rows(&mut cpu, &p, evap, rewet, by, Rows::Serial);
    a.step(&gpu, 1);
    let mut placa = g.clone();
    a.readback(&gpu, &mut placa);
    assert!(maxdif(&cpu.film, &placa.film) < 1e-3, "film");

    // O controlo: a borda da célula muda mesmo o resultado.
    let mut uniforme = g.clone();
    uniforme.aparencia.clear();
    ph2d_wet_paint::drying::drying_pass_jacobi_rows(
        &mut uniforme,
        &p,
        evap,
        rewet,
        by,
        Rows::Serial,
    );
    assert!(maxdif(&cpu.film, &uniforme.film) > 1e-3, "a cena não mede a borda");
}
