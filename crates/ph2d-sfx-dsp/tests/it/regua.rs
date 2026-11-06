//! **A régua de `aliasing`:** um oscilador não tem nada abaixo da fundamental. Tudo o que aparece ali veio de um
//! harmônico que dobrou em Nyquist. Conferida contra o `sox` em `aliasing.rs`, e usada também pela sonda de W2.

use std::f64::consts::PI;

const JANELA: usize = 4096;

fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut tam = 2;
    while tam <= n {
        let a = -2.0 * PI / tam as f64;
        let (wr, wi) = (a.cos(), a.sin());
        for inicio in (0..n).step_by(tam) {
            let (mut cr, mut ci) = (1.0f64, 0.0f64);
            for k in 0..tam / 2 {
                let (u, v) = (inicio + k, inicio + k + tam / 2);
                let (tr, ti) = (re[v] * cr - im[v] * ci, re[v] * ci + im[v] * cr);
                re[v] = re[u] - tr;
                im[v] = im[u] - ti;
                re[u] += tr;
                im[u] += ti;
                let nr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = nr;
            }
        }
        tam *= 2;
    }
}

/// A energia por raia da janela que começa em `de`, com a janela de Hann.
fn espectro(pcm: &[f32], de: usize) -> Vec<f64> {
    let mut re: Vec<f64> = (0..JANELA)
        .map(|i| {
            let w = 0.5 - 0.5 * (2.0 * PI * i as f64 / JANELA as f64).cos();
            f64::from(pcm[de + i]) * w
        })
        .collect();
    let mut im = vec![0.0; JANELA];
    fft(&mut re, &mut im);
    (0..JANELA / 2)
        .map(|k| re[k] * re[k] + im[k] * im[k])
        .collect()
}

/// `aliasing` em dB sobre o sinal: a energia entre 20 Hz e 0,8 × fundamental, contra a energia total. Com `f0(t)`,
/// em segundos, é a PIOR janela de uma varredura. A folga de 0,8 tira o vazamento da janela de Hann. A primeira e a
/// última janela ficam de fora: o começo e o fim do buffer são um estalo de banda larga, não `aliasing`.
#[must_use]
pub fn alias_db(pcm: &[f32], sr: f32, f0: &dyn Fn(f32) -> f32) -> f64 {
    let hz_por_raia = f64::from(sr) / JANELA as f64;
    let mut pior = f64::NEG_INFINITY;
    let mut de = JANELA;
    while de + 2 * JANELA <= pcm.len() {
        let meio = (de + JANELA / 2) as f32 / sr;
        let e = espectro(pcm, de);
        let total: f64 = e.iter().sum();
        let ate = (0.8 * f64::from(f0(meio)) / hz_por_raia) as usize;
        let baixo = (20.0 / hz_por_raia) as usize;
        let abaixo: f64 = e[baixo..ate.max(baixo)].iter().sum();
        if total > 0.0 && abaixo > 0.0 {
            pior = pior.max(10.0 * (abaixo / total).log10());
        }
        de += JANELA / 2;
    }
    pior
}

/// A energia de `pcm` abaixo de `hz`, em dB sobre a energia total; a mesma pergunta do oráculo.
#[must_use]
pub fn abaixo_de_db(pcm: &[f32], sr: f32, hz: f32) -> f64 {
    let hz_por_raia = f64::from(sr) / JANELA as f64;
    let ate = (f64::from(hz) / hz_por_raia) as usize;
    let baixo = (20.0 / hz_por_raia) as usize;
    let (mut abaixo, mut total) = (0.0, 0.0);
    let mut de = JANELA;
    while de + 2 * JANELA <= pcm.len() {
        let e = espectro(pcm, de);
        abaixo += e[baixo..ate].iter().sum::<f64>();
        total += e.iter().sum::<f64>();
        de += JANELA;
    }
    10.0 * (abaixo / total).log10()
}
