//! F4-2 E5: o ecrã da célula pela lei da tinta (`docs/Painter/Arquitetura/E5_ecra.md`).

use super::*;
use crate::grid::tinta::garante;
use crate::jsmath::clamp_u8;
use crate::painter::Engine;
use crate::render::{PigmentVisual, RenderLayer, render_pigment_row_visual};
use crate::sim::{LeiDaTinta, Params};
use ph2d_paint_tinta::oklab::srgb_para_linear;
use ph2d_paint_tinta::{Soma, Tinta, camada};

const W: usize = 6;
const H: usize = 1;

fn params() -> Params {
    let e = Engine::new(8, 8);
    let mut p = e.sim.gather_params(&e.tuning);
    p.tinta = LeiDaTinta::Pigmento;
    p
}

/// Uma linha de células com somas conhecidas (e uma massa qualquer: a cor não a lê). A célula 6 fica NEUTRA.
fn grade() -> Grid {
    let mut g = Grid::new(W, H);
    let azul = Tinta::de_srgb([0.1, 0.2, 0.7], 0.0);
    let amarelo = Tinta::de_srgb([0.95, 0.8, 0.1], 0.9);
    let celulas = [
        (Soma::de(azul, 3.0), Soma::default(), [1.0; 3], 0.0),
        (Soma::default(), Soma::de(amarelo, 0.4), [1.0; 3], 0.0),
        (
            Soma::de(azul, 0.3),
            Soma::de(amarelo, 1.5),
            [0.9, 0.7, 0.6],
            0.0,
        ),
        (Soma::de(amarelo, 0.05), Soma::default(), [1.0; 3], 5.0),
        (
            Soma::de(azul, 0.8),
            Soma::de(azul, 0.8),
            [0.4, 0.5, 0.9],
            2.0,
        ),
    ];
    let s = g.s;
    for (k, &(_, _, _, w)) in celulas.iter().enumerate() {
        let i = (k + 1) + s;
        g.susp[i] = 1234.0 * (k as f32 + 1.0);
        g.sett[i] = 777.0;
        g.film[i] = w;
    }
    let t = garante(&mut g);
    for (k, &(f, a, rg, _)) in celulas.iter().enumerate() {
        let c = &mut t[(k + 1) + s];
        c.filme = f;
        c.aberto = a;
        c.r_g = rg;
    }
    // Depois do `garante` (que migra a massa velha): massa sem somas, que a lei velha pintaria.
    g.susp[W + s] = 3000.0;
    g
}

fn linha(g: &Grid, p: &Params) -> Vec<u8> {
    let camadas = [RenderLayer {
        grid: g,
        opacity: 1.0,
        visible: true,
    }];
    let mut out = vec![0u8; W * 4];
    render_pigment_row_visual(
        Some(p),
        &camadas,
        PigmentVisual::default(),
        1,
        1,
        W,
        &mut out,
    );
    out
}

/// O ecrã da lei para a célula `i`, pelas funções da crate da tinta (a régua).
fn esperado(g: &Grid, i: usize) -> ([f32; 3], f32, [f32; 3]) {
    let c = tinta::em(g, i);
    let o = optica_celula(&c.filme, &c.aberto, s_fator(c.m, g.film[i] > 0.0));
    let (cor, a) = ecra_sobre_branco(&o, &Bandas(c.r_g));
    (cor.0, a, camada(&o, &Bandas(c.r_g)).0)
}

#[test]
fn o_ecra_da_celula_e_a_lei_da_tinta_ao_bit() {
    let g = grade();
    let p = params();
    let px = linha(&g, &p);
    for k in 0..W - 1 {
        let i = (k + 1) + g.s;
        let (cor, a, _) = esperado(&g, i);
        assert!(a > 0.0, "célula {k} tem tinta");
        let srgb = cor.map(|v| f64::from(linear_para_srgb(v)) * 255.0);
        let (c, la) = ecra_da_celula(&g, i).expect("tinta");
        assert_eq!(c.map(f64::to_bits), srgb.map(f64::to_bits), "célula {k}");
        assert_eq!(la.to_bits(), f64::from(a).to_bits(), "célula {k}");
        let q = &px[k * 4..k * 4 + 4];
        assert_eq!(
            q,
            [
                clamp_u8(srgb[0]),
                clamp_u8(srgb[1]),
                clamp_u8(srgb[2]),
                clamp_u8(f64::from(a) * 255.0)
            ],
            "pixel {k}"
        );
    }
}

#[test]
fn celula_vazia_e_transparente() {
    let g = grade();
    let px = linha(&g, &params());
    assert_eq!(&px[(W - 1) * 4..W * 4], [0, 0, 0, 0], "massa sem tinta");
    assert!(ecra_da_celula(&g, W + g.s).is_none());
    // Sem o campo, toda a grade é NEUTRA.
    let mut v = Grid::new(W, H);
    v.susp[1 + v.s] = 3000.0;
    assert!(linha(&v, &params()).iter().all(|&b| b == 0));
}

#[test]
fn sobre_branco_o_pixel_composto_devolve_r_w() {
    let g = grade();
    let px = linha(&g, &params());
    let mut pior = 0.0f32;
    for k in 0..W - 1 {
        let (_, _, rw) = esperado(&g, (k + 1) + g.s);
        let q = &px[k * 4..k * 4 + 4];
        let a = f32::from(q[3]) / 255.0;
        for b in 0..3 {
            let c = srgb_para_linear(f32::from(q[b]) / 255.0);
            let sobre_branco = linear_para_srgb(c * a + (1.0 - a));
            pior = pior.max((sobre_branco - linear_para_srgb(rw[b])).abs());
        }
    }
    assert!(
        pior <= 1.0 / 255.0,
        "erro sobre branco {:.3}/255",
        pior * 255.0
    );
}
