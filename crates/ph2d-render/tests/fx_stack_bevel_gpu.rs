//! **O BEVEL sobre uma aresta OBLÍQUA ANTIALIASADA** — o gate que faltava, e o que o defeito
//! reportado exigia.
//!
//! ⚠️ **O gate irmão media zero POR CONSTRUÇÃO.** Ele sonda quatro texels numa caixa axis-aligned
//! com luz `(2,5)/√29` — normal RACIONAL —, e sob uma normal racional todo texel à mesma distância
//! da aresta é translação de rede de outro: a fase da rasterização é idêntica em todos, e o pente
//! não tem por onde aparecer. Um artefato de dezenas de níveis atravessou 13 gates verdes assim.
//!
//! O que se afirma aqui é a propriedade que o bevel É: **numa aresta RETA a normal é constante,
//! logo `N·L` é constante, logo o sombreado a distância fixa tem de ser o MESMO ao longo dela.**
//! Não se mede um valor — mede-se a AUSÊNCIA de variação onde a geometria proíbe variação.
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_bevel_gpu -- --ignored`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{
    oblique_segments, oblique_signed, oblique_source, readback, try_headless_gpu,
};

const W: u32 = 96;
const H: u32 = 96;
/// A banda do relevo, em texels.
const BAND: f32 = 12.0;

fn beveled(gpu: &ph2d_gpu::GpuContext, geom: &[[f32; 4]]) -> Vec<u8> {
    let src = oblique_source(gpu, W, H);
    let dst = make_output_texture(gpu, W, H);
    let mut pass = FxStackPass::new(gpu);
    pass.run(
        gpu,
        &src,
        &dst,
        W,
        H,
        &[FxOpGpu {
            kind: FxOp::BEVEL,
            sigma_px: BAND,
            offset_px: [-8, 8],
            tint: [0.0, 0.0, 0.0, 1.0],
            opacity: 1.0,
            mode: 0,
        }],
        geom,
    );
    readback(gpu, &dst, W, H)
}

/// O espalhamento do sombreado ao longo da aresta, a `dist` texels PARA DENTRO.
fn ripple(px: &[u8], dist: f64) -> f64 {
    let mut vals = Vec::new();
    for y in 3..H - 3 {
        for x in 3..W - 3 {
            let d = oblique_signed(x, y);
            if (d - dist).abs() > 0.06 {
                continue;
            }
            let o = ((y * W + x) * 4) as usize;
            // Luminância — o bevel tinge de branco ou de preto, então é ela que muda.
            vals.push(
                0.299 * f64::from(px[o])
                    + 0.587 * f64::from(px[o + 1])
                    + 0.114 * f64::from(px[o + 2]),
            );
        }
    }
    if vals.len() < 8 {
        return -1.0;
    }
    vals.iter().copied().fold(f64::MIN, f64::max) - vals.iter().copied().fold(f64::MAX, f64::min)
}

/// **Numa aresta reta o relevo é CONSTANTE ao longo dela.**
///
/// Com o pé exato vindo da geometria, `off` é perpendicular à silhueta por definição de ponto mais
/// próximo — a normal não se estima, ela É o vetor. Sem geometria, a fronteira é estimada da
/// COBERTURA por um estêncil de 2 texels sobre uma rampa de 1,0–1,41, e o erro é função da fase da
/// escada: o mesmo sombreado varre dezenas de níveis ao longo de uma aresta que não muda de
/// direção.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_relief_is_constant_along_a_straight_edge() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_bevel] sem adapter — skip");
        return;
    };
    let px = beveled(&gpu, &oblique_segments(W));
    for d in [3.0, 5.0, 8.0] {
        let r = ripple(&px, d);
        assert!(r >= 0.0, "poucas amostras a d={d}");
        assert!(
            r <= 6.0,
            "o relevo varia {r:.1} níveis ao longo de uma aresta RETA a {d} texels de \
             profundidade — a normal está sendo estimada, não medida"
        );
    }
}

/// **Os DOIS caminhos desenham relevo de verdade.**
///
/// ⚠️ **Este gate nasceu cobrindo só o caminho do raster, e uma mutação o pegou:** desligar o pé
/// por texel (`far = true`) faz o braço da geometria não desenhar NADA, e um bevel que não desenha
/// nada tem ondulação ZERO — ele passava pelo gate acima como se fosse a cura. Uma afirmação de
/// AUSÊNCIA (não ondula) só vale acompanhada da de PRESENÇA, e a de presença tem de cobrir o mesmo
/// caminho.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn both_paths_actually_light_the_rim() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_bevel] sem adapter — skip");
        return;
    };
    for (name, geom) in [
        ("com geometria", oblique_segments(W)),
        ("sem geometria", Vec::new()),
    ] {
        let px = beveled(&gpu, &geom);
        let mut lit = 0u32;
        for y in 3..H - 3 {
            for x in 3..W - 3 {
                let d = oblique_signed(x, y);
                if !(1.0..=f64::from(BAND)).contains(&d) {
                    continue;
                }
                let o = ((y * W + x) * 4) as usize;
                let l = 0.299 * f64::from(px[o])
                    + 0.587 * f64::from(px[o + 1])
                    + 0.114 * f64::from(px[o + 2]);
                // A tinta crua tem luminância ~179,8; o relevo a afasta disso nos dois sentidos.
                if (l - 179.8).abs() > 12.0 {
                    lit += 1;
                }
            }
        }
        assert!(
            lit > 200,
            "{name}: só {lit} texels da banda foram sombreados — não há relevo nenhum"
        );
    }
}
