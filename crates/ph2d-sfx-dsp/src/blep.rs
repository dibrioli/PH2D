//! **O antialias do oscilador:** cada salto ingênuo é substituído por um degrau de banda limitada, escrito também
//! ANTES do salto, o que um render fora de tempo real permite. A tabela é o sinc janelado integrado.
//! Medição do raio e do corte: `docs/SfxLab/03_W2_medicao.md`.

use std::sync::OnceLock;

use ph2d_det_math::DetMath;

/// Meia largura do degrau, em amostras.
pub(crate) const RAIO: usize = 16;
/// O corte do degrau, em fração de Nyquist: é a transição do filtro que come o que passaria de Nyquist.
pub(crate) const CORTE: f64 = 0.9;
const POR_AMOSTRA: usize = 256;

pub(crate) struct Tabelas {
    /// O degrau suave, de 0 a 1. O salto ideal é subtraído na hora: interpolar por cima dele vale −21 dB de ruído.
    degrau: Vec<f64>,
    /// A integral de `degrau`: o mesmo para um salto de inclinação.
    rampa: Vec<f64>,
}

/// Blackman-Harris de 4 termos, com `x` em `[-1, 1]`.
pub(crate) fn janela(x: f64) -> f64 {
    let t = core::f64::consts::PI * (x + 1.0);
    0.35875 - 0.48829 * t.det_cos() + 0.14128 * (2.0 * t).det_cos() - 0.01168 * (3.0 * t).det_cos()
}

/// `sin(πx)/(πx)`.
pub(crate) fn sinc(x: f64) -> f64 {
    if x.abs() < 1.0e-9 {
        1.0
    } else {
        let a = core::f64::consts::PI * x;
        a.det_sin() / a
    }
}

pub(crate) fn tabelas() -> &'static Tabelas {
    static T: OnceLock<Tabelas> = OnceLock::new();
    T.get_or_init(|| {
        let n = 2 * RAIO * POR_AMOSTRA;
        let dx = 1.0 / POR_AMOSTRA as f64;
        let raio = RAIO as f64;
        let mut bruto = Vec::with_capacity(n + 1);
        let mut acumulado = 0.0f64;
        for i in 0..=n {
            bruto.push(acumulado);
            let x = (i as f64 + 0.5) * dx - raio;
            acumulado += CORTE * sinc(CORTE * x) * janela(x / raio) * dx;
        }
        let degrau: Vec<f64> = bruto.iter().map(|s| s / acumulado).collect();
        // Trapézio: a soma pela esquerda deixa um viés que vira um salto no fim da tabela, −55 dB no triângulo.
        let mut rampa = vec![0.0f64];
        let mut soma = 0.0f64;
        for par in degrau.windows(2) {
            soma += (par[0] + par[1]) * 0.5 * dx;
            rampa.push(soma);
        }
        Tabelas { degrau, rampa }
    })
}

/// A tabela em `x`, em amostras a partir do começo dela.
fn le(tabela: &[f64], x: f64) -> f64 {
    let p = x * POR_AMOSTRA as f64;
    let i = p as usize;
    match (tabela.get(i), tabela.get(i + 1)) {
        (Some(a), Some(b)) => a + (b - a) * (p - i as f64),
        (Some(a), None) => *a,
        _ => tabela.last().copied().unwrap_or(0.0),
    }
}

/// As amostras que o degrau em `t` alcança.
fn alcance(t: f64, len: usize) -> core::ops::Range<usize> {
    let raio = RAIO as f64;
    let de = (t - raio).ceil().max(0.0) as usize;
    let ate = ((t + raio).floor().max(0.0) as usize).saturating_add(1);
    de.min(len)..ate.min(len)
}

/// Um salto de `altura` no instante `t` (em amostras). A amostra ingênua já tem o salto ideal; aqui entra a diferença.
pub(crate) fn salto(tab: &Tabelas, buf: &mut [f32], t: f64, altura: f32) {
    let raio = RAIO as f64;
    for j in alcance(t, buf.len()) {
        let x = j as f64 - t;
        let ideal = if x >= 0.0 { 1.0 } else { 0.0 };
        buf[j] += altura * (le(&tab.degrau, x + raio) - ideal) as f32;
    }
}

/// Uma mudança de inclinação de `delta` por amostra no instante `t`.
pub(crate) fn dobra(tab: &Tabelas, buf: &mut [f32], t: f64, delta: f32) {
    let raio = RAIO as f64;
    for j in alcance(t, buf.len()) {
        let x = j as f64 - t;
        let ideal = if x >= 0.0 { x } else { 0.0 };
        buf[j] += delta * (le(&tab.rampa, x + raio) - ideal) as f32;
    }
}
