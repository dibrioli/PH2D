//! O oscilador: seno, triângulo, serra, quadrada e FM, com a altura em glissando exponencial.

use core::f32::consts::TAU;

use ph2d_det_math::DetMath;
use ph2d_sfx_recipe::{MAX_HZ, SAMPLE_RATE, Tone, Wave, frames_of};

use crate::blep;

/// O glissando: `inc` multiplica até o fim do `glide` e depois fica.
struct Altura {
    inc: f32,
    /// A conta em `f64`: um glissando longo em `f32` não chega à nota declarada.
    exato: f64,
    passo: f64,
    resta: usize,
}

impl Altura {
    fn new(t: &Tone) -> Self {
        let sr = SAMPLE_RATE as f32;
        let n = frames_of(t.glide);
        let passo = if n > 0 && t.hz != t.hz_end {
            f64::from(t.hz_end / t.hz).det_powf(1.0 / n as f64)
        } else {
            1.0
        };
        let inc = if n == 0 { t.hz_end } else { t.hz } / sr;
        Self {
            inc,
            exato: f64::from(inc),
            passo,
            resta: n,
        }
    }

    fn anda(&mut self) {
        if self.resta > 0 {
            self.exato *= self.passo;
            self.inc = self.exato as f32;
            self.resta -= 1;
        }
    }
}

pub(crate) fn render(buf: &mut [f32], t: &Tone) {
    match t.wave {
        Wave::Sine => seno(buf, t),
        Wave::Fm {
            ratio,
            index,
            index_end,
        } => fm(buf, t, ratio, index, index_end),
        Wave::Saw | Wave::Square | Wave::Triangle => saltos(buf, t),
    }
}

fn seno(buf: &mut [f32], t: &Tone) {
    let mut a = Altura::new(t);
    let mut fase = 0.0f32;
    for s in buf {
        *s = (TAU * fase).det_sin();
        fase += a.inc;
        if fase >= 1.0 {
            fase -= 1.0;
        }
        a.anda();
    }
}

/// A modulação de fase com a banda de Carson presa abaixo do teto do oscilador: o índice cede antes que uma banda
/// lateral dobre em Nyquist.
fn fm(buf: &mut [f32], t: &Tone, ratio: f32, index: f32, index_end: f32) {
    let teto = MAX_HZ / SAMPLE_RATE as f32;
    let mut a = Altura::new(t);
    let n = frames_of(t.glide);
    let inclina = if n > 0 {
        (index_end - index) / n as f32
    } else {
        0.0
    };
    let (mut fp, mut fm, mut indice) = (0.0f32, 0.0f32, if n > 0 { index } else { index_end });
    let mut resta = n;
    for s in buf {
        let inc_m = a.inc * ratio;
        let solto = if inc_m > 0.0 {
            ((teto - a.inc) / inc_m - 1.0).max(0.0)
        } else {
            0.0
        };
        let i = indice.min(solto);
        *s = (TAU * fp + i * (TAU * fm).det_sin()).det_sin();
        fp += a.inc;
        fm += inc_m;
        if fp >= 1.0 {
            fp -= 1.0;
        }
        if fm >= 1.0 {
            fm -= fm.floor();
        }
        if resta > 0 {
            indice += inclina;
            resta -= 1;
        }
        a.anda();
    }
}

/// Serra, quadrada e triângulo: a forma ingênua, e depois cada descontinuidade corrigida em volta dela.
fn saltos(buf: &mut [f32], t: &Tone) {
    let tabelas = blep::tabelas();
    let onda = t.wave;
    let mut a = Altura::new(t);
    // Serra e triângulo nascem no zero, para o começo do som não ser um estalo.
    let mut fase = match onda {
        Wave::Saw => 0.5,
        Wave::Triangle => 0.25,
        _ => 0.0f32,
    };
    for i in 0..buf.len() {
        buf[i] += match onda {
            Wave::Saw => 2.0 * fase - 1.0,
            Wave::Square => {
                if fase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            _ => 1.0 - 4.0 * (fase - 0.5).abs(),
        };
        let inc = a.inc;
        let seguinte = fase + inc;
        let quando = |limite: f32| i as f64 + f64::from((limite - fase) / inc);
        if fase < 0.5 && seguinte >= 0.5 {
            match onda {
                Wave::Square => blep::salto(tabelas, buf, quando(0.5), -2.0),
                Wave::Triangle => blep::dobra(tabelas, buf, quando(0.5), -8.0 * inc),
                _ => {}
            }
        }
        if seguinte >= 1.0 {
            match onda {
                Wave::Saw => blep::salto(tabelas, buf, quando(1.0), -2.0),
                Wave::Square => blep::salto(tabelas, buf, quando(1.0), 2.0),
                _ => blep::dobra(tabelas, buf, quando(1.0), 8.0 * inc),
            }
            fase = seguinte - 1.0;
        } else {
            fase = seguinte;
        }
        a.anda();
    }
}
