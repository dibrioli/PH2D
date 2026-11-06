//! **A proteção dos ouvidos na escuta.** Corre sobre uma cópia do som: o arquivo nunca passa por aqui.
//!
//! Limitador com antecipação, offline: o ganho pedido por amostra desce na hora e sobe devagar (soltura), toma o mínimo
//! da janela à frente e passa por uma média da mesma largura. Toda média que cobre uma amostra só contém valores menores
//! ou iguais ao ganho que ela pediu, logo nenhum pico passa do alvo; é a construção, não um ajuste.

use std::collections::VecDeque;

use ph2d_det_math::DetMath;

/// −1 dBFS, a margem usual de pico.
pub const TETO_DA_ESCUTA: f32 = 0.891_250_9;
/// O alvo fica 0,1 % abaixo do teto: é a folga dos arredondamentos da média.
const ALVO: f32 = TETO_DA_ESCUTA * 0.999;
/// Os tempos usuais de um limitador de pico. Decidem o caráter do estouro, nunca a garantia do teto.
const ANTECIPA_S: f32 = 0.005;
const SOLTA_S: f32 = 0.05;

/// O som como se ouve: igual ao bit se já cabe no teto. Não-número vira silêncio.
#[must_use]
pub fn limitar_escuta(amostras: &[f32], taxa: u32) -> Vec<f32> {
    let limpo = |x: f32| if x.is_finite() { x } else { 0.0 };
    let n = amostras.len();
    if amostras.iter().all(|x| limpo(*x).abs() <= TETO_DA_ESCUTA) {
        return amostras.iter().map(|x| limpo(*x)).collect();
    }

    let janela = (ANTECIPA_S * taxa as f32) as usize;
    let sobe = 1.0 - (-1.0 / (f64::from(SOLTA_S) * f64::from(taxa))).det_exp();
    // Em `f64`: em `f32` o passo da soltura some perto de 1 e o som ficaria 0,014 % abaixo para sempre.
    let mut ganho = 1.0f64;
    let pedido: Vec<f32> = amostras
        .iter()
        .map(|x| {
            let a = limpo(*x).abs();
            let quer = if a > ALVO { f64::from(ALVO / a) } else { 1.0 };
            ganho = quer.min(ganho + (1.0 - ganho) * sobe);
            ganho as f32
        })
        .collect();

    // A média de `janela + 1` mínimos, em `f64`: a soma corrida anda milhões de amostras sem acumular erro audível.
    let mut minimos: VecDeque<usize> = VecDeque::new();
    let mut historia: VecDeque<f32> = VecDeque::with_capacity(janela + 2);
    let mut soma = 0.0f64;
    let mut saida = Vec::with_capacity(n);
    let mut proximo = 0;
    for (i, amostra) in amostras.iter().enumerate() {
        let ate = (i + janela).min(n - 1);
        while proximo <= ate {
            while minimos
                .back()
                .is_some_and(|&j| pedido[j] >= pedido[proximo])
            {
                minimos.pop_back();
            }
            minimos.push_back(proximo);
            proximo += 1;
        }
        while minimos.front().is_some_and(|&j| j < i) {
            minimos.pop_front();
        }
        let minimo = minimos.front().map_or(1.0, |&j| pedido[j]);
        historia.push_back(minimo);
        soma += f64::from(minimo);
        if historia.len() > janela + 1 {
            soma -= f64::from(historia.pop_front().unwrap_or(1.0));
        }
        let media = soma / historia.len() as f64;
        saida.push(limpo(*amostra) * media as f32);
    }
    saida
}
