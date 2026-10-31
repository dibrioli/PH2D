//! F4-2 E2/E3: o interruptor [`LeiDaTinta`] e os campos da lei nova ([`super::CelulaTinta`]).

use super::*;
use crate::grid::{
    Registo, restore_grid, restore_grid_region, snapshot_grid, snapshot_grid_region,
};
use crate::painter::Engine;
use crate::render::{RenderLayer, render_pigment_only};
use crate::sim::{LeiDaTinta, sim_step, sim_step_stage};

const W: usize = 96;
const H: usize = 64;

fn traco(e: &mut Engine, x0: f64, x1: f64, y: f64) {
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 6.0f64).min(x1);
        e.dispatch_pressure_dab(x, y, 6.0, 40.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

/// Dois traços molhados e `passos` passos da água, com a lei `antes` no 1.º traço e `depois` no resto.
fn sessao(antes: LeiDaTinta, depois: LeiDaTinta, passos: usize) -> Engine {
    let mut e = Engine::new(W, H);
    e.sim.lei = crate::assenta::Lei::Cobre;
    e.sliders.water = 1.0;
    e.sim.set_tinta(antes);
    traco(&mut e, 10.0, 80.0, 20.0);
    for _ in 0..passos / 2 {
        e.step_simulation();
    }
    e.sim.set_tinta(depois);
    e.color = [0.8, 0.2, 0.1];
    traco(&mut e, 15.0, 70.0, 26.0);
    for _ in 0..passos - passos / 2 {
        e.step_simulation();
    }
    e
}

fn pixeis(e: &Engine) -> Vec<u8> {
    let layers: Vec<RenderLayer<'_>> = e
        .layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let mut out = vec![0u8; W * H * 4];
    render_pigment_only(&layers, &mut out);
    out
}

fn planos_velhos(g: &Grid) -> Vec<u32> {
    let mut v = Vec::new();
    for p in [&g.film, &g.susp, &g.sett, &g.vel_x, &g.vel_y] {
        v.extend(p.iter().map(|x| x.to_bits()));
    }
    for p in [&g.susp_rgb, &g.sett_rgb] {
        v.extend(p.iter().flatten().map(|x| x.to_bits()));
    }
    v
}

#[test]
fn o_interruptor_da_tinta_nao_muda_um_pixel_com_a_lei_velha() {
    use LeiDaTinta::{Pigmento, Velha};
    let base = sessao(Velha, Velha, 40);
    assert!(pixeis(&base).iter().any(|&b| b != 0), "a cena pinta");
    assert_eq!(pixeis(&sessao(Velha, Velha, 40)), pixeis(&base));
    // E6a: com `Pigmento` o depósito escreve as somas e espelha a cor em `susp_rgb`/`sett_rgb` (sem `mix.mix`) —
    // a cor e os píxeis mudam de propósito; a MASSA e a água seguem a lei velha (sem `WetCover`/`DryCover` na cena).
    for (a, d) in [(Pigmento, Pigmento), (Pigmento, Velha), (Velha, Pigmento)] {
        let e = sessao(a, d, 40);
        assert_eq!(
            massas(&e.layers[0].grid),
            massas(&base.layers[0].grid),
            "{a:?}→{d:?}: a massa mudou"
        );
    }
}

fn massas(g: &Grid) -> Vec<u32> {
    let mut v = Vec::new();
    for p in [&g.film, &g.susp, &g.sett, &g.vel_x, &g.vel_y] {
        v.extend(p.iter().map(|x| x.to_bits()));
    }
    v
}

#[test]
fn os_campos_da_tinta_nova_nao_existem_na_lei_velha() {
    let e = sessao(LeiDaTinta::Velha, LeiDaTinta::Velha, 20);
    assert!(
        e.layers[0].grid.tinta.is_empty(),
        "a lei velha não aloca a tinta nova"
    );
    assert!(e.sim.gather_params(&e.tuning).tinta == LeiDaTinta::Velha);
    let e = sessao(LeiDaTinta::Velha, LeiDaTinta::Pigmento, 20);
    let g = &e.layers[0].grid;
    assert_eq!(
        g.tinta.len(),
        g.cells,
        "com Pigmento o passo aloca a grade inteira"
    );
    // E6a: o campo nasce MIGRADO da tinta já pousada (não NEUTRO): trocar de lei não apaga a tela.
    assert!(g.tinta.iter().any(|c| c.filme.v > 0.0 || c.aberto.v > 0.0));
}

/// Uma tinta reconhecível por célula (ninguém a escreve ainda: E4–E8).
fn marca(g: &mut Grid, k: f32) {
    for (i, c) in garante(g).iter_mut().enumerate() {
        let x = i as f32 * 1e-3 + k;
        c.filme = ph2d_paint_tinta::Soma {
            v: x,
            lab: [x, -x, 0.5 * x],
            t: 0.25 * x,
        };
        c.aberto.v = 2.0 * x;
        c.r_g = [0.9, 0.8, x];
        c.m = 0.5;
        c.o = 0.75;
        c.altura_fechada = x;
    }
}

#[test]
fn a_retoma_e_o_desfazer_levam_os_campos_da_tinta_nova() {
    let mut e = sessao(LeiDaTinta::Pigmento, LeiDaTinta::Pigmento, 10);
    let g = &mut e.layers[0].grid;
    marca(g, 1.0);
    let antes = g.tinta.clone();

    // Instantâneo de folha inteira.
    let foto = snapshot_grid(g);
    marca(g, 7.0);
    restore_grid(g, &foto);
    assert_eq!(g.tinta, antes, "restore_grid");

    // Registo do histórico por tiles (o desfazer do motor).
    let mut r = Registo::abre(g);
    r.marca(g, 10, 10, 40, 30);
    marca(g, 3.0);
    let inv = Registo::inverso(g, &r, None);
    r.restaura(g, None);
    let s = g.s;
    for y in 10..30 {
        for x in 10..40 {
            assert_eq!(g.tinta[y * s + x], antes[y * s + x], "desfazer ({x},{y})");
        }
    }
    inv.restaura(g, None);
    assert_eq!(
        g.tinta[20 * s + 20].altura_fechada,
        (20 * s + 20) as f32 * 1e-3 + 3.0,
        "refazer"
    );
    // Registo aberto numa grade SEM o campo: desfazer devolve-a sem ele.
    let mut sem = Grid::new(W, H);
    let mut r2 = Registo::abre(&sem);
    r2.marca_tudo(&sem);
    marca(&mut sem, 1.0);
    r2.restaura(&mut sem, None);
    assert!(sem.tinta.is_empty(), "a base sem tinta volta sem tinta");

    // Recorte regional.
    marca(g, 1.0);
    let p = snapshot_grid_region(g, 5, 5, 30, 20).expect("recorte");
    marca(g, 9.0);
    restore_grid_region(g, &p);
    assert_eq!(g.tinta[12 * s + 12], antes[12 * s + 12], "recorte");

    // Clone (instantâneo da macro) e a retoma por estágios = o passo atómico.
    marca(g, 1.0);
    let copia = e.clone();
    assert_eq!(copia.layers[0].grid.tinta, e.layers[0].grid.tinta, "clone");
    assert!(e.layers[0].grid.bytes_retidos() >= e.layers[0].grid.cells * 64);
    let (mut sa, mut ga) = (e.sim.clone(), e.layers[0].grid.clone());
    let (mut sb, mut gb) = (e.sim.clone(), e.layers[0].grid.clone());
    ga.has_fluid = true;
    gb.has_fluid = true;
    for _ in 0..7 {
        sim_step(&mut sa, &mut ga, &e.tuning);
        while sim_step_stage(&mut sb, &mut gb, &e.tuning).is_none() {}
    }
    assert_eq!(ga.tinta, gb.tinta, "retoma");
    assert_eq!(planos_velhos(&ga), planos_velhos(&gb), "retoma (velhos)");
}

fn rss_kib() -> u64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    s.lines()
        .find_map(|l| l.strip_prefix("VmRSS:"))
        .and_then(|v| v.trim().trim_end_matches("kB").trim().parse().ok())
        .unwrap_or(0)
}

/// A régua do F4-2 §2.4: o RSS de uma tela 2048² com um traço, lei velha × a mesma depois de um passo com `Pigmento`.
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --profile smoke --lib medida_rss_da_tinta -- --ignored --nocapture`
#[test]
#[ignore = "medida — corre-se à mão"]
fn medida_rss_da_tinta() {
    let n = 2048;
    let r0 = rss_kib();
    let mut e = Engine::new(n, n);
    e.sliders.water = 1.0;
    traco(&mut e, 100.0, 1900.0, 1000.0);
    for _ in 0..4 {
        e.step_simulation();
    }
    let r1 = rss_kib();
    let g = &e.layers[0].grid;
    println!(
        "[rss] velha: RSS {} MiB (Δ desde o início {} MiB) · grade {} MiB · tinta {} B",
        r1 / 1024,
        (r1 - r0) / 1024,
        g.bytes_retidos() >> 20,
        g.tinta.len() * std::mem::size_of::<CelulaTinta>()
    );
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    for _ in 0..4 {
        e.step_simulation();
    }
    let r2 = rss_kib();
    let g = &e.layers[0].grid;
    println!(
        "[rss] pigmento: RSS {} MiB (Δ sobre a velha {} MiB) · grade {} MiB · tinta {} MiB ({} B/célula)",
        r2 / 1024,
        (r2 - r1) / 1024,
        g.bytes_retidos() >> 20,
        (g.tinta.len() * std::mem::size_of::<CelulaTinta>()) >> 20,
        std::mem::size_of::<CelulaTinta>()
    );
    assert_eq!(std::mem::size_of::<CelulaTinta>(), 64);
}
