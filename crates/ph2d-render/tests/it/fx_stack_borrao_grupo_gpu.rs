//! **O borrão por GRUPO dá o MESMO byte que o de sempre** (doc 132 §1, o P1 do Motion): os mesmos
//! taps, os mesmos pesos, a mesma ordem de soma — só a memória de onde se lêem muda. A fixture é a
//! aresta OBLÍQUA antialiasada (cobertura parcial em toda a banda), num tamanho que não é múltiplo
//! do grupo (64), e os degraus cobrem o de FORA e o de DENTRO, o deslocamento sub-pixel e o kernel
//! no tecto (`MAX_HALF`).
//!
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-render --test it borrao_grupo -- --ignored --nocapture`

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

use crate::fx_stack_common::{oblique_source, readback, try_headless_gpu};

const W: u32 = 150;
const H: u32 = 97;

fn degrau(kind: u8, sigma_px: f32, offset_px: [f32; 2]) -> FxOpGpu {
    FxOpGpu {
        kind,
        sigma_px,
        offset_px,
        tint: [1.0, 0.7, 0.2, 0.9],
        tint_b: [1.0; 4],
        opacity: 1.0,
        mode: FxOp::MODE_PROXIMITY,
        blend: 0,
        noise_scale_px: 0.0,
        detail: 1,
        seed: 0,
        grow_px: 0.0,
        hue: 0.0,
        sat: 0.0,
        bright: 0.0,
        stops: [[0.0; 4]; 8],
        stop_pos: [[0.0; 4]; 2],
        stop_count: 0,
        ramp_lut: None,
    }
}

#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn o_borrao_por_grupo_da_o_mesmo_byte() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = oblique_source(&gpu, W, H);
    let dst = make_output_texture(&gpu, W, H);
    let mut pass = FxStackPass::new(&gpu);
    let pilhas: Vec<(&str, Vec<FxOpGpu>)> = vec![
        ("Outer Glow σ 5,3", vec![degrau(FxOp::GLOW, 5.3, [0.0; 2])]),
        ("Glow σ 0,4", vec![degrau(FxOp::GLOW, 0.4, [0.0; 2])]),
        (
            "Glow σ 32 (MAX_HALF)",
            vec![degrau(FxOp::GLOW, 32.0, [0.0; 2])],
        ),
        (
            "Drop Shadow sub-pixel",
            vec![degrau(FxOp::DROP_SHADOW, 4.0, [3.25, -2.5])],
        ),
        (
            "Drop Shadow σ ínfimo",
            vec![degrau(FxOp::DROP_SHADOW, 1e-5, [2.5, 1.75])],
        ),
        (
            "Inner Shadow",
            vec![degrau(FxOp::INNER_SHADOW, 5.3, [6.0, 6.0])],
        ),
        ("Blur", vec![degrau(FxOp::BLUR, 3.0, [0.0; 2])]),
        (
            "Drop Shadow + Glow",
            vec![
                degrau(FxOp::DROP_SHADOW, 3.0, [2.0, 2.0]),
                degrau(FxOp::GLOW, 6.0, [0.0; 2]),
            ],
        ),
    ];
    for (nome, ops) in pilhas {
        let mut fotos = Vec::new();
        for grupo in [false, true] {
            pass.borrao_em_grupo(grupo);
            pass.run(&gpu, &src, &dst, W, H, &ops, &[]);
            fotos.push(readback(&gpu, &dst, W, H));
        }
        let (a, b) = (&fotos[0], &fotos[1]);
        let diferentes = a.iter().zip(b).filter(|(x, y)| x != y).count();
        let pior = a
            .iter()
            .zip(b)
            .map(|(x, y)| x.abs_diff(*y))
            .max()
            .unwrap_or(0);
        let pintados = a.chunks(4).filter(|p| p[3] > 0).count();
        eprintln!(
            "[borrão por grupo] {nome}: {diferentes} bytes diferem (pior {pior}) · {pintados} pintados"
        );
        assert!(pintados > 1_000, "{nome}: o degrau pinta");
        assert_eq!(
            diferentes, 0,
            "{nome}: o borrão por grupo mudou {diferentes} bytes (pior {pior})"
        );
    }
}
