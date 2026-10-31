//! Paridade CPU × placa da advecção (ADR-0146: ε medido POR PLACA; ADR-0127 D4: um passo
//! semeado afirma, a trajetória só se relata).
//!
//! ```text
//! bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo test --profile smoke -p ph2d-wet-gpu -- --ignored --nocapture
//! ```

use ph2d_gpu::GpuContext;
use ph2d_wet_gpu::{
    AdvectGpu, Estado, Rect, Uniforms, cena, pack_active, pack_state, unpack_state,
};
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::par::Rows;
use ph2d_wet_paint::sim::Sim;
use ph2d_wet_paint::solver::advect_jacobi_rows;
use ph2d_wet_paint::tuning::{Knob, Tuning};

/// ε por campo após UM passo, absoluto (campos O(1)). Medido RTX: máx 2,4e-7 (2 ulp); ver PLACA_degrau1.md.
const EPS_1PASSO: f64 = 1e-6;
/// Σ da placa contra Σ inicial, relativo. Medido RTX: 1,8e-9 (1 passo), 1,5e-8 (8 passos).
const EPS_MASSA: f64 = 1e-7;

fn params() -> ph2d_wet_paint::sim::Params {
    Sim::default().gather_params(&Tuning::default())
}

fn cpu(g: &Grid, n: usize) -> Estado {
    let p = params();
    let mut g = g.clone();
    for _ in 0..n {
        advect_jacobi_rows(&mut g, &p, 0.0, 0.0, Rows::Serial);
    }
    Estado {
        film: g.film,
        susp: g.susp,
        rgb: g.susp_rgb,
    }
}

fn gpu_run(gpu: &GpuContext, g: &Grid, n: u32) -> Estado {
    let mv = params().k(Knob::MaxVelocity);
    let a = AdvectGpu::new(gpu, g);
    a.upload(gpu, g, &Uniforms::from_grid(g, mv, Rect::full(g)));
    a.step(gpu, Rect::full(g), n);
    a.readback(gpu)
}

/// (máx, médio) de |a − b| sobre o interior.
fn desvio(g: &Grid, a: &[f32], b: &[f32]) -> (f64, f64) {
    let (mut mx, mut soma, mut n) = (0.0f64, 0.0f64, 0usize);
    for y in 1..=g.h {
        for x in 1..=g.w {
            let i = x + y * g.s;
            let d = (f64::from(a[i]) - f64::from(b[i])).abs();
            mx = mx.max(d);
            soma += d;
            n += 1;
        }
    }
    (mx, soma / n as f64)
}

fn canal(v: &[[f32; 3]], k: usize) -> Vec<f32> {
    v.iter().map(|c| c[k]).collect()
}

fn soma(v: &[f32]) -> f64 {
    v.iter().map(|&x| f64::from(x)).sum()
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

#[test]
fn o_empacotamento_e_os_uniformes_casam_com_a_grade() {
    let g = cena::poca(40, 24, 4, 0.9);
    let st = pack_state(&g.film, &g.susp);
    assert_eq!(st.len(), g.cells);
    let (f, m) = unpack_state(&st);
    assert_eq!((f, m), (g.film.clone(), g.susp.clone()));
    assert!(
        pack_active(&g.active)
            .iter()
            .zip(&g.active)
            .all(|(a, b)| *a == u32::from(*b))
    );
    assert_eq!(
        std::mem::size_of::<Uniforms>(),
        96,
        "o struct U do WGSL tem 24 × 4 B"
    );
    let u = Uniforms::from_grid(&g, 1.0, Rect::full(&g));
    assert_eq!((u.w, u.h, u.s), (40, 24, 42));
    assert_eq!(
        (u.fs, u.fw, u.fh, u.rf, u.half),
        (g.flow.s as i32, 10, 6, 4, 1)
    );
    assert_eq!(
        u.reach, 2,
        "maxVelocity 1 ⇒ alcance ceil(1)+1 (destination_reach)"
    );
    assert_eq!((u.x0, u.y0, u.x1, u.y1), (1, 1, 40, 24));
    // O `vec3` do rgb sobe como 3 × f32 contíguos — o índice `3·i` do shader.
    let b: &[f32] = bytemuck::cast_slice(&g.susp_rgb);
    assert_eq!(b[3 * 100 + 1], g.susp_rgb[100][1]);
}

#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn a_placa_casa_com_a_cpu_conserva_e_e_simetrica() {
    let Some(gpu) = contexto() else { return };
    eprintln!("| rf | passos | campo | desvio máx | desvio médio |");
    eprintln!("|---|---|---|---|---|");
    for rf in [1usize, 4] {
        let g = cena::poca(320, 200, rf, 0.9);
        let (m0f, m0s) = (soma(&g.film), soma(&g.susp));
        for n in [1usize, 8] {
            let c = cpu(&g, n);
            let p = gpu_run(&gpu, &g, n as u32);
            let campos: [(&str, Vec<f32>, Vec<f32>); 5] = [
                ("film", c.film.clone(), p.film.clone()),
                ("susp", c.susp.clone(), p.susp.clone()),
                ("r", canal(&c.rgb, 0), canal(&p.rgb, 0)),
                ("g", canal(&c.rgb, 1), canal(&p.rgb, 1)),
                ("b", canal(&c.rgb, 2), canal(&p.rgb, 2)),
            ];
            for (nome, a, b) in &campos {
                let (mx, md) = desvio(&g, a, b);
                eprintln!("| {rf} | {n} | {nome} | {mx:.3e} | {md:.3e} |");
                if n == 1 {
                    assert!(
                        mx <= EPS_1PASSO,
                        "rf {rf}: {nome} desvia {mx:e} > {EPS_1PASSO:e}"
                    );
                }
            }
            let rel = |a: f64, b: f64| ((a - b) / b).abs();
            let (cf, cs, pf, ps) = (soma(&c.film), soma(&c.susp), soma(&p.film), soma(&p.susp));
            eprintln!(
                "massa rf {rf} n {n}: Σfilm cpu {:.2e} placa {:.2e} · Σsusp cpu {:.2e} placa {:.2e} (rel. ao inicial)",
                rel(cf, m0f),
                rel(pf, m0f),
                rel(cs, m0s),
                rel(ps, m0s)
            );
            assert!(
                rel(pf, m0f) <= EPS_MASSA && rel(ps, m0s) <= EPS_MASSA,
                "a placa não conserva"
            );
        }
    }
    // Simetria: a cena espelhada dá o resultado espelhado.
    let g = cena::poca(320, 200, 1, 0.9);
    let m = cena::espelha_x(&g);
    let (a, b) = (gpu_run(&gpu, &g, 8), gpu_run(&gpu, &m, 8));
    let espelho = |v: &[f32]| -> Vec<f32> {
        let mut o = v.to_vec();
        for y in 0..g.rows {
            for x in 1..=g.w {
                o[(g.w + 1 - x) + y * g.s] = v[x + y * g.s];
            }
        }
        o
    };
    for (nome, x, y) in [("film", &a.film, &b.film), ("susp", &a.susp, &b.susp)] {
        let (mx, md) = desvio(&g, &espelho(x), y);
        eprintln!("simetria 8 passos {nome}: máx {mx:.3e} médio {md:.3e}");
        assert!(mx <= 8.0 * EPS_1PASSO, "{nome}: o espelho desvia {mx:e}");
    }
}

/// Gravidade do gate da cadeia (inclinação leve, para o `momentum` ter o que somar).
const GRAV: (f64, f64) = (0.004, 0.012);

/// O quadro de suavização na CPU: `smooth_velocity` → `advect_jacobi` (com `momentum`).
fn cpu_suave(g: &Grid, n: usize, gx: f64) -> Grid {
    let p = params();
    let mut g = g.clone();
    for _ in 0..n {
        ph2d_wet_paint::solver::smooth_velocity_rows(&mut g, &p, Rows::Serial);
        advect_jacobi_rows(&mut g, &p, gx, GRAV.1, Rows::Serial);
    }
    g
}

fn gpu_suave(gpu: &GpuContext, g: &Grid, n: u32, gx: f64) -> (Estado, [Vec<f32>; 4]) {
    let mv = params().k(Knob::MaxVelocity);
    let a = AdvectGpu::new(gpu, g);
    let mut u = Uniforms::from_grid(g, mv, Rect::full(g));
    u.gx = gx as f32;
    u.gy = GRAV.1 as f32;
    a.upload(gpu, g, &u);
    a.step_plano(gpu, &u, n, &ph2d_wet_gpu::PLANO_SUAVE);
    (a.readback(gpu), a.readback_fluxo(gpu, g.flow.cells))
}

fn desvio_grade(w: usize, h: usize, s: usize, a: &[f32], b: &[f32]) -> (f64, f64) {
    let (mut mx, mut soma) = (0.0f64, 0.0f64);
    for y in 1..=h {
        for x in 1..=w {
            let d = (f64::from(a[x + y * s]) - f64::from(b[x + y * s])).abs();
            mx = mx.max(d);
            soma += d;
        }
    }
    (mx, soma / (w * h) as f64)
}

#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn o_quadro_de_suavizacao_na_placa_casa_com_a_cpu() {
    let Some(gpu) = contexto() else { return };
    eprintln!("| rf | passos | campo | desvio máx | desvio médio |");
    eprintln!("|---|---|---|---|---|");
    for rf in [1usize, 4] {
        let g = cena::poca(320, 200, rf, 0.9);
        let (m0f, m0s) = (soma(&g.film), soma(&g.susp));
        for n in [1usize, 8, 64] {
            let c = cpu_suave(&g, n, GRAV.0);
            let (p, fl) = gpu_suave(&gpu, &g, n as u32, GRAV.0);
            let fg = g.flow;
            let fino: [(&str, &[f32], &[f32]); 2] =
                [("film", &c.film, &p.film), ("susp", &c.susp, &p.susp)];
            let fluxo: [(&str, &[f32], &[f32]); 4] = [
                ("vel_x", &c.vel_x, &fl[0]),
                ("vel_y", &c.vel_y, &fl[1]),
                ("flow_x", &c.flow_x, &fl[2]),
                ("flow_y", &c.flow_y, &fl[3]),
            ];
            let mut piores = Vec::new();
            for (nome, a, b) in fino {
                let d = desvio_grade(g.w, g.h, g.s, a, b);
                eprintln!("| {rf} | {n} | {nome} | {:.3e} | {:.3e} |", d.0, d.1);
                piores.push((nome, d.0));
            }
            let (cr, pr) = (canal(&c.susp_rgb, 0), canal(&p.rgb, 0));
            let d = desvio_grade(g.w, g.h, g.s, &cr, &pr);
            eprintln!("| {rf} | {n} | r | {:.3e} | {:.3e} |", d.0, d.1);
            piores.push(("r", d.0));
            for (nome, a, b) in fluxo {
                let d = desvio_grade(fg.w, fg.h, fg.s, a, b);
                eprintln!("| {rf} | {n} | {nome} | {:.3e} | {:.3e} |", d.0, d.1);
                piores.push((nome, d.0));
            }
            let rel = |a: f64, b: f64| ((a - b) / b).abs();
            let (pf, ps) = (soma(&p.film), soma(&p.susp));
            eprintln!(
                "massa rf {rf} n {n}: Σfilm cpu {:.2e} placa {:.2e} · Σsusp cpu {:.2e} placa {:.2e}",
                rel(soma(&c.film), m0f),
                rel(pf, m0f),
                rel(soma(&c.susp), m0s),
                rel(ps, m0s)
            );
            if n == 1 {
                for (nome, mx) in &piores {
                    assert!(
                        *mx <= EPS_1PASSO,
                        "rf {rf}: {nome} desvia {mx:e} após 1 quadro"
                    );
                }
            }
            assert!(
                rel(pf, m0f) <= EPS_MASSA * n as f64 && rel(ps, m0s) <= EPS_MASSA * n as f64,
                "a placa não conserva"
            );
        }
    }
    let g = cena::poca(320, 200, 1, 0.9);
    let m = cena::espelha_x(&g);
    let (a, _) = gpu_suave(&gpu, &g, 8, GRAV.0);
    let (b, _) = gpu_suave(&gpu, &m, 8, -GRAV.0);
    let mut am = a.film.clone();
    for y in 0..g.rows {
        for x in 1..=g.w {
            am[(g.w + 1 - x) + y * g.s] = a.film[x + y * g.s];
        }
    }
    let (mx, md) = desvio_grade(g.w, g.h, g.s, &am, &b.film);
    eprintln!("simetria quadro suave 8 passos film: máx {mx:.3e} médio {md:.3e}");
    assert!(mx <= 8.0 * EPS_1PASSO, "o espelho desvia {mx:e}");
}

/// A cadência do produto sem secagem: quadro `k` (1-based) é de CAMPO se `k % 4 == 0`, senão suave.
fn e_campo(k: usize) -> bool {
    k.is_multiple_of(4)
}

/// `n` quadros na CPU: `build_flow_field_jacobi` ou `smooth_velocity`, depois a advecção.
fn cpu_quadros(g: &Grid, n: usize, gx: f64, primeiro: usize) -> Grid {
    let p = params();
    let mut g = g.clone();
    for k in primeiro..primeiro + n {
        if e_campo(k) {
            ph2d_wet_paint::solver::build_flow_field_jacobi_rows(
                &mut g,
                &p,
                gx,
                GRAV.1,
                false,
                Rows::Serial,
            );
        } else {
            ph2d_wet_paint::solver::smooth_velocity_rows(&mut g, &p, Rows::Serial);
        }
        advect_jacobi_rows(&mut g, &p, gx, GRAV.1, Rows::Serial);
    }
    g
}

fn gpu_quadros(
    gpu: &GpuContext,
    g: &Grid,
    n: usize,
    gx: f64,
    primeiro: usize,
) -> (Estado, [Vec<f32>; 4], Vec<u32>) {
    let p = params();
    let a = AdvectGpu::new(gpu, g);
    let mut u = Uniforms::from_grid(g, p.k(Knob::MaxVelocity), Rect::full(g));
    u.gx = gx as f32;
    u.gy = GRAV.1 as f32;
    a.upload(gpu, g, &u);
    a.set_knobs(gpu, &ph2d_wet_gpu::Knobs::from_params(&p, g.flow.rf));
    for k in primeiro..primeiro + n {
        let plano: &[usize] = if e_campo(k) {
            &ph2d_wet_gpu::PLANO_CAMPO
        } else {
            &ph2d_wet_gpu::PLANO_SUAVE
        };
        a.step_plano(gpu, &u, 1, plano);
    }
    (
        a.readback(gpu),
        a.readback_fluxo(gpu, g.flow.cells),
        a.readback_wet(gpu),
    )
}

/// Desvio da cor PESADO pela massa suspensa: `Σ susp·|Δc| / Σ susp`. A cor é `Σcp/Σp` e,
/// onde `susp` é ínfimo, um ulp de `p` vira uma cor inteira — sem peso visível nenhum.
fn desvio_cor_pesado(g: &Grid, susp: &[f32], a: &[[f32; 3]], b: &[[f32; 3]]) -> f64 {
    let (mut num, mut den) = (0.0f64, 0.0f64);
    for y in 1..=g.h {
        for x in 1..=g.w {
            let i = x + y * g.s;
            let m = f64::from(susp[i]);
            let d = (0..3)
                .map(|c| (f64::from(a[i][c]) - f64::from(b[i][c])).abs())
                .fold(0.0, f64::max);
            num += m * d;
            den += m;
        }
    }
    num / den
}

#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn o_quadro_de_campo_na_placa_casa_com_a_cpu() {
    let Some(gpu) = contexto() else { return };
    // O `campo` corre só nos quadros do caminho por omissão: confira que a cena o exercita.
    let g0 = cena::poca_papel(320, 200, 1, 0.9);
    assert!(
        g0.film.iter().any(|&f| f > 3.0),
        "a cena tem de disparar o carimbo"
    );
    eprintln!("| rf | quadros | campo | desvio máx | desvio médio | células > 1e-6 |");
    eprintln!("|---|---|---|---|---|---|");
    for rf in [1usize, 4] {
        let g = cena::poca_papel(320, 200, rf, 0.9);
        let (m0f, m0s) = (soma(&g.film), soma(&g.susp));
        // `primeiro = 4`: o 1.º quadro é de CAMPO — o gate de 1 passo mede o campo sozinho.
        for n in [1usize, 8, 64] {
            let c = cpu_quadros(&g, n, GRAV.0, 4);
            let (p, fl, wet) = gpu_quadros(&gpu, &g, n, GRAV.0, 4);
            let fg = g.flow;
            let conta = |w: usize, h: usize, s: usize, a: &[f32], b: &[f32]| {
                let mut k = 0usize;
                for y in 1..=h {
                    for x in 1..=w {
                        if (f64::from(a[x + y * s]) - f64::from(b[x + y * s])).abs() > 1e-6 {
                            k += 1;
                        }
                    }
                }
                k
            };
            let mut piores = Vec::new();
            for (nome, a, b) in [("film", &c.film, &p.film), ("susp", &c.susp, &p.susp)] {
                let d = desvio_grade(g.w, g.h, g.s, a, b);
                let k = conta(g.w, g.h, g.s, a, b);
                eprintln!("| {rf} | {n} | {nome} | {:.3e} | {:.3e} | {k} |", d.0, d.1);
                piores.push((nome, d.0));
            }
            for (nome, a, b) in [
                ("vel_x", &c.vel_x, &fl[0]),
                ("vel_y", &c.vel_y, &fl[1]),
                ("flow_x", &c.flow_x, &fl[2]),
                ("flow_y", &c.flow_y, &fl[3]),
            ] {
                let d = desvio_grade(fg.w, fg.h, fg.s, a, b);
                let k = conta(fg.w, fg.h, fg.s, a, b);
                eprintln!("| {rf} | {n} | {nome} | {:.3e} | {:.3e} | {k} |", d.0, d.1);
                piores.push((nome, d.0));
            }
            let cor = desvio_cor_pesado(&g, &c.susp, &c.susp_rgb, &p.rgb);
            eprintln!("| {rf} | {n} | cor (pesada por susp) | {cor:.3e} | — | — |");
            let wet_dif = c
                .wet
                .iter()
                .zip(&wet)
                .filter(|(a, b)| u32::from(**a) != **b)
                .count();
            eprintln!("| {rf} | {n} | wet (bytes diferentes) | {wet_dif} | — | — |");
            let rel = |a: f64, b: f64| ((a - b) / b).abs();
            let (pf, ps) = (soma(&p.film), soma(&p.susp));
            eprintln!(
                "massa rf {rf} n {n}: Σfilm cpu {:.2e} placa {:.2e} · Σsusp cpu {:.2e} placa {:.2e}",
                rel(soma(&c.film), m0f),
                rel(pf, m0f),
                rel(soma(&c.susp), m0s),
                rel(ps, m0s)
            );
            if n == 1 {
                for (nome, mx) in &piores {
                    assert!(
                        *mx <= EPS_1PASSO,
                        "rf {rf}: {nome} desvia {mx:e} após 1 quadro de campo"
                    );
                }
                assert_eq!(wet_dif, 0, "rf {rf}: o carimbo diverge");
            }
            assert!(
                rel(pf, m0f) <= EPS_MASSA * n as f64 && rel(ps, m0s) <= EPS_MASSA * n as f64,
                "a placa não conserva"
            );
        }
    }
    let g = cena::poca_papel(320, 200, 1, 0.9);
    let m = cena::espelha_x(&g);
    let (a, _, _) = gpu_quadros(&gpu, &g, 8, GRAV.0, 4);
    let (b, _, _) = gpu_quadros(&gpu, &m, 8, -GRAV.0, 4);
    let mut am = a.film.clone();
    for y in 0..g.rows {
        for x in 1..=g.w {
            am[(g.w + 1 - x) + y * g.s] = a.film[x + y * g.s];
        }
    }
    let (mx, md) = desvio_grade(g.w, g.h, g.s, &am, &b.film);
    // A LEI da CPU já não é espelhada (o retro-traço do freio salta de linha nas bordas):
    // a placa tem de reproduzir o defeito da CPU, não um espelho que a CPU não tem.
    let (ca, cb) = (
        cpu_quadros(&g, 8, GRAV.0, 4),
        cpu_quadros(&m, 8, -GRAV.0, 4),
    );
    let mut cam = ca.film.clone();
    for y in 0..g.rows {
        for x in 1..=g.w {
            cam[(g.w + 1 - x) + y * g.s] = ca.film[x + y * g.s];
        }
    }
    let (cmx, cmd) = desvio_grade(g.w, g.h, g.s, &cam, &cb.film);
    eprintln!(
        "simetria quadros de campo 8 passos film: placa máx {mx:.3e} médio {md:.3e} · cpu máx {cmx:.3e} médio {cmd:.3e}"
    );
    assert!(
        (mx - cmx).abs() <= 8.0 * EPS_1PASSO,
        "o espelho da placa ({mx:e}) não é o da CPU ({cmx:e})"
    );
}
