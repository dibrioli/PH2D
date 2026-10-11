//! **O deslocamento SUB-PIXEL da pilha** (doc 129 §5.5 A7): a sombra anda contínua com o
//! deslocamento, e a luz do bevel é a DIREÇÃO pedida mesmo abaixo de meio pixel. Antes, o
//! `FxOpGpu::resolve` arredondava o deslocamento a pixel inteiro: a sombra saltava aos degraus e,
//! com `|offset| < 0,5` px (um zoom-out), a luz caía no `(0, −1)` do shader.
//!
//! `PH2D_GPU=1 cargo test -p ph2d-render --test it fx_stack_subpixel -- --ignored --nocapture`

use ph2d_fx_op::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

use crate::fx_stack_common::{make_src, op, readback, try_headless_gpu};

const W: u32 = 64;

/// Um disco de raio 10 no centro, cobertura de área por 8×8 sub-amostras, branco opaco.
fn disco() -> Vec<u8> {
    let mut bytes = vec![0u8; (W * W * 4) as usize];
    for y in 0..W {
        for x in 0..W {
            let mut n = 0u32;
            for sy in 0..8 {
                for sx in 0..8 {
                    let px = f64::from(x) + (f64::from(sx) + 0.5) / 8.0 - 32.0;
                    let py = f64::from(y) + (f64::from(sy) + 0.5) / 8.0 - 32.0;
                    n += u32::from(px * px + py * py < 100.0);
                }
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let a = (f64::from(n) / 64.0 * 255.0).round() as u8;
            let o = ((y * W + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[a, a, a, a]);
        }
    }
    bytes
}

fn corre(gpu: &ph2d_gpu::GpuContext, ops: &[FxOpGpu]) -> Vec<u8> {
    let src = make_src(gpu, W, W, &disco());
    let dst = make_output_texture(gpu, W, W);
    FxStackPass::new(gpu).run(gpu, &src, &dst, W, W, ops, &[]);
    readback(gpu, &dst, W, W)
}

/// O centróide em X da SOMBRA (o que ficou escuro fora do disco: alfa da saída onde a fonte é zero).
fn centroide_da_sombra(px: &[u8], src: &[u8]) -> f64 {
    let (mut sx, mut s) = (0.0, 0.0);
    for y in 0..W {
        for x in 0..W {
            let o = ((y * W + x) * 4) as usize;
            if src[o + 3] == 0 {
                let a = f64::from(px[o + 3]);
                sx += a * f64::from(x);
                s += a;
            }
        }
    }
    sx / s
}

/// ⭐ A sombra ANDA com o deslocamento em passos de ¼ px — cada passo move o centróide dela.
/// Controlo: com o deslocamento ARREDONDADO (a lei antiga), três dos quatro passos não a movem.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_shadow_moves_by_quarter_pixels() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = disco();
    let sombra = |dx: f32| {
        let mut o = op(FxOp::DROP_SHADOW, 2.0, [0.0, 0.0, 0.0, 1.0]);
        o.offset_px = [dx, 0.0];
        centroide_da_sombra(&corre(&gpu, &[o]), &src)
    };
    let passos = [2.0f32, 2.25, 2.5, 2.75, 3.0];
    let c: Vec<f64> = passos.iter().map(|&d| sombra(d)).collect();
    let r: Vec<f64> = passos.iter().map(|&d| sombra(d.round())).collect();
    eprintln!("[sub-pixel] centróide {c:?} · arredondado {r:?}");
    for par in c.windows(2) {
        assert!(par[1] - par[0] > 0.05, "cada ¼ px move a sombra: {c:?}");
    }
    let parados = r.windows(2).filter(|p| (p[1] - p[0]).abs() < 1e-6).count();
    assert!(
        parados >= 2,
        "controlo: arredondado ela pára aos degraus ({r:?})"
    );
}

/// ⭐ A luz do bevel é a DIREÇÃO pedida, de qualquer comprimento — `(0,3, 0,2)` acende o mesmo
/// que `(3, 2)`. Controlo: a luz de cima (`(0, −1)`, onde o arredondamento a deixava cair) é OUTRA.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_bevel_light_is_a_direction_below_half_a_pixel() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let bevel = |l: [f32; 2]| {
        let mut o = op(FxOp::BEVEL, 4.0, [0.0, 0.0, 0.0, 1.0]);
        o.offset_px = l;
        corre(&gpu, &[o])
    };
    let curta = bevel([0.3, 0.2]);
    let longa = bevel([3.0, 2.0]);
    let de_cima = bevel([0.0, -1.0]);
    let dif = |a: &[u8], b: &[u8]| {
        a.iter()
            .zip(b)
            .map(|(p, q)| p.abs_diff(*q))
            .max()
            .unwrap_or(0)
    };
    eprintln!(
        "[luz] curta × longa {} níveis · curta × de cima {} níveis",
        dif(&curta, &longa),
        dif(&curta, &de_cima)
    );
    assert!(dif(&curta, &longa) <= 1, "a luz curta acende como a longa");
    assert!(
        dif(&curta, &de_cima) > 20,
        "controlo: a luz de cima acende OUTRO lado"
    );
}
