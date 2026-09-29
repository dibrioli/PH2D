//! ⭐⭐⭐ **A inércia da rolagem** — a lei pura do LANÇAMENTO (spec `04_a_rolagem_unica` §3.4).
//!
//! Largar o dedo depois de arrastar o corpo de um painel **não pára a lista no sítio**: ela
//! continua com a velocidade que o dedo tinha e desacelera até parar. É o gesto de todo sistema
//! tátil, e até 2026-09-29 este app não o tinha em painel nenhum.
//!
//! Duas perguntas, duas funções, **zero estado** (quem guarda as amostras e a velocidade em voo é
//! o `WidgetStore`; quem anda o relógio é o tique da UI viva):
//!
//! 1. [`release_velocity`] — *com que velocidade o dedo largou?*
//! 2. [`step`] — *onde está a lista depois de `dt`, e com que velocidade?*
//!
//! ## De onde vêm os números — nenhum é escolhido aqui
//!
//! | constante | valor | fonte |
//! |---|---|---|
//! | [`HORIZON_NS`] | `100 ms` | AOSP `VelocityTracker` — `HORIZON` (Apache-2.0) |
//! | [`STOPPED_NS`] | `40 ms` | AOSP `VelocityTracker` — `ASSUME_POINTER_STOPPED_TIME` |
//! | [`MIN_FLING`] | `50 px/s` | AOSP `ViewConfiguration.MINIMUM_FLING_VELOCITY` |
//! | [`MAX_FLING`] | `8000 px/s` | AOSP `ViewConfiguration.MAXIMUM_FLING_VELOCITY` |
//! | [`DECAY_PER_MS`] | `0.998` | Apple `UIScrollView.DecelerationRate.normal` |
//!
//! ⚠️ **São constantes de PRODUTO, não de recurso** (o §0.0 pede que se diga): elas descrevem o que
//! um dedo produz e como uma lista «pesa» nos dois sistemas que a mão do artista já conhece. As
//! duas da AOSP estão em `dp/s`; aqui aplicam-se a píxeis físicos, que é a unidade que o evento
//! traz — num ecrã de densidade 2 o limiar de lançamento fica, em milímetros, metade do Android.
//! ⏳ Converter pela escala do monitor é uma melhoria nomeada, não um defeito.
//!
//! ## A lei da paragem é EXACTA, não um limiar de velocidade escolhido
//!
//! Com `v(t) = v₀·e^{−kt}` a lista ainda vai percorrer `v/k` antes de parar. Ela pára quando isso
//! é **menos de meio pixel** — o que ela ainda andaria já não se veria. É independente da taxa de
//! quadros, ao contrário de «pára quando a deslocação por quadro for pequena».

/// Janela das amostras que contam para a velocidade de largada.
pub const HORIZON_NS: u128 = 100_000_000;
/// Se o último `Move` é mais velho que isto quando o dedo sai, o dedo **parou antes de largar** e
/// a velocidade é zero — senão uma lista arrastada, segurada e largada saltava.
pub const STOPPED_NS: u128 = 40_000_000;
/// Abaixo disto largar não lança (px/s).
pub const MIN_FLING: f32 = 50.0;
/// Tecto da velocidade de lançamento (px/s).
pub const MAX_FLING: f32 = 8000.0;
/// Fracção da velocidade que sobra a cada milissegundo.
pub const DECAY_PER_MS: f64 = 0.998;

/// `k` da exponencial, em `1/s`: `v(t) = v₀·e^{−k·t}`.
#[must_use]
pub fn decay_rate() -> f64 {
    -DECAY_PER_MS.ln() * 1000.0
}

/// ⭐ **A velocidade do DEDO no momento em que ele sai**, em px/s (positiva = para baixo).
///
/// `samples` são `(tempo_ns, y)` dos `Move` do arrasto, por ordem. Ajuste de mínimos quadrados de
/// `y` contra `t` sobre as amostras dos últimos [`HORIZON_NS`] — a média de duas pontas
/// ampliaria o tremor da última amostra, e um evento de rato chega com jitter de milissegundos.
///
/// Devolve `0.0` quando não há do que medir (menos de duas amostras na janela) e quando o dedo
/// parou antes de largar ([`STOPPED_NS`]). O resultado **não** é limitado aqui: o [`MIN_FLING`] e
/// o [`MAX_FLING`] são a lei do lançamento, não da medida.
#[must_use]
pub fn release_velocity(samples: &[(u128, f32)], t_up_ns: u128) -> f32 {
    let Some(&(t_last, _)) = samples.last() else {
        return 0.0;
    };
    if t_up_ns.saturating_sub(t_last) > STOPPED_NS {
        return 0.0;
    }
    let window: Vec<(f64, f64)> = samples
        .iter()
        .filter(|(t, _)| t_last.saturating_sub(*t) <= HORIZON_NS)
        // ⚠️ O tempo entra RELATIVO à última amostra, em segundos: um `u128` de nanossegundos
        //    desde o arranque não cabe num `f64` sem perder os bits que medem a velocidade.
        .map(|(t, y)| {
            #[allow(clippy::cast_precision_loss)]
            let dt = -((t_last - *t) as f64) * 1e-9;
            (dt, f64::from(*y))
        })
        .collect();
    if window.len() < 2 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let n = window.len() as f64;
    let mt = window.iter().map(|(t, _)| t).sum::<f64>() / n;
    let my = window.iter().map(|(_, y)| y).sum::<f64>() / n;
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for (t, y) in &window {
        sxy += (t - mt) * (y - my);
        sxx += (t - mt) * (t - mt);
    }
    if sxx <= f64::EPSILON {
        return 0.0;
    }
    #[allow(clippy::cast_possible_truncation)]
    let v = (sxy / sxx) as f32;
    v
}

/// ⭐ **A velocidade com que a lista SAI**, dada a do dedo: o conteúdo segue o dedo, logo o
/// deslocamento de rolagem anda ao contrário (dedo para baixo ⇒ `scroll` desce). `None` quando o
/// dedo não chega ao [`MIN_FLING`] — largar devagar é pousar, não lançar.
#[must_use]
pub fn launch(finger_velocity: f32) -> Option<f32> {
    let v = -finger_velocity;
    (v.abs() >= MIN_FLING).then(|| v.clamp(-MAX_FLING, MAX_FLING))
}

/// Um passo do voo: `(posição nova, velocidade nova, acabou?)`.
///
/// ⚠️ **Integração EXACTA da exponencial** (`Δ = v·(1 − e^{−k·dt})/k`), e não Euler: com Euler o
/// quanto a lista anda dependeria do `dt`, e dois monitores com taxas diferentes dariam à mesma
/// largada distâncias diferentes. A lei aqui é *o traço é facto do gesto, nunca da amostragem* —
/// a mesma que o Painter pagou seis vezes.
///
/// Acaba ao bater numa borda (`0` ou `max`) ou quando o que falta percorrer é menos de meio pixel.
#[must_use]
pub fn step(pos: f32, v: f32, dt_s: f64, max: f32) -> (f32, f32, bool) {
    let k = decay_rate();
    let e = (-k * dt_s).exp();
    let travel = f64::from(v) * (1.0 - e) / k;
    #[allow(clippy::cast_possible_truncation)]
    let (raw, v_next) = (pos + travel as f32, (f64::from(v) * e) as f32);
    let hi = max.max(0.0);
    let clamped = raw.clamp(0.0, hi);
    if (clamped - raw).abs() > f32::EPSILON {
        return (clamped, 0.0, true);
    }
    #[allow(clippy::cast_possible_truncation)]
    let remaining = (f64::from(v_next.abs()) / k) as f32;
    if remaining < 0.5 {
        return (clamped, 0.0, true);
    }
    (clamped, v_next, false)
}

#[cfg(test)]
#[path = "fling_tests.rs"]
mod fling_tests;
