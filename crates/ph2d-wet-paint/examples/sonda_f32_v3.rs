//! **Sonda C (30/10) — fora do produto.** Na poça do ADR-0147 §4 (`scene_big` de
//! `tests/it/measure_parallel_rows.rs`: 4096², três faixas diagonais), um «passo» = `advect_jacobi`
//! + `drying_pass_jacobi` sobre o mesmo campo de fluxo.
//!
//! - **C.1** — a cópia em aritmética f32 (`sonda_f32_v3/k32.rs`) contra a original f64 da crate:
//!   ms/passo intercalado e o desvio por campo após N passos.
//! - **C.2** — a mesma cópia f64 compilada duas vezes no binário: x86-64-v2 (a do build) e inlinada
//!   numa função `#[target_feature(avx2,fma,bmi1,bmi2,lzcnt,movbe)]`; ms/passo e FNV-1a do estado.
//!
//! As cópias vêm do commit base `341341b233` (cabeçalho de `sonda_f32_v3/kern.rs`); a `k64` tem de
//! dar o MESMO hash que a original — é a prova de que a cópia é fiel.
//!
//! ⚠️ Chamar uma função `#[target_feature]` exige `unsafe`, e a workspace FORBIDE `unsafe` em todos
//! os alvos. Por isso o C.2 só existe sob `--cfg sonda_v3`, que só a corrida manual passa
//! (com `--cap-lints warn`, que rebaixa o `forbid` NESTE alvo e nessa invocação):
//! ```text
//! bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo rustc --profile smoke \
//!   -p ph2d-wet-paint --example sonda_f32_v3 -- --cfg sonda_v3 --cap-lints warn
//! target/smoke/examples/sonda_f32_v3
//! ```
//! Sem a cfg (o `cargo build --all-targets` de sempre) compila sob o `forbid` e corre só o C.1.

#![allow(unexpected_cfgs)]
// A cópia verbatim de `kern.rs` herda as isenções que `src/lib.rs:23-24` dá ao original.
#![allow(clippy::explicit_counter_loop, clippy::manual_clamp)]

#[path = "sonda_f32_v3/k32.rs"]
mod k32;
#[path = "sonda_f32_v3/k64.rs"]
mod k64;
#[cfg(sonda_v3)]
#[path = "sonda_f32_v3/k64v3.rs"]
mod k64v3;
#[path = "sonda_f32_v3/par.rs"]
mod par;

use std::time::Instant;

use par::Mode;
use ph2d_wet_paint::grid::{Grid, GridSnapshot, restore_grid, snapshot_grid};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::Rows;
use ph2d_wet_paint::sim::Params;
use ph2d_wet_paint::tuning::Knob;

const DIAG: f64 = std::f64::consts::FRAC_1_SQRT_2;
const ROUNDS: usize = 7;
const BLOCK: usize = 3;
const N_HASH: usize = 12;
const N_DEV: [usize; 2] = [12, 96];

/// VERBATIM `tests/util/mod.rs::drive_stroke` (o módulo de teste não é visível a um example).
fn drive_stroke(e: &mut Engine, x0: f64, y0: f64, x1: f64, y1: f64, ppf: f64, after: usize) {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let len = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = if len > 0.0 {
        (dx / len, dy / len)
    } else {
        (0.0, 0.0)
    };
    e.pointer_down(x0, y0, None);
    let (mut t, mut x, mut y) = (0.0, x0, y0);
    while t < len {
        t = (t + ppf).min(len);
        x = x0 + ux * t;
        y = y0 + uy * t;
        e.pointer_frame(x, y);
        if e.sim_should_run() {
            e.step_simulation();
        }
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
    for _ in 0..after {
        e.step_simulation();
    }
}

/// VERBATIM o `scene_big` (ADR-0147 §4).
fn scene_big() -> Engine {
    let mut e = Engine::new(4096, 4096);
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let c = 4096.0 * 0.5;
    for lane in 0..3 {
        let off = 420.0 * f64::from(lane) - 420.0;
        drive_stroke(
            &mut e,
            c - 1500.0 * DIAG + off,
            c - 1500.0 * DIAG,
            c + 1500.0 * DIAG + off,
            c + 1500.0 * DIAG,
            120.0,
            10,
        );
    }
    e
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum V {
    OrigPar,
    K64Par,
    K32Par,
    OrigSer,
    K64Ser,
    K32Ser,
    SerV3,
}

struct Ctx {
    p: Params,
    gx: f64,
    gy: f64,
    evap: f64,
    rewet: f64,
    bypass: bool,
}

#[cfg(sonda_v3)]
#[target_feature(enable = "avx2,fma,bmi1,bmi2,lzcnt,movbe")]
fn step_v3(g: &mut Grid, c: &Ctx, adv: bool, dry: bool) {
    if adv {
        k64v3::advect_jacobi_rows(g, &c.p, c.gx, c.gy, Mode::Serial);
    }
    if dry {
        k64v3::drying_pass_jacobi_rows(g, &c.p, c.evap, c.rewet, c.bypass, Mode::Serial);
    }
}

fn v3_available() -> bool {
    cfg!(sonda_v3)
        && is_x86_feature_detected!("avx2")
        && is_x86_feature_detected!("fma")
        && is_x86_feature_detected!("bmi1")
        && is_x86_feature_detected!("bmi2")
        && is_x86_feature_detected!("lzcnt")
        && is_x86_feature_detected!("movbe")
}

/// `adv`: o passe de advecção; `dry`: o de secagem (o produto seca 1 em `dry_every` passos).
fn step(v: V, g: &mut Grid, c: &Ctx, adv: bool, dry: bool) {
    use ph2d_wet_paint::{drying, solver};
    match v {
        V::OrigPar | V::OrigSer => {
            let r = if v == V::OrigPar {
                Rows::Parallel
            } else {
                Rows::Serial
            };
            if adv {
                solver::advect_jacobi_rows(g, &c.p, c.gx, c.gy, r);
            }
            if dry {
                drying::drying_pass_jacobi_rows(g, &c.p, c.evap, c.rewet, c.bypass, r);
            }
        }
        V::K64Par | V::K64Ser => {
            let m = if v == V::K64Par {
                Mode::Parallel
            } else {
                Mode::Serial
            };
            if adv {
                k64::advect_jacobi_rows(g, &c.p, c.gx, c.gy, m);
            }
            if dry {
                k64::drying_pass_jacobi_rows(g, &c.p, c.evap, c.rewet, c.bypass, m);
            }
        }
        V::K32Par | V::K32Ser => {
            let m = if v == V::K32Par {
                Mode::Parallel
            } else {
                Mode::Serial
            };
            if adv {
                k32::advect_jacobi_rows(g, &c.p, c.gx, c.gy, m);
            }
            if dry {
                k32::drying_pass_jacobi_rows(g, &c.p, c.evap, c.rewet, c.bypass, m);
            }
        }
        V::SerV3 => {
            #[cfg(sonda_v3)]
            // SAFETY: só chega aqui com `v3_available()` (as seis features detectadas em execução).
            unsafe {
                step_v3(g, c, adv, dry)
            };
            #[cfg(not(sonda_v3))]
            unreachable!("C.2 exige --cfg sonda_v3");
        }
    }
}

/// `n` passos na cadência do produto: advecção sempre, secagem quando `k % dry_every == 0`.
fn run(v: V, g: &mut Grid, c: &Ctx, n: usize, dry_every: usize) {
    for k in 1..=n {
        step(v, g, c, true, k % dry_every == 0);
    }
}

fn fmax(a: &[f32]) -> f32 {
    a.iter().fold(0.0f32, |m, &v| m.max(v))
}

fn restore(g: &mut Grid, snap: &GridSnapshot, fx: &[f32], fy: &[f32]) {
    restore_grid(g, snap);
    g.flow_x.copy_from_slice(fx);
    g.flow_y.copy_from_slice(fy);
}

fn fnv(g: &Grid) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |v: f32| {
        for b in v.to_bits().to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for a in [&g.film, &g.susp, &g.sett, &g.vel_x, &g.vel_y] {
        a.iter().for_each(|&v| eat(v));
    }
    for a in [&g.susp_rgb, &g.sett_rgb] {
        a.iter().flatten().for_each(|&v| eat(v));
    }
    h
}

fn stats(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

fn dev(a: &Grid, b: &Grid, win: (i32, i32, i32, i32)) {
    let (x0, y0, x1, y1) = win;
    let s = a.s;
    let idx = || (y0..=y1).flat_map(move |y| (x0..=x1).map(move |x| x as usize + y as usize * s));
    let one = |name: &str, fa: &dyn Fn(usize) -> f32, fb: &dyn Fn(usize) -> f32| {
        let (mut mx, mut sum, mut n, mut big, mut diff) = (0.0f64, 0.0f64, 0usize, 0.0f64, 0usize);
        for i in idx() {
            let (va, vb) = (f64::from(fa(i)), f64::from(fb(i)));
            let e = (va - vb).abs();
            mx = mx.max(e);
            sum += e;
            n += 1;
            big = big.max(va.abs());
            diff += usize::from(fa(i).to_bits() != fb(i).to_bits());
        }
        println!(
            "  {name:<10} max|Δ| {mx:.3e}  médio|Δ| {:.3e}  (max|campo| {big:.3e}, células ≠ {:.2} %)",
            sum / n as f64,
            100.0 * diff as f64 / n as f64
        );
    };
    one("film", &|i| a.film[i], &|i| b.film[i]);
    one("susp", &|i| a.susp[i], &|i| b.susp[i]);
    one("sett", &|i| a.sett[i], &|i| b.sett[i]);
    one("vel_x", &|i| a.vel_x[i], &|i| b.vel_x[i]);
    one("vel_y", &|i| a.vel_y[i], &|i| b.vel_y[i]);
    for k in 0..3 {
        one(&format!("susp_rgb{k}"), &|i| a.susp_rgb[i][k], &|i| {
            b.susp_rgb[i][k]
        });
        one(&format!("sett_rgb{k}"), &|i| a.sett_rgb[i][k], &|i| {
            b.sett_rgb[i][k]
        });
    }
}

fn main() {
    let t0 = Instant::now();
    let mut e = scene_big();
    let c = Ctx {
        p: e.sim.gather_params(&e.tuning),
        gx: e.sim.gravity(&e.tuning)[0],
        gy: e.sim.gravity(&e.tuning)[1],
        evap: e.sim.evap_scale * e.sim.gather_params(&e.tuning).k(Knob::Evaporation),
        rewet: e.sim.rewet_base * e.sim.gather_params(&e.tuning).k(Knob::Rewet),
        bypass: e.sim.ext_bypass,
    };
    let e_dry_every = e.sim.dry_every as usize;
    let g = e.active_grid_mut();
    let snap = snapshot_grid(g);
    let (fx, fy) = (g.flow_x.clone(), g.flow_y.clone());
    let win = (g.bx0, g.by0, g.bx1, g.by1);
    let dry_every = e_dry_every;
    println!(
        "no instantâneo: max film {:.3e}  max susp {:.3e}  max sett {:.3e}  dry_every {dry_every}",
        fmax(&g.film),
        fmax(&g.susp),
        fmax(&g.sett)
    );
    println!(
        "cena {}x{}  janela x {}..{} y {}..{}  km_mixing {}  montada em {:.1} s  rayon {} fios",
        g.w,
        g.h,
        win.0,
        win.2,
        win.1,
        win.3,
        c.p.km_mixing,
        t0.elapsed().as_secs_f64(),
        rayon::current_num_threads()
    );

    let mut vs = vec![
        V::OrigPar,
        V::K64Par,
        V::K32Par,
        V::OrigSer,
        V::K64Ser,
        V::K32Ser,
    ];
    if v3_available() {
        vs.push(V::SerV3);
    } else {
        println!("C.2: v3 indisponível (sem --cfg sonda_v3 ou CPU sem as features)");
    }

    // Hashes após N_HASH passos, e o estado final de cada variante para os desvios.
    for &v in &vs {
        restore(g, &snap, &fx, &fy);
        run(v, g, &c, N_HASH, dry_every);
        println!("hash N={N_HASH} {v:?}: {:016x}", fnv(g));
    }

    // C.1 desvio f32 × f64 (rota paralela; a serial dá o mesmo byte por construção).
    for n in N_DEV {
        restore(g, &snap, &fx, &fy);
        run(V::OrigPar, g, &c, n, dry_every);
        let a = g.clone();
        restore(g, &snap, &fx, &fy);
        run(V::K32Par, g, &c, n, dry_every);
        println!("desvio f32 × f64 após N={n} passos (na janela inicial):");
        dev(&a, g, win);
    }

    // Régua intercalada: aquecer 1 bloco cada; ROUNDS rodadas × BLOCK passos, início rodado.
    // Cada bloco: restaura, BLOCK advecções cronometradas; restaura, BLOCK secagens cronometradas
    // (cada secagem sobre o instantâneo restaurado — a poça molhada, nunca a já evaporada).
    let mut ma: Vec<Vec<f64>> = vec![Vec::new(); vs.len()];
    let mut md: Vec<Vec<f64>> = vec![Vec::new(); vs.len()];
    for round in 0..=ROUNDS {
        for k in 0..vs.len() {
            let vi = (k + round) % vs.len();
            restore(g, &snap, &fx, &fy);
            for _ in 0..BLOCK {
                let t = Instant::now();
                step(vs[vi], g, &c, true, false);
                if round > 0 {
                    ma[vi].push(t.elapsed().as_secs_f64() * 1e3);
                }
            }
            for _ in 0..BLOCK {
                restore(g, &snap, &fx, &fy);
                let t = Instant::now();
                step(vs[vi], g, &c, false, true);
                if round > 0 {
                    md[vi].push(t.elapsed().as_secs_f64() * 1e3);
                }
            }
        }
    }
    println!(
        "ms por chamada, {ROUNDS} rodadas × {BLOCK}: advect (mín / mediana) · drying (mín / mediana)"
    );
    for (vi, v) in vs.iter().enumerate() {
        let (a0, a1) = stats(&mut ma[vi]);
        let (d0, d1) = stats(&mut md[vi]);
        println!("  {v:?}: advect {a0:.2} / {a1:.2} · drying {d0:.2} / {d1:.2}");
    }
    println!("total {:.1} s", t0.elapsed().as_secs_f64());
}
