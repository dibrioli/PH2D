//! Ruído semeado por um filtro de estado variável (forma TPT, estável em qualquer corte).

use core::f32::consts::PI;

use ph2d_det_math::DetMath;
use ph2d_sfx_recipe::{Noise, NoiseFilter, SAMPLE_RATE};

use crate::rng::Rng;

pub(crate) fn render(buf: &mut [f32], semente: u64, n: &Noise) {
    let mut rng = Rng::new(semente);
    if n.filter == NoiseFilter::White {
        for s in buf {
            *s = rng.bipolar();
        }
        return;
    }
    let sr = SAMPLE_RATE as f32;
    let k = 1.0 / n.q;
    let passo = if n.hz == n.hz_end || buf.len() < 2 {
        1.0
    } else {
        f64::from(n.hz_end / n.hz).det_powf(1.0 / (buf.len() - 1) as f64)
    };
    let mut hz = f64::from(n.hz);
    let mut g = (PI * hz as f32 / sr).det_tan();
    let (mut ic1, mut ic2) = (0.0f32, 0.0f32);
    for s in buf {
        let x = rng.bipolar();
        let a1 = 1.0 / (1.0 + g * (g + k));
        let v3 = x - ic2;
        let v1 = a1 * (ic1 + g * v3);
        let v2 = ic2 + g * v1;
        ic1 = 2.0 * v1 - ic1;
        ic2 = 2.0 * v2 - ic2;
        *s = match n.filter {
            NoiseFilter::Low => v2,
            NoiseFilter::High => x - k * v1 - v2,
            // Normalizado pelo pico: mudar o Q não muda o volume na ressonância.
            NoiseFilter::Band | NoiseFilter::White => k * v1,
        };
        if passo != 1.0 {
            hz *= passo;
            g = (PI * hz as f32 / sr).det_tan();
        }
    }
}
