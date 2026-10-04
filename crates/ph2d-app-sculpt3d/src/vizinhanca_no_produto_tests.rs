//! ⭐⭐⭐ **Os efeitos de vizinhança da peça NA PLACA** (`docs/3D/30` §14, W6b) —
//! os gates que pedem a placa: o composto da placa é o da CPU a um degrau, e a
//! saída da PLACA, sozinha, cumpre as leis (nada vaza, a risca alarga, a aresta
//! lê o que o meio lê); e a sonda do preço que fixou o tecto do raio.

use std::time::Instant;

use ph2d_tool_painter::{
    AdjustmentKind, AdjustmentParams, BlendMode, BloomParams, GaussianBlurParams, LayerId,
    SURFACE_RADIUS_MAX, ShadowsHighlightsParams, SharpenParams, SpatialUnits,
};

use crate::composto_na_placa::CompostoNaPlaca;
use crate::pilha_da_peca::PilhaDaPeca;
use crate::vizinhanca_da_peca::tests as fx;
use crate::vizinhanca_da_peca::unidades;

/// Um ajuste de `kind` com `params` no topo.
fn ajuste(
    p: &mut PilhaDaPeca,
    u: SpatialUnits,
    kind: AdjustmentKind,
    params: AdjustmentParams,
) -> LayerId {
    let id = p.novo_ajuste(kind, u).expect("serve na peça");
    p.define_parametros(id, params).expect("parâmetros");
    id
}

/// ⭐ **A pilha de vizinhança**: a risca suave, uma camada `Multiply` translúcida
/// pintada por hash e os QUATRO ajustes de vizinhança, cada um a mexer.
fn pilha_de_vizinhanca(k: u8) -> (ph2d_mesh::Mesh, ph2d_mesh_colors::Tinta, PilhaDaPeca) {
    let (mesh, tinta, mut p, _) = fx::peca(k, fx::risca_suave(0.05, 0.02));
    let n = p.amostras();
    let mult = p.nova_camada("mult").expect("camada");
    p.define_modo(mult, BlendMode::Multiply);
    p.define_opacidade(mult, 0.6);
    let px: Vec<[u8; 4]> = (0..n)
        .map(|i| {
            let h = (i as u32).wrapping_mul(2_654_435_761);
            let h = h ^ (h >> 15);
            [
                (h >> 8) as u8 | 0x80,
                (h >> 16) as u8 | 0x80,
                (h >> 24) as u8 | 0x80,
                h as u8,
            ]
        })
        .collect();
    p.plano_mut(mult).expect("plano").escreve(&px, None);
    let u = unidades(&mesh);
    ajuste(
        &mut p,
        u,
        AdjustmentKind::GaussianBlur,
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 0.06 }),
    );
    ajuste(
        &mut p,
        u,
        AdjustmentKind::Sharpen,
        AdjustmentParams::Sharpen(SharpenParams {
            amount: 0.8,
            radius: 0.03,
            mask_edges: false,
        }),
    );
    ajuste(
        &mut p,
        u,
        AdjustmentKind::Bloom,
        AdjustmentParams::Bloom(BloomParams {
            threshold: 0.5,
            intensity: 0.6,
            radius: 0.08,
            falloff: 0.2,
        }),
    );
    ajuste(
        &mut p,
        u,
        AdjustmentKind::ShadowsHighlights,
        AdjustmentParams::ShadowsHighlights(ShadowsHighlightsParams {
            shadows_amount: 0.3,
            shadows_tonal_width: 0.5,
            shadows_radius: 0.07,
            highlights_amount: 0.2,
            highlights_tonal_width: 0.5,
            highlights_radius: 0.07,
            color_correction: 0.1,
            midtone_contrast: 0.05,
        }),
    );
    p.garante_vizinhanca(&tinta, &mesh);
    assert!(p.sincronizada());
    (mesh, tinta, p)
}

/// Quantos bytes diferem e a maior diferença.
fn compara(cpu: &[u8], placa: &[u8]) -> (usize, u8) {
    cpu.iter()
        .zip(placa)
        .filter(|(a, b)| a != b)
        .fold((0, 0), |(n, m), (a, b)| (n + 1, m.max(a.abs_diff(*b))))
}

/// O contrato placa↔CPU dos efeitos de vizinhança: nenhum byte a mais de UM
/// degrau, e poucos a um degrau (o `f32` da placa contrai somas em `fma` e o da
/// CPU não — um valor na fronteira de um degrau sai `±1`).
const FRACCAO_A_UM_DEGRAU: f64 = 0.01;

/// ⭐⭐⭐⭐ **O composto da placa é o da CPU, a um degrau de sRGB8** — com os
/// quatro ajustes de vizinhança, a `8x`, `16x` e `32x`.
#[test]
#[ignore = "precisa de placa"]
fn a_placa_desfoca_a_peca_como_a_cpu() {
    let gpu = gpu_or_skip!();
    for k in [3u8, 4, 5] {
        let (_, _, mut p) = pilha_de_vizinhanca(k);
        let n = p.amostras();
        let mut placa = CompostoNaPlaca::novo(&gpu);
        placa
            .compoe(&gpu, &mut p)
            .expect("a placa leva a vizinhança da peça");
        let lida = placa.le(&gpu).expect("composto");
        let cpu = p.compor();
        let (difs, pior) = compara(&cpu, &lida[..n * 4]);
        eprintln!(
            "{}x: {difs} de {} bytes a um degrau (pior {pior})",
            1u32 << k,
            n * 4
        );
        assert!(
            pior <= 1 && (difs as f64) <= FRACCAO_A_UM_DEGRAU * (n * 4) as f64,
            "{}x: {difs} bytes de {} diferem da CPU (pior {pior})",
            1u32 << k,
            n * 4
        );
    }
}

/// ⭐⭐⭐⭐ **A saída da PLACA, sozinha, borra na superfície** (o gate absoluto,
/// não só a paridade): longe da risca a peça fica como estava, a risca alarga
/// no mundo, e uma amostra de aresta lê o que o meio de uma face lê.
#[test]
#[ignore = "precisa de placa"]
fn na_placa_o_desfoque_e_na_superficie() {
    let gpu = gpu_or_skip!();
    let (w, raio) = (0.05f32, 0.15f32);
    let sigma = raio / 3.0;
    let (mesh, tinta, mut p, xs) = fx::peca(5, fx::risca_suave(w, 0.02));
    fx::desfoque(&mut p, raio, &tinta, &mesh);
    let mut placa = CompostoNaPlaca::novo(&gpu);
    placa.compoe(&gpu, &mut p).expect("compõe");
    let c = placa.le(&gpu).expect("composto");
    let v = |i: usize| c[i * 4];
    let claro = fx::CLARO[0];
    let mut vaza = 0;
    let (mut soma, mut perto) = (0.0f32, 0);
    for (i, &x) in xs.iter().enumerate() {
        let d = fx::dist(x);
        if d > w + 0.01 + 5.0 * sigma && v(i).abs_diff(claro) > 1 {
            vaza += 1;
        }
        if d > w + 0.01 && d < w + 0.01 + sigma {
            soma += f32::from(v(i));
            perto += 1;
        }
    }
    assert_eq!(vaza, 0, "na placa a cor da risca chegou longe dela");
    assert!(
        soma / (perto as f32) < f32::from(claro) - 40.0,
        "na placa a risca não alargou (média {})",
        soma / perto as f32
    );
    // A aresta lê o meio: o desvio ao meio-mais-perto-em-distância, longe no mundo.
    let fronteira = tinta.topologia().verts() + tinta.topologia().arestas_amostras() as usize;
    let mut meio: Vec<(f32, usize)> = (fronteira..xs.len())
        .map(|i| (fx::dist(xs[i]), i))
        .collect();
    meio.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut pior = 0u8;
    for i in (0..fronteira).filter(|&i| fx::dist(xs[i]) < w + 2.0 * raio) {
        let d = fx::dist(xs[i]);
        let k = meio.partition_point(|m| m.0 < d);
        let longe = |j: usize| {
            let (a, b) = (xs[i], xs[meio[j].1]);
            (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2) > 0.09
        };
        if let Some(j) = (k.saturating_sub(64)..(k + 64).min(meio.len()))
            .filter(|&j| longe(j))
            .min_by(|&a, &b| (meio[a].0 - d).abs().total_cmp(&(meio[b].0 - d).abs()))
            .filter(|&j| (meio[j].0 - d).abs() <= 2e-4)
        {
            pior = pior.max(v(i).abs_diff(v(meio[j].1)));
        }
    }
    assert!(
        pior <= 2,
        "na placa uma amostra de aresta afasta-se do meio em {pior} degraus"
    );
}

/// 🔎 **SONDA — o preço do desfoque NA PLACA** (`docs/3D/30` §7, o critério de
/// desistência da W6: *«blur de raio razoável a `32x` passar de `100 ms` ⇒ vira
/// aplicar»*). A peça da lição, um desfoque gaussiano, o raio em fracções do
/// curso do slider (`SURFACE_RADIUS_MAX` da diagonal) e além dele; o passo =
/// metadado pela porta + composição + `poll` à espera.
#[test]
#[ignore = "sonda: imprime a tabela"]
fn diag_o_preco_do_desfoque_na_placa() {
    let gpu = gpu_or_skip!();
    for k in 3u8..=6 {
        let (mesh, tinta, mut p, _) = fx::peca(k, fx::risca_suave(0.05, 0.02));
        let SpatialUnits::Surface { size } = unidades(&mesh) else {
            panic!("unidades da peça")
        };
        let id = fx::desfoque(&mut p, 0.01, &tinta, &mesh);
        let mut placa = CompostoNaPlaca::novo(&gpu);
        placa.compoe(&gpu, &mut p).expect("compõe");
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        for curso in [0.25f32, 0.5, 1.0, 2.0, 4.0] {
            let raio = curso * SURFACE_RADIUS_MAX * size;
            let grau = p
                .vizinhanca()
                .map_or(0, |v| v.difusao().polinomio(raio / 3.0).len());
            let mut passos = Vec::new();
            for q in 0..8u32 {
                let mut nova = p.pilha().clone();
                if let Some(a) = nova.adjustment_mut(id) {
                    a.params = AdjustmentParams::GaussianBlur(GaussianBlurParams {
                        radius: raio * (1.0 - q as f32 * 0.01),
                    });
                }
                let t = Instant::now();
                p.troca_metadado(nova).expect("metadado");
                placa.compoe(&gpu, &mut p).expect("compõe");
                let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
                passos.push(t.elapsed().as_secs_f64() * 1e3);
            }
            passos.sort_by(f64::total_cmp);
            eprintln!(
                "degrau {k} ({}x) · {} amostras · raio {:.0} % do curso ({raio:.4}) · grau {grau} · \
                 um passo NA PLACA: mediana {:.2} ms · pior {:.2} ms",
                1u32 << k,
                p.amostras(),
                curso * 100.0,
                passos[4],
                passos[7]
            );
        }
    }
}
