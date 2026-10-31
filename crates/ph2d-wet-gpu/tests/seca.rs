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
