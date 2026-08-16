//! Os gates do **warp de fase** — o *Pulse Width* / *Bias*, o grupo O.

use super::*;

/// **O NEUTRO É A IDENTIDADE, BIT A BIT** — e por aritmética, não por promessa:
/// `0.5 / 0.5` é exactamente `1.0`, e no segundo ramo `f − 0.5` é exacto
/// (Sterbenz) e `0.5 + (f − 0.5)` reconstrói `f`.
#[test]
fn the_neutral_pulse_width_is_the_identity_to_the_bit() {
    for k in 0..1000 {
        let f = k as f32 / 1000.0;
        assert_eq!(
            skew(f, 0.5).to_bits(),
            f.to_bits(),
            "pw = 0.5 tem de devolver `f` ao bit, e devolveu {} para {f}",
            skew(f, 0.5)
        );
    }
}

/// **NA SQUARE ELE É O CICLO DE TRABALHO** — a fração do período em cima.
///
/// O oráculo CONTA, não amostra um ponto: quantos dos mil instantes do ciclo
/// saem positivos. Com `pw = 0.25` tem de ser um quarto deles.
#[test]
fn on_the_square_it_is_the_duty_cycle() {
    for (pw, want) in [(0.25_f32, 0.25_f32), (0.5, 0.5), (0.75, 0.75)] {
        let up = (0..1000)
            .filter(|k| waveform(2, *k as f32 / 1000.0, pw) > 0.0)
            .count() as f32
            / 1000.0;
        assert!(
            (up - want).abs() < 0.01,
            "pw {pw}: a Square fica em cima {up:.3} do ciclo, e o pedido é {want:.3}"
        );
    }
}

/// **NA TRIANGLE ELE É O BIAS** — o pico anda.
///
/// ⚠️ O oráculo é ONDE o máximo acontece, não o valor dele: um pico que
/// continuasse a valer `+1` no mesmo lugar satisfaria *"a onda mudou"*.
#[test]
fn on_the_triangle_it_is_the_bias_and_the_peak_moves() {
    let peak = |pw: f32| {
        (0..1000)
            .map(|k| (k as f32 / 1000.0, waveform(1, k as f32 / 1000.0, pw)))
            .fold(
                (0.0_f32, f32::NEG_INFINITY),
                |a, b| if b.1 > a.1 { b } else { a },
            )
            .0
    };
    let (early, mid, late) = (peak(0.2), peak(0.5), peak(0.8));
    assert!(
        early < mid && mid < late,
        "o pico tem de ANDAR com o pulse width: {early:.3} · {mid:.3} · {late:.3}"
    );
    assert!(
        (mid - 0.25).abs() < 0.01,
        "e no neutro ele fica onde sempre esteve (¼): {mid:.3}"
    );
}

/// **UM PULSE WIDTH FORA DA FAIXA AINDA OSCILA DE PONTA A PONTA.**
///
/// ⚠️ **A minha primeira justificativa para o clamp era FALSA, e a mutação a
/// derrubou:** ela dizia *"a lei divide por `p` e por `1 − p`, então os extremos
/// são uma divisão por zero"* — não são. Com `p = 0` o teste `f < p` é sempre
/// falso e o ramo que divide por `p` **nunca corre**; com `p = 1` o `f < 1` é
/// sempre verdadeiro (o `frac` vive em `[0,1)`) e o outro ramo nunca corre. A
/// estrutura dos ramos já protegia, e apagar o clamp mantinha tudo finito —
/// **a mutação sobreviveu aos quatro gates**.
///
/// ⇒ o que o clamp de facto compra é a onda continuar a ser uma ONDA: o
/// `pulse_width` é um param `f32`, logo **dirigível por fio** (doc 58), e com
/// `pw = 7` sem clamp o warp comprime o ciclo inteiro nos primeiros 7% do
/// domínio da forma — a excursão colapsa e a saída fica quase plana. É isso que
/// este gate mede, e é isso que a mutação quebra.
#[test]
fn an_out_of_range_pulse_width_still_swings_the_full_range() {
    let span = |pw: f32| {
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for k in 0..1000 {
            let v = waveform(0, k as f32 / 1000.0, pw);
            assert!(v.is_finite(), "pw {pw} produziu {v}");
            lo = lo.min(v);
            hi = hi.max(v);
        }
        hi - lo
    };
    let neutral = span(0.5);
    for pw in [-5.0_f32, 0.0, 1.0, 7.0] {
        let s = span(pw);
        assert!(
            s > neutral * 0.9,
            "pw {pw}: a onda tem de continuar a oscilar de ponta a ponta — \
             {s:.4} contra {neutral:.4} no neutro"
        );
    }
}
