//! **O FEATHER sobre uma borda que tem COBERTURA PARCIAL** — o gate que faltava.
//!
//! Irmão do `fx_stack_kinds_gpu.rs` (que está no teto de LOC) e coeso por assunto: aqui a fixture é
//! uma aresta OBLÍQUA ANTIALIASADA, e é isso que separa este arquivo do outro.
//!
//! ⚠️ **Toda fixture do feather no arquivo irmão é um retângulo axis-aligned de alfa 0/255.** Sem
//! cobertura parcial `a_fonte ≡ 1`, e a lei antiga — `outc = mix(over, base * f, opacity)` com
//! `base` PREMULTIPLICADO, ou seja alfa de saída `a_fonte · f` em vez de `f` — é **invisível por
//! construção**. Os 13 gates de lá passavam antes e depois da correção; o defeito que o smoke
//! reportou (uma linha tracejada ao longo do contorno) vivia inteiro na fileira que aquelas
//! fixtures não tinham.
//!
//! ⚠️ **O ângulo é irracional de propósito.** Numa normal racional — o gate do bevel usa
//! `(2,5)/√29` — todo texel à mesma distância da aresta é translação de rede de outro, então a
//! fase da rasterização é a MESMA em todos e o artefato mede zero **por construção**.
//!
//! ⚠️ **A fixture vem do `fx_stack_common`, e ISSO É A CORREÇÃO DE UM DEFEITO REAL.** Este arquivo
//! carregava uma cópia própria dela — mesmos números, outro `fn` — e quando a partilhada aprendeu
//! que o Vello premultiplica em LUZ LINEAR, a cópia ficou para trás a montar `byte · cobertura`.
//! O gate ficou vermelho acusando **149 níveis** de desvio de cor sobre um produto correto: uma
//! fonte que o produto nunca produz responde perguntas sobre outra coisa. Duas portas para
//! *"como é uma aresta antialiasada?"* divergem, e divergiram.
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_feather_gpu -- --ignored`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{INK, oblique_signed as signed, oblique_source, readback, try_headless_gpu};

const W: u32 = 96;
const H: u32 = 96;
/// A largura da banda do feather, em texels.
const BAND: f32 = 8.0;

fn feathered(gpu: &ph2d_gpu::GpuContext) -> Vec<u8> {
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
            kind: FxOp::FEATHER,
            sigma_px: BAND,
            offset_px: [0, 0],
            tint: [1.0, 1.0, 1.0, 1.0],
            opacity: 1.0,
            mode: 0,
            blend: 0,
            noise_scale_px: 0.0,
            detail: 1,
            seed: 0,
            grow_px: 0.0,
        }],
        &[],
    );
    readback(gpu, &dst, W, H)
}

/// **A COR RETA é a da forma em TODA a banda — o feather move o alfa, nunca a cor.**
///
/// É a lei que as três implementações canônicas afirmam (o feather do GIMP é um blur da MÁSCARA, o
/// do Krita é uma gaussiana só no canal alfa, e nos layer styles a cor entra depois como fill), e é
/// o oráculo que pega os DOIS modos de falha renderizados: um texel sem cor sai como FURO (alfa 0
/// cercado de forma — 459 deles na sonda) e um texel com cor preta e alfa sai como DENTE escuro.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_feathered_band_keeps_the_shapes_colour_with_no_hole_and_no_tooth() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_feather] sem adapter — skip");
        return;
    };
    let px = feathered(&gpu);
    let mut worst = 0.0_f64;
    let mut worst_at = (0u32, 0u32);
    let mut seen = 0u32;
    for y in 2..H - 2 {
        for x in 2..W - 2 {
            let d = signed(x, y);
            // A banda do feather, com folga das bordas da textura.
            if d.abs() > f64::from(BAND) * 0.5 {
                continue;
            }
            let o = ((y * W + x) * 4) as usize;
            let a = f64::from(px[o + 3]) / 255.0;
            // Abaixo de ~3% de alfa a cor reta é ruído de quantização, não uma afirmação.
            if a < 0.03 {
                continue;
            }
            seen += 1;
            for c in 0..3 {
                // ⚠️ A saída do passe é RGBA **RETO** — o `cs_resolve` já divide pelo alfa. Dividir
                // aqui de novo mediria a cor duas vezes des-premultiplicada (e foi o que a 1ª
                // versão deste gate fez, acusando 7255 níveis sobre um pixel `[235,175,60,8]`
                // perfeito).
                let straight = f64::from(px[o + c]);
                let err = (straight - INK[c]).abs();
                if err > worst {
                    worst = err;
                    worst_at = (x, y);
                }
            }
        }
    }
    assert!(seen > 200, "a banda não foi amostrada: {seen} texels");
    let (wx, wy) = worst_at;
    let wo = ((wy * W + wx) * 4) as usize;
    let raw = &px[wo..wo + 4];
    assert!(
        worst <= 14.0,
        "a cor RETA saiu da banda em {worst:.1} níveis (pior em {worst_at:?}, rgba cru {raw:?}, \
         d={:.2}, {seen} texels): um FURO ou um DENTE — o feather escreveu cor onde devia \
         escrever só alfa",
        signed(wx, wy)
    );
}

/// **Na fronteira o alfa é a RAMPA, não a rampa vezes a cobertura da fonte.**
///
/// A dupla contagem só é visível onde `a_fonte` é parcial: com a cobertura ~0,5 do texel do
/// contorno, a lei antiga entregava ~0,25 onde a rampa pede ~0,50. É um gate de NÚMERO, e é ele que
/// nomeia o mecanismo — o gate irmão acima mede a consequência renderizada.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_alpha_at_the_boundary_is_the_ramp_itself_not_the_ramp_times_the_coverage() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_feather] sem adapter — skip");
        return;
    };
    let px = feathered(&gpu);
    let mut vals = Vec::new();
    for y in 2..H - 2 {
        for x in 2..W - 2 {
            if signed(x, y).abs() < 0.12 {
                let o = ((y * W + x) * 4) as usize;
                vals.push(f64::from(px[o + 3]) / 255.0);
            }
        }
    }
    assert!(
        vals.len() >= 4,
        "poucos texels na fronteira: {}",
        vals.len()
    );
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    assert!(
        (mean - 0.5).abs() <= 0.06,
        "alfa médio na fronteira = {mean:.3}, esperado ~0,50 ({} texels): a cobertura da fonte \
         está sendo contada DUAS vezes",
        vals.len()
    );
}
