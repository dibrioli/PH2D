//! **A pirâmide do borrão** (doc 129 §5.5 A8): o borrão alcança QUALQUER σ — o oráculo é a lei: o
//! borrão gaussiano de um degrau é `Φ(x/σ)`. Antes o kernel parava em σ = 32 px (`MAX_HALF / 3`) e a
//! régua do nó era o raio dessa saturação numa janela medida.
//!
//! `PH2D_GPU=1 cargo test -p ph2d-render --test it fx_stack_piramide -- --ignored --nocapture`

use ph2d_fx_op::FxOp;
use ph2d_render::{FxStackPass, make_output_texture};

use crate::fx_stack_common::{make_src, op, readback, try_headless_gpu};

const W: u32 = 2000;
const H: u32 = 800;
const BORDA: f64 = 1000.0;

/// `Φ` pela `erf` de Abramowitz & Stegun 7.1.26 (erro < 1,5·10⁻⁷).
fn phi(z: f64) -> f64 {
    let x = z.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0
        - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t
            + 0.254_829_592)
            * t
            * (-x * x).exp();
    let erf = if z >= 0.0 { y } else { -y };
    0.5 * (1.0 + erf)
}

/// O borrão gaussiano do RECTÂNGULO `[0, BORDA) × [0, H)` (fora dele, transparente — a borda da
/// pilha) no centro do texel: o produto das diferenças de `Φ` nos dois eixos. A lei exacta a
/// qualquer σ — um degrau infinito só a aproxima enquanto as bordas da imagem estão longe.
fn lei(x: u32, y: u32, sigma: f64) -> f64 {
    let (cx, cy) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
    let eixo = |c: f64, fim: f64| phi((fim - c) / sigma) - phi(-c / sigma);
    255.0 * eixo(cx, BORDA) * eixo(cy, f64::from(H))
}

/// O pior desvio (níveis de alfa) do rectângulo borrado contra a lei, em três linhas.
fn desvio(px: &[u8], sigma: f64) -> f64 {
    let mut pior = 0.0f64;
    for y in [H / 2, H / 4, 40] {
        for x in (0..W).step_by(7) {
            let a = f64::from(px[((y * W + x) * 4 + 3) as usize]);
            pior = pior.max((a - lei(x, y, sigma)).abs());
        }
    }
    pior
}

/// ⭐ **O borrão de um rectângulo é o produto de `Φ`, a qualquer σ** — 30 px (o kernel de sempre),
/// 100 e 300 (a pirâmide, 2 e 4 reduções). Controlo: a lei antiga (σ parado em 32) desvia dezenas
/// de níveis.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_pyramid_blur_of_a_step_is_the_normal_cdf() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..(BORDA as u32) {
            let o = ((y * W + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let src = make_src(&gpu, W, H, &bytes);
    let mut pass = FxStackPass::new(&gpu);
    for sigma in [30.0f32, 100.0, 300.0] {
        let dst = make_output_texture(&gpu, W, H);
        let o = op(FxOp::BLUR, sigma, [0.0; 4]);
        pass.run(&gpu, &src, &dst, W, H, &[o], &[]);
        let px = readback(&gpu, &dst, W, H);
        let d = desvio(&px, f64::from(sigma));
        let parado = (0..W)
            .map(|x| (lei(x, H / 2, f64::from(sigma)) - lei(x, H / 2, 32.0)).abs())
            .fold(0.0f64, f64::max);
        eprintln!(
            "[pirâmide] σ {sigma}: desvio {d:.2} níveis · a lei antiga (σ 32) desviaria {parado:.1}"
        );
        assert!(
            d <= 2.0,
            "σ {sigma}: o degrau borrado desvia {d:.2} níveis de Φ"
        );
        if sigma > 32.0 {
            assert!(
                parado > 20.0,
                "controlo: a saturação seria visível ({parado:.1})"
            );
        }
    }
}
