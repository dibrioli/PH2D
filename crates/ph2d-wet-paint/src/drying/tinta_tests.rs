//! F4-2 E8: o tempo pela lei da tinta. Doc: `docs/Painter/Arquitetura/E8_tempo.md`.

use super::*;
use crate::grid::tinta::{M_UM, garante};
use crate::painter::Engine;
use crate::par::Rows;
use crate::render::{PigmentVisual, RenderLayer, render_pigment_row_visual};
use crate::sim::{LeiDaTinta, Params};
use ph2d_paint_tinta::{Soma, Tinta};

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

fn motor(cor: [f32; 3]) -> Engine {
    let mut e = Engine::new(W, H);
    e.sim.lei = crate::assenta::Lei::Cobre;
    e.sliders.water = 1.0;
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    e.color = cor.map(f64::from);
    e
}

fn params(e: &Engine) -> Params {
    e.sim.gather_params(&e.tuning)
}

/// A imagem RGBA da grade inteira pela lei da tinta, SEM papel nem relevo.
fn imagem(g: &Grid, p: &Params) -> Vec<u8> {
    let camadas = [RenderLayer {
        grid: g,
        opacity: 1.0,
        visible: true,
    }];
    let mut out = vec![0u8; g.w * g.h * 4];
    for (y, linha) in out.chunks_mut(g.w * 4).enumerate() {
        render_pigment_row_visual(
            Some(p),
            &camadas,
            PigmentVisual::default(),
            y + 1,
            1,
            g.w,
            linha,
        );
    }
    out
}

/// A mesma grade vista SECA (o «molhado escurece» é outra lei, `s_fator(m, film > 0)`: tirá-lo isola o assentar).
fn seca_a_vista(g: &Grid) -> Grid {
    let mut v = g.clone();
    v.film.fill(0.0);
    v
}

/// (pior diferença por canal, quantos canais diferem).
fn diferenca(a: &[u8], b: &[u8]) -> (u8, usize) {
    a.iter().zip(b).fold((0, 0), |(m, n), (&x, &y)| {
        let d = x.abs_diff(y);
        (m.max(d), n + usize::from(d > 0))
    })
}

/// A rota Jacobi do produto até a água acabar (o laço do `fast_dry`, com a rota escolhida).
fn seca_jacobi(g: &mut Grid, p: &Params, rows: Rows) {
    for _ in 0..40 {
        if !g.has_fluid {
            break;
        }
        g.film.iter_mut().for_each(|w| *w *= 0.5);
        crate::drying::drying_pass_jacobi_rows(g, p, 1000.0, 0.0001, false, rows);
        crate::solver::rebuild_active_region(g);
    }
}

fn volume(g: &Grid) -> f64 {
    g.tinta
        .iter()
        .map(|c| f64::from(c.filme.v) + f64::from(c.aberto.v) + f64::from(c.altura_fechada))
        .sum()
}

/// A pior violação relativa do invariante `filme.v·M_UM == susp`, `aberto.v·M_UM == sett` (somada na grade).
fn invariante(g: &Grid) -> (f64, f64) {
    let (mut f, mut s, mut a, mut t) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for i in 0..g.cells {
        f += f64::from(g.tinta[i].filme.v) * f64::from(M_UM);
        a += f64::from(g.tinta[i].aberto.v) * f64::from(M_UM);
        s += f64::from(g.susp[i]);
        t += f64::from(g.sett[i]);
    }
    let rel = |x: f64, y: f64| (x - y).abs() / x.abs().max(y.abs()).max(1.0);
    (rel(f, s), rel(a, t))
}

#[test]
fn assentar_nao_muda_o_ecra_ao_bit_no_motor() {
    let mut e = motor([25.0, 76.0, 204.0]);
    traco(&mut e, 10.0, 80.0, 30.0);
    traco(&mut e, 30.0, 60.0, 34.0);
    let p = params(&e);
    let g0 = e.layers[0].grid.clone();
    assert!(g0.susp.iter().any(|&m| m > 0.0), "há filme por assentar");
    let antes = imagem(&seca_a_vista(&g0), &p);
    assert!(antes.chunks(4).any(|px| px[3] > 0), "o traço vê-se");

    // As três rotas: Gauss-Seidel (`fast_dry`), Jacobi serial e paralela, e o «Dry canvas».
    e.fast_dry_now();
    let gs = imagem(&e.layers[0].grid, &p);
    let mut js = g0.clone();
    seca_jacobi(&mut js, &p, Rows::Serial);
    let mut jp = g0.clone();
    seca_jacobi(&mut jp, &p, Rows::Parallel);
    assert!(
        js.sett == jp.sett && js.tinta == jp.tinta,
        "Jacobi serial = paralelo ao bit"
    );
    let mut dc = g0.clone();
    crate::grid::dry_canvas(&mut dc, p.mix, p.glaze, p.ecra);
    for (nome, g) in [
        ("gs", &e.layers[0].grid),
        ("jacobi", &js),
        ("dry_canvas", &dc),
    ] {
        assert!(
            g.susp.iter().all(|&m| m == 0.0) || nome == "gs",
            "{nome}: assentou tudo"
        );
        let depois = if nome == "gs" {
            gs.clone()
        } else {
            imagem(g, &p)
        };
        let (pior, n) = diferenca(&antes, &depois);
        println!(
            "[e8] {nome}: pior Δ = {pior}/255 em {n} canais de {}",
            antes.len()
        );
        // Tolerância: 1/255 — as somas movem-se em f32 (`transfere`, ≤ 4 ulp) e o pixel é um `u8` arredondado.
        assert!(pior <= 1, "{nome}: o ecrã mudou {pior}/255 ao assentar");
        let (fi, ab) = invariante(g);
        println!("[e8] {nome}: invariante filme {fi:.2e} aberto {ab:.2e}");
    }
}

/// Uma linha de células SECAS só com aberto, de óleo (`m = 1`), com a massa coerente.
fn grade_de_oleo(cores: &[([f32; 3], f32, f32)]) -> Grid {
    let mut g = Grid::new(cores.len(), 1);
    let s = g.s;
    let t = garante(&mut g);
    for (k, &(rgb, transp, v)) in cores.iter().enumerate() {
        let c = &mut t[k + 1 + s];
        c.aberto = Soma::de(Tinta::de_srgb(rgb, transp), v);
        c.m = 1.0;
    }
    for k in 0..cores.len() {
        let i = k + 1 + s;
        g.sett[i] = g.tinta[i].aberto.v * M_UM;
    }
    g
}

fn sobre_branco(px: &[u8]) -> [f64; 3] {
    let a = f64::from(px[3]) / 255.0;
    [0, 1, 2].map(|k| f64::from(px[k]) * a + 255.0 * (1.0 - a))
}

#[test]
fn fechar_e_o_que_se_via() {
    let e = Engine::new(8, 8);
    let mut p = params(&e);
    p.tinta = LeiDaTinta::Pigmento;
    // Opacas saturadas, uma transparente e o branco opaco.
    let mut g = grade_de_oleo(&[
        ([0.1, 0.2, 0.7], 0.0, 2.0),
        ([0.9, 0.2, 0.1], 0.0, 0.3),
        ([0.95, 0.8, 0.1], 0.9, 1.5),
        ([0.2, 0.6, 0.3], 0.5, 0.05),
        ([1.0, 1.0, 1.0], 0.0, 0.5),
    ]);
    let antes = imagem(&g, &p);
    cura(&mut g, 1.0e9);
    assert!(g.tinta.iter().all(|c| c.aberto.v == 0.0), "tudo fechou");
    let depois = imagem(&g, &p);
    for k in 0..5 {
        let (a, d) = (&antes[k * 4..k * 4 + 4], &depois[k * 4..k * 4 + 4]);
        let (wa, wd) = (sobre_branco(a), sobre_branco(d));
        let dw = (0..3).map(|j| (wa[j] - wd[j]).abs()).fold(0.0, f64::max);
        println!("[e8] célula {k}: antes {a:?} depois {d:?} · sobre branco Δ {dw:.3}");
        // O fundo fecha sobre o branco E sobre o preto (`r_g`, `r_g0`): cor e cobertura são as de antes, em TODAS
        // (a opaca espessa e o branco tinham `α` pela cobertura, não por `1 − R_w`).
        assert!(dw <= 1.5, "célula {k}: sobre branco mudou {dw}");
        assert!(
            (0..4).all(|j| a[j].abs_diff(d[j]) <= 1),
            "célula {k}: o pixel mudou no fecho"
        );
    }
}

#[test]
fn a_secagem_conserva() {
    let mut e = motor([204.0, 51.0, 25.0]);
    traco(&mut e, 10.0, 80.0, 30.0);
    let p = params(&e);
    let mut g = e.layers[0].grid.clone();
    for c in g.tinta.iter_mut() {
        c.m = 1.0;
    }
    let (fi0, ab0) = invariante(&g);
    let v0 = volume(&g);
    seca_jacobi(&mut g, &p, Rows::Parallel);
    let v1 = volume(&g);
    let (fi1, ab1) = invariante(&g);
    cura(&mut g, 1.0e9);
    let v2 = volume(&g);
    let (fi2, ab2) = invariante(&g);
    let fechadas = g.tinta.iter().filter(|c| c.altura_fechada > 0.0).count();
    println!(
        "[e8] Σv: {v0:.6} → seco {v1:.6} → fechado {v2:.6} ({fechadas} células fecharam) · invariante \
         {fi0:.1e}/{ab0:.1e} → {fi1:.1e}/{ab1:.1e} → {fi2:.1e}/{ab2:.1e}"
    );
    assert!(v0 > 0.0 && fechadas > 0);
    assert!((v1 - v0).abs() / v0 < 1e-5, "secar perdeu volume");
    assert!((v2 - v0).abs() / v0 < 1e-5, "fechar perdeu volume");
    assert!(
        ab1 <= ab0 + 1e-5 && fi1 <= fi0 + 1e-5,
        "o invariante piorou ao secar"
    );
    assert!(ab2 <= ab0 + 1e-5, "o invariante piorou ao fechar");
}

#[test]
fn a_altura_do_oleo_nao_some_no_fecho() {
    let mut g = grade_de_oleo(&[([0.4, 0.3, 0.2], 0.0, 3.0), ([0.9, 0.9, 0.8], 0.2, 0.7)]);
    let s = g.s;
    let antes: Vec<f64> = (0..2)
        .map(|k| crate::render::tinta::massa_da_altura(&g, k + 1 + s))
        .collect();
    cura(&mut g, 1.0e9);
    for k in 0..2 {
        let i = k + 1 + s;
        let depois = crate::render::tinta::massa_da_altura(&g, i);
        println!(
            "[e8] altura {k}: {:.4} → {depois:.4} (sett {})",
            antes[k], g.sett[i]
        );
        assert_eq!(g.sett[i], 0.0, "a massa saiu de sett");
        assert!(
            (depois - antes[k]).abs() / antes[k] < 1e-6,
            "a altura mudou"
        );
    }
}

#[test]
fn a_aguarela_nao_fecha() {
    let mut g = grade_de_oleo(&[([0.1, 0.2, 0.7], 0.0, 2.0)]);
    g.tinta.iter_mut().for_each(|c| c.m = 0.0);
    cura(&mut g, 1.0e9);
    assert!(g.tinta.iter().any(|c| c.aberto.v > 0.0 && c.o == 1.0));
}
