//! A curva de volume sobre o buffer de uma camada.

use ph2d_det_math::DetMath;
use ph2d_sfx_recipe::{Envelope, frames_of};

pub(crate) fn apply(env: &Envelope, buf: &mut [f32]) {
    let Some(primeiro) = env.points.first() else {
        return;
    };
    let total = buf.len();
    let inicio = frames_of(primeiro.t).min(total);
    for s in &mut buf[..inicio] {
        *s *= primeiro.v;
    }
    let mut pos = inicio;
    for par in env.points.windows(2) {
        let (a, b) = (par[0], par[1]);
        let fim = frames_of(b.t).min(total);
        if fim <= pos {
            continue;
        }
        let n = frames_of(b.t) - frames_of(a.t);
        let trecho = &mut buf[pos..fim];
        if b.exp && a.v > 0.0 && b.v > 0.0 {
            // Em `f64`: dois minutos de produtos em `f32` terminam 5 % longe do valor declarado.
            let passo = f64::from(b.v / a.v).det_powf(1.0 / n as f64);
            let mut v = f64::from(a.v);
            for s in trecho {
                *s *= v as f32;
                v *= passo;
            }
        } else {
            let inclina = (b.v - a.v) / n as f32;
            for (k, s) in trecho.iter_mut().enumerate() {
                *s *= a.v + inclina * k as f32;
            }
        }
        pos = fim;
    }
    if let Some(ultimo) = env.points.last() {
        for s in &mut buf[pos..] {
            *s *= ultimo.v;
        }
    }
}
