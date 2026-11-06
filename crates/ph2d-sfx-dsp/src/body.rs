//! **O corpo vibrante:** um contato curto excita um banco paralelo de ressonadores (síntese modal, J. O. Smith, PASP).
//! Cada modo é um polo complexo: não perde precisão com grave e queda longa, como a forma direta de dois polos.

use core::f32::consts::PI;

use ph2d_det_math::DetMath;
use ph2d_sfx_recipe::{Body, MAX_HZ, ModeLaw, SAMPLE_RATE};

use crate::FLOOR;
use crate::rng::Rng;

/// −60 dB em log natural.
const LN_MILESIMO: f32 = -6.907_755;
/// De quantas em quantas amostras um modo pergunta se já calou: a conta custa tanto quanto a amostra.
const CHECA_A_CADA: usize = 256;

/// Barra livre nas duas pontas, razão ao 1.º modo: `((2k + 1) / 3,0112)²` a partir do 3.º (Euler-Bernoulli).
fn barra(k: u32) -> f32 {
    match k {
        0 => 1.0,
        1 => 2.756,
        _ => {
            let x = (2.0 * k as f32 + 3.0) / 3.0112;
            x * x
        }
    }
}

/// Membrana circular: os zeros de `J_m` em ordem, sobre o primeiro (2,4048).
const MEMBRANA: [f32; 16] = [
    1.0, 1.593, 2.136, 2.296, 2.653, 2.917, 3.156, 3.500, 3.598, 3.647, 4.059, 4.132, 4.230, 4.601,
    4.610, 4.832,
];

fn razao(law: ModeLaw, k: u32) -> Option<f32> {
    match law {
        ModeLaw::Stretched { stretch } => Some(((k + 1) as f32).det_powf(stretch)),
        ModeLaw::Bar => Some(barra(k)),
        ModeLaw::Membrane => MEMBRANA.get(k as usize).copied(),
    }
}

/// O contato: meia-senoide com área 1, modulada pelo ruído semeado. Mais curto excita mais agudo.
fn contato(semente: u64, b: &Body) -> Vec<f32> {
    let n = ((b.contact_ms * SAMPLE_RATE as f32 / 1000.0).round() as usize).max(1);
    let mut rng = Rng::new(semente);
    let mut pulso: Vec<f32> = (0..n)
        .map(|i| {
            let liso = (PI * (i as f32 + 0.5) / n as f32).det_sin();
            liso * ((1.0 - b.noise) + b.noise * rng.bipolar())
        })
        .collect();
    let liso: f32 = (0..n)
        .map(|i| (PI * (i as f32 + 0.5) / n as f32).det_sin())
        .sum();
    for s in &mut pulso {
        *s /= liso;
    }
    pulso
}

pub(crate) fn render(buf: &mut [f32], semente: u64, b: &Body) {
    let golpe = contato(semente, b);
    let sr = SAMPLE_RATE as f32;
    let sigma = -LN_MILESIMO / b.t60;

    // Os modos dentro da banda, cada um com a queda dele; os ganhos somam 1, então o número de modos não muda o volume.
    let mut modos: Vec<(f32, f32, f32)> = Vec::new();
    let mut soma = 0.0f32;
    for k in 0..b.modes {
        let Some(r) = razao(b.law, k) else { break };
        let f = b.base_hz * r;
        if f >= MAX_HZ {
            break;
        }
        let ganho = 1.0 / ((k + 1) as f32).det_powf(b.rolloff);
        soma += ganho;
        modos.push((f, sigma * (1.0 + b.damping * (r - 1.0)), ganho));
    }
    if soma <= 0.0 {
        return;
    }

    for (f, s_k, ganho) in modos {
        let raio = (-s_k / sr).det_exp();
        let (sin, cos) = (2.0 * PI * f / sr).det_sin_cos();
        let (c, s) = (raio * cos, raio * sin);
        let g = ganho / soma;
        let (mut re, mut im) = (0.0f32, 0.0f32);
        let piso = FLOOR * FLOOR;
        for (i, o) in buf.iter_mut().enumerate() {
            let x = golpe.get(i).copied().unwrap_or(0.0);
            let re2 = c * re - s * im + x;
            im = s * re + c * im;
            re = re2;
            *o += im * g;
            if i % CHECA_A_CADA == 0 && i >= golpe.len() && re * re + im * im < piso {
                break;
            }
        }
    }
}
