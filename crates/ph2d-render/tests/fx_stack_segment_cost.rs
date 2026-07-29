//! **O PREÇO da semeadura pela geometria** — o número que autoriza o teto de segmentos.
//!
//! O passe de campo é `O(texels × segmentos)`: ele pergunta a CADA texel qual é o ponto mais
//! próximo sobre a silhueta. Isso é o que troca uma estimativa de cobertura (barata e errada por
//! até 0,68 px, com erro de direção que chega ao ângulo inteiro numa aresta rasa) por uma resposta
//! exata — e um custo linear no número de segmentos.
//!
//! ⚠️ **O teto é MEDIDO, não escolhido** (§0): sem este arquivo o `MAX_SEGMENTS` seria um palpite
//! esperando um smoke. O que ele afirma são as DUAS coisas que um teto de custo tem de dizer — de
//! que recurso ele é (tempo de um passe de campo sobre a pior textura que a pilha aloca) e quanto
//! custa no valor escolhido.
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_segment_cost -- --ignored --nocapture`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{make_src, readback, try_headless_gpu};

const W: u32 = 512;
const H: u32 = 512;

/// Um círculo de `n` lados no espaço de texel — segmentos plausíveis, não uma lista degenerada
/// (todos colineares poderiam deixar o compilador dobrar o laço).
fn ring(n: usize) -> Vec<[f32; 4]> {
    let (cx, cy, r) = (256.0_f64, 256.0, 200.0);
    (0..n)
        .map(|i| {
            let a0 = std::f64::consts::TAU * i as f64 / n as f64;
            let a1 = std::f64::consts::TAU * (i + 1) as f64 / n as f64;
            [
                (cx + r * a0.cos()) as f32,
                (cy + r * a0.sin()) as f32,
                (cx + r * a1.cos()) as f32,
                (cy + r * a1.sin()) as f32,
            ]
        })
        .collect()
}

fn disc_src(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let d = f64::from(x).midpoint(0.0).mul_add(0.0, 0.0)
                + ((f64::from(x) - 256.0).powi(2) + (f64::from(y) - 256.0).powi(2)).sqrt();
            if d <= 200.0 {
                let o = ((y * W + x) * 4) as usize;
                bytes[o..o + 4].copy_from_slice(&[235, 175, 60, 255]);
            }
        }
    }
    make_src(gpu, W, H, &bytes)
}

/// **O custo é LINEAR nos segmentos, e no teto ele cabe num frame.**
///
/// Duas afirmações, porque há dois modos de falha e só um é sobre o relógio da máquina: a RAZÃO
/// pega uma regressão de complexidade (alguém trocar o laço por algo quadrático) e é imune à deriva
/// do hardware; o kill pega o resto.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_field_pass_is_linear_in_the_segment_count_and_the_cap_fits_a_frame() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_segcost] sem adapter — skip");
        return;
    };
    let src = disc_src(&gpu);
    let dst = make_output_texture(&gpu, W, H);
    let mut pass = FxStackPass::new(&gpu);
    let op = FxOpGpu {
        kind: FxOp::BEVEL,
        sigma_px: 20.0,
        offset_px: [-12, 12],
        tint: [0.0, 0.0, 0.0, 1.0],
        tint_b: [1.0; 4],
        opacity: 1.0,
        mode: 0,
        blend: 0,
        noise_scale_px: 0.0,
        detail: 1,
        seed: 0,
        grow_px: 0.0,
        hue: 0.0,
        sat: 0.0,
        bright: 0.0,
    };
    let time = |pass: &mut FxStackPass, segs: &[[f32; 4]]| -> f64 {
        // Aquece (compila pipelines, aloca) e só então mede — e o readback força o fim do trabalho,
        // senão o `submit` volta antes de a GPU ter feito qualquer coisa.
        for _ in 0..2 {
            pass.run(&gpu, &src, &dst, W, H, &[op], segs);
            let _ = readback(&gpu, &dst, W, H);
        }
        let t = std::time::Instant::now();
        const N: u32 = 4;
        for _ in 0..N {
            pass.run(&gpu, &src, &dst, W, H, &[op], segs);
            let _ = readback(&gpu, &dst, W, H);
        }
        t.elapsed().as_secs_f64() * 1000.0 / f64::from(N)
    };
    let base = time(&mut pass, &ring(64));
    let cap = time(&mut pass, &ring(ph2d_vec_render_max_segments()));
    let raster = time(&mut pass, &[]);
    eprintln!(
        "[fx_stack_segcost] raster {raster:.2} ms | 64 segs {base:.2} ms | \
         {} segs (o TETO) {cap:.2} ms",
        ph2d_vec_render_max_segments()
    );
    // A razão TETO/64 sob um laço linear é ~64×, mas o passe carrega custo fixo (o blur, o
    // resolve, o readback), então o que se afirma é o LIMITE SUPERIOR: nada de quadrático.
    let ratio = cap / base.max(1e-6);
    assert!(
        ratio < 200.0,
        "custo do campo cresceu {ratio:.1}× para 64× os segmentos — isso não é linear \
         (base {base:.2} ms, teto {cap:.2} ms)"
    );
    assert!(
        cap < 16.0,
        "no teto de segmentos um passe de campo custa {cap:.2} ms — mais que um frame de 60 fps"
    );
}

/// O teto vive na crate do vetor (é ela que produz a lista); aqui só se mede.
fn ph2d_vec_render_max_segments() -> usize {
    4096
}
