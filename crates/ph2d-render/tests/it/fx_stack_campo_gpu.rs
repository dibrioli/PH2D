//! **O CAMPO DE DISTÂNCIA do raster contra a VERDADE analítica** (doc 129 §3.4.1 e §5.6).
//!
//! A morfologia é a leitura mais direta do campo: `α = clamp(sdist + r + 0,5)`, logo cada texel da
//! rampa devolve a distância com sinal que o campo lhe deu, a 1/255 px. A forma é um DISCO de
//! centro sub-pixel com cobertura de ÁREA exata — ele tem TODAS as inclinações (uma aresta
//! axis-aligned nunca exercita a semente sub-pixel nem o ramo do pé) e a distância dele é conhecida
//! em fórmula fechada.
//!
//! `PH2D_GPU=1 cargo test -p ph2d-render --test it fx_stack_campo -- --ignored --nocapture`

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

use crate::fx_stack_common::{make_src, op, readback, try_headless_gpu};

const W: u32 = 96;
/// O centro do disco — fora da grade nos dois eixos, e por números diferentes.
const CENTRO: [f64; 2] = [47.37, 48.81];
const RAIO: f64 = 21.6;
/// Os passos de morfologia que varrem o campo dos dois lados da borda, até 4,5 px.
const GROWS: [f32; 8] = [-4.0, -3.0, -2.0, -1.0, 1.0, 2.0, 3.0, 4.0];

/// A distância COM SINAL do centro do texel à circunferência — positiva DENTRO.
fn disco_signed(x: u32, y: u32) -> f64 {
    let dx = f64::from(x) + 0.5 - CENTRO[0];
    let dy = f64::from(y) + 0.5 - CENTRO[1];
    RAIO - dx.hypot(dy)
}

/// A cobertura de ÁREA do texel `(x, y)` pelo disco: a corda vertical integrada em `x` (ponto
/// médio, 512 fatias — o integrando é contínuo, o erro fica abaixo de 1e-5).
fn cobertura(x: u32, y: u32) -> f64 {
    const N: u32 = 512;
    let (y0, y1) = (f64::from(y), f64::from(y) + 1.0);
    let mut acc = 0.0;
    for i in 0..N {
        let px = f64::from(x) + (f64::from(i) + 0.5) / f64::from(N);
        let dx = px - CENTRO[0];
        let h2 = RAIO * RAIO - dx * dx;
        if h2 <= 0.0 {
            continue;
        }
        let h = h2.sqrt();
        acc += ((CENTRO[1] + h).min(y1) - (CENTRO[1] - h).max(y0)).max(0.0);
    }
    acc / f64::from(N)
}

fn disco_fonte(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * W * 4) as usize];
    for y in 0..W {
        for x in 0..W {
            let o = ((y * W + x) * 4) as usize;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let a = (cobertura(x, y) * 255.0).round() as u8;
            bytes[o..o + 4].copy_from_slice(&[235, 175, 60, a]);
        }
    }
    make_src(gpu, W, W, &bytes)
}

fn morph(grow_px: f32) -> FxOpGpu {
    FxOpGpu {
        grow_px,
        ..op(FxOp::MORPHOLOGY, 0.0, [0.0, 0.0, 0.0, 1.0])
    }
}

/// O erro do campo, em px, em cada texel da rampa de cada `r` (sem geometria: semente raster).
fn erros_do_campo(gpu: &ph2d_gpu::GpuContext) -> Vec<f64> {
    let src = disco_fonte(gpu);
    let mut pass = FxStackPass::new(gpu);
    let mut out = Vec::new();
    for r in GROWS {
        let dst = make_output_texture(gpu, W, W);
        pass.run(gpu, &src, &dst, W, W, &[morph(r)], &[]);
        let px = readback(gpu, &dst, W, W);
        for y in 0..W {
            for x in 0..W {
                let v = disco_signed(x, y) + f64::from(r) + 0.5;
                // Só a rampa: fora dela o recorte esconde o campo.
                if !(0.03..=0.97).contains(&v) {
                    continue;
                }
                let a = f64::from(px[((y * W + x) * 4 + 3) as usize]) / 255.0;
                out.push((a - v).abs());
            }
        }
    }
    out
}

/// **O campo raster É a distância euclidiana, a todo ângulo e dos dois lados da borda.**
///
/// As três peças da semente G&S e do finalize, cada uma vista VERMELHA aqui (pior erro em px sobre
/// 1027 texels; o código real dá **0,047**):
///
/// | mutação | pior | rms |
/// |---|---|---|
/// | Q29 `cs_sdf_seed`: `n * edgedf(n, a)` → `n * 0.0` (a semente vai ao centro do texel) | 0,654 | 0,295 |
/// | Q30 `edgedf`: `a1 = 0.0` (os cantos caem no trecho da faixa) | 0,297 | 0,071 |
/// | Q31 `seed_dist`: `return p` (distância ao PONTO, nunca à tangente) | 0,297 | 0,037 |
///
/// A barra (0,1 px) fica a 2× do real e a 3× da mutação mais fraca.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_raster_field_is_the_true_distance_to_a_disc_at_every_angle() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_campo] sem adapter — skip");
        return;
    };
    let e = erros_do_campo(&gpu);
    let pior = e.iter().copied().fold(0.0, f64::max);
    let rms = (e.iter().map(|v| v * v).sum::<f64>() / e.len().max(1) as f64).sqrt();
    let acima = e.iter().filter(|&&v| v > 0.1).count();
    eprintln!(
        "[fx_stack_campo] {} texels | pior {pior:.4} px | rms {rms:.4} px | {acima} acima de 0,1",
        e.len()
    );
    // Controlo: a rampa de oito passos atravessa a borda inteira — sem amostras, o gate é vácuo.
    assert!(e.len() > 900, "só {} texels na rampa", e.len());
    assert!(
        pior <= 0.1,
        "o campo erra {pior:.3} px contra a distância exata ao disco ({acima} texels acima de \
         0,1) — a semente sub-pixel, o pé da aresta ou a distância à tangente se perdeu"
    );
}
