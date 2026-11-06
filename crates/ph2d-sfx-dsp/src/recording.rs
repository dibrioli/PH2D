//! A leitura do material gravado em outra altura e outra taxa: interpolação por sinc janelado, com o corte descendo
//! quando a leitura acelera, para o que passaria de Nyquist não dobrar.

use std::borrow::Cow;
use std::sync::OnceLock;

use ph2d_det_math::DetMath;
use ph2d_sfx_recipe::{Recording, SAMPLE_RATE, Take};

use crate::blep::{CORTE, RAIO, janela, sinc};

const POR_ZERO: usize = 512;

fn tabela() -> &'static [f32] {
    static T: OnceLock<Vec<f32>> = OnceLock::new();
    T.get_or_init(|| {
        let raio = RAIO as f64;
        (0..=RAIO * POR_ZERO + 1)
            .map(|i| {
                let x = i as f64 / POR_ZERO as f64;
                if x >= raio {
                    0.0
                } else {
                    (sinc(x) * janela(x / raio)) as f32
                }
            })
            .collect()
    })
}

pub(crate) fn render(buf: &mut [f32], take: &Take, r: &Recording) {
    let mono: Cow<[f32]> = if take.channels == 1 {
        Cow::Borrowed(&take.samples)
    } else {
        Cow::Owned((0..take.frames()).map(|i| take.mono(i)).collect())
    };
    if mono.is_empty() {
        return;
    }
    let razao = f64::from(take.rate) / f64::from(SAMPLE_RATE)
        * if r.semitones == 0.0 {
            1.0
        } else {
            2.0f64.det_powf(f64::from(r.semitones) / 12.0)
        };
    let inicio = f64::from(r.offset) * f64::from(take.rate);

    // Mesma taxa e mesma altura: o material é copiado, sem filtro nenhum.
    if razao == 1.0 && inicio.fract() == 0.0 {
        let de = (inicio as usize).min(mono.len());
        for (o, s) in buf.iter_mut().zip(&mono[de..]) {
            *o = *s;
        }
        return;
    }

    let tab = tabela();
    let corte = CORTE * (1.0 / razao).min(1.0);
    let meia = RAIO as f64 / corte;
    let escala = corte as f32;
    for (i, o) in buf.iter_mut().enumerate() {
        let pos = inicio + i as f64 * razao;
        if pos - meia >= mono.len() as f64 {
            break;
        }
        let de = (pos - meia).ceil().max(0.0) as usize;
        let ate = ((pos + meia).floor() as usize).min(mono.len().saturating_sub(1));
        let mut soma = 0.0f32;
        for k in de..=ate {
            let x = (k as f64 - pos).abs() * corte * POR_ZERO as f64;
            let j = x as usize;
            if j + 1 >= tab.len() {
                continue;
            }
            let f = (x - j as f64) as f32;
            soma += mono[k] * (tab[j] + (tab[j + 1] - tab[j]) * f);
        }
        *o = soma * escala;
    }
}
