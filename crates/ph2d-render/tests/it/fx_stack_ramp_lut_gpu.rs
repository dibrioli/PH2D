//! **O degradê ASSADO no dispositivo** (Motion, ZERAR família B) — o Gradient Map do `fx.filter` lê
//! uma `ColorRamp` inteira como tabela (`FxOpGpu::ramp_lut`, binding 5), e o Vector continua nos
//! stops.
//!
//! Rodar: `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-render --test it --release
//! fx_stack_ramp_lut -- --ignored`.

use std::sync::Arc;

use ph2d_color::oklab::OklabColor;
use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte, srgb_to_linear_unit};
use ph2d_color::{
    ColorRamp, GradientPreset, LinearRgba, RampColorMode, RampHue, RampInterp, RampStop,
};
use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

use crate::fx_stack_common::{make_src, readback, try_headless_gpu};

const W: u32 = 96;
const H: u32 = 16;
const X0: u32 = 16;
const X1: u32 = 80;
const MID: u32 = H / 2;

/// O limite dos irmãos (Duotone, Gradient Map): `pow(x, 1/3)` no dispositivo contra `cbrt` no Rust,
/// e duas transferências sRGB no ida-e-volta.
const MAX_DELTA: i32 = 4;

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn cinza(x: u32) -> u8 {
    (255.0 * (x - X0) as f32 / (X1 - X0 - 1) as f32) as u8
}

/// Uma faixa em rampa de cinza — a lei é função da luminância, e uma chapa seria UM ponto dela.
fn fonte(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in 4..12 {
        for x in X0..X1 {
            let o = ((y * W + x) * 4) as usize;
            let v = cinza(x);
            bytes[o..o + 4].copy_from_slice(&[v, v, v, 255]);
        }
    }
    make_src(gpu, W, H, &bytes)
}

fn oklab_l(v: u8) -> f32 {
    let c = srgb_to_linear_byte(v);
    OklabColor::from_linear(LinearRgba::new(c, c, c, 1.0)).l
}

#[expect(clippy::cast_precision_loss, reason = "4096 amostras")]
fn assa(r: &ColorRamp) -> Arc<[[f32; 4]]> {
    let n = FxOp::RAMP_LUT_TEXELS;
    (0..n).map(|i| r.eval(i as f32 / (n - 1) as f32)).collect()
}

/// Um Gradient Map pela porta do produto, com os stops `de → ate` e a tabela `lut`.
fn mapa(de: [f32; 4], ate: [f32; 4], opacity: f32, lut: Option<Arc<[[f32; 4]]>>) -> FxOpGpu {
    let mut op = FxOp::new(FxOp::GRADIENT_MAP);
    op.stops[0] = de;
    op.stops[1] = ate;
    op.stop_pos[..2].copy_from_slice(&[0.0, 1.0]);
    op.stop_count = 2;
    op.opacity = opacity;
    FxOpGpu {
        ramp_lut: lut,
        ..FxOpGpu::resolve(&op, [1.0, 0.0, 0.0, 1.0])
    }
}

fn corre(gpu: &ph2d_gpu::GpuContext, pass: &mut FxStackPass, ops: &[FxOpGpu]) -> Vec<u8> {
    let src = fonte(gpu);
    let dst = make_output_texture(gpu, W, H);
    pass.run(gpu, &src, &dst, W, H, ops, &[]);
    readback(gpu, &dst, W, H)
}

fn px(b: &[u8], x: u32) -> [u8; 4] {
    let o = ((MID * W + x) * 4) as usize;
    [b[o], b[o + 1], b[o + 2], b[o + 3]]
}

/// A rampa de duas pontas sRGB `de → ate`, como `ColorRamp` (linear, RGB, Linear).
fn duas(de: [f32; 4], ate: [f32; 4]) -> ColorRamp {
    let lin = |c: [f32; 4]| [0, 1, 2].map(|i| srgb_to_linear_unit(c[i]));
    let [a, b] = [lin(de), lin(ate)];
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [a[0], a[1], a[2], 1.0]),
            RampStop::new(1.0, [b[0], b[1], b[2], 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

const FRIO: [f32; 4] = [0.10, 0.12, 0.35, 1.0];
const QUENTE: [f32; 4] = [1.0, 0.86, 0.62, 1.0];

/// ⭐ **O dispositivo pinta o `ColorRamp::eval` da luminância** — com o que os stops não têm (HSV,
/// `Ease`, matiz longa em OKLCH). O oráculo é a lei contínua, não a tabela.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_baked_ramp_draws_what_the_colour_library_evaluates() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_ramp] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let mut hsv = GradientPreset::Rainbow.ramp();
    hsv.color_mode = RampColorMode::Hsv;
    hsv.interp = RampInterp::Ease;
    let mut longe = GradientPreset::Heat.ramp();
    longe.color_mode = RampColorMode::Oklch;
    longe.hue = RampHue::Far;
    let mut pior = 0;
    for (nome, r) in [("Rainbow/HSV/Ease", hsv), ("Heat/OKLCH far", longe)] {
        let out = corre(&gpu, &mut pass, &[mapa(FRIO, QUENTE, 1.0, Some(assa(&r)))]);
        for x in X0..X1 {
            let want = r.eval(oklab_l(cinza(x)));
            let got = px(&out, x);
            for c in 0..3 {
                let d = (i32::from(got[c]) - i32::from(linear_to_srgb_byte(want[c]))).abs();
                pior = pior.max(d);
                assert!(
                    d <= MAX_DELTA,
                    "{nome} coluna {x} canal {c}: {got:?} (delta {d})"
                );
            }
        }
    }
    eprintln!("[fx_ramp] pior delta contra o eval: {pior} nível(is)");
}

/// **A tabela VENCE os stops, e duas pontas são a lei dos stops.** Controlo: a mesma `op` com a
/// tabela INVERTIDA pinta a rampa invertida — sem ele, um passe que ignorasse a tabela passaria.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_two_stop_baked_ramp_is_the_stops_law_and_the_table_wins() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_ramp] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let stops = corre(&gpu, &mut pass, &[mapa(FRIO, QUENTE, 1.0, None)]);
    let igual = corre(
        &gpu,
        &mut pass,
        &[mapa(FRIO, QUENTE, 1.0, Some(assa(&duas(FRIO, QUENTE))))],
    );
    let invertida = corre(
        &gpu,
        &mut pass,
        &[mapa(FRIO, QUENTE, 1.0, Some(assa(&duas(QUENTE, FRIO))))],
    );
    let delta = |a: &[u8], b: &[u8]| {
        a.iter()
            .zip(b)
            .map(|(p, q)| (i32::from(*p) - i32::from(*q)).abs())
            .max()
            .unwrap_or(0)
    };
    assert!(
        delta(&stops, &igual) <= 1,
        "duas pontas: {} níveis",
        delta(&stops, &igual)
    );
    assert!(
        delta(&stops, &invertida) > 100,
        "controlo: a tabela invertida tem de se ver"
    );
}

/// **Cada degrau lê a SUA tabela** (a base no buffer): um 1.º degrau inerte (opacidade 0) com outra
/// tabela não muda o que o 2.º pinta. Sem a base, o 2.º leria a tabela do 1.º.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn each_step_reads_its_own_table() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_ramp] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let a = assa(&duas(QUENTE, FRIO));
    let b = assa(&GradientPreset::Rainbow.ramp());
    let so = corre(&gpu, &mut pass, &[mapa(FRIO, QUENTE, 1.0, Some(b.clone()))]);
    let par = corre(
        &gpu,
        &mut pass,
        &[
            mapa(FRIO, QUENTE, 0.0, Some(a)),
            mapa(FRIO, QUENTE, 1.0, Some(b)),
        ],
    );
    assert_eq!(so, par, "o 2.º degrau leu outra tabela");
}
