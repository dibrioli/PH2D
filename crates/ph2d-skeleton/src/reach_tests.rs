//! Os gates do ALCANCE. ⚠️ Cada um mede um sintoma que o artista vê: a mão não chega, o osso
//! estica, o cotovelo salta, o joelho estala.

use super::reach::*;

/// Uma corrente recta de `n` ossos de comprimento `l`, deitada sobre o eixo X a partir da origem.
fn corrente(n: usize, l: f64) -> (Vec<[f64; 2]>, Vec<f64>) {
    #[expect(clippy::cast_precision_loss, reason = "n é um punhado de ossos")]
    let j = (0..=n).map(|i| [i as f64 * l, 0.0]).collect();
    (j, vec![l; n])
}

fn comprimentos(j: &[[f64; 2]]) -> Vec<f64> {
    j.windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .collect()
}

/// ⭐⭐⭐ **A PONTA CHEGA AO ALVO** — a razão de existir da coisa, nas duas leis.
#[test]
fn the_tip_reaches_a_goal_that_is_within_reach() {
    for n in [2usize, 3, 6, 12] {
        let (mut j, l) = corrente(n, 10.0);
        #[expect(clippy::cast_precision_loss, reason = "idem")]
        let alcance = n as f64 * 10.0;
        // Um alvo a metade do alcance, fora da recta — dobra de verdade.
        let alvo = [alcance * 0.35, alcance * 0.30];
        reach(&mut j, &l, alvo, Reach::default());
        let erro = (j[n][0] - alvo[0]).hypot(j[n][1] - alvo[1]);
        assert!(
            erro < 1e-3 * alcance,
            "com {n} ossos a ponta parou a {erro} do alvo"
        );
    }
}

/// ⛔⛔ **NENHUM OSSO ESTICA** — o invariante das duas leis, e o defeito mais feio possível: um
/// membro que se alonga para chegar lê-se como o rig a partir-se.
#[test]
fn no_bone_ever_stretches_not_even_to_reach_an_impossible_goal() {
    for n in [2usize, 3, 7] {
        for alvo in [[5.0, 5.0], [1e4, -1e4], [0.0, 0.0], [-3.0, 0.0]] {
            let (mut j, l) = corrente(n, 10.0);
            reach(&mut j, &l, alvo, Reach::default());
            for (i, (saiu, pedido)) in comprimentos(&j).iter().zip(&l).enumerate() {
                assert!(
                    (saiu - pedido).abs() < 1e-6,
                    "com {n} ossos e alvo {alvo:?} o osso {i} saiu com {saiu} em vez de {pedido}"
                );
                assert!(saiu.is_finite(), "comprimento nao finito");
            }
        }
    }
}

/// ⭐ **Fora de alcance, a corrente ESTICA na direcção do alvo** — é o que um braço faz, e o que
/// todo solver faz. ⛔ Nunca `NaN`, nunca um membro dobrado ao contrário.
#[test]
fn out_of_reach_the_chain_points_straight_at_the_goal() {
    for n in [2usize, 5] {
        let (mut j, l) = corrente(n, 10.0);
        let alvo = [600.0, 800.0]; // a 1000 de distância, alcance 20..50
        reach(&mut j, &l, alvo, Reach::default());
        let u = [0.6, 0.8];
        for (i, junta) in j.iter().enumerate().take(n + 1).skip(1) {
            #[expect(clippy::cast_precision_loss, reason = "idem")]
            let esperado = [u[0] * 10.0 * i as f64, u[1] * 10.0 * i as f64];
            let d = (junta[0] - esperado[0]).hypot(junta[1] - esperado[1]);
            assert!(d < 1e-3, "a junta {i} saiu de linha em {d} com {n} ossos");
        }
    }
}

/// ⭐⭐⭐ **O COTOVELO NÃO SALTA** — a dobra ACTUAL é preservada.
///
/// ⚠️ É o defeito que um bit de «lado» produz e que nenhuma quantidade de afinação cura: com o
/// lado fixo, a mão a atravessar a recta raiz→alvo faz o cotovelo trocar de lado num quadro. Aqui
/// o sinal sai do produto vectorial da pose que o artista já vê.
#[test]
fn the_elbow_keeps_the_side_it_is_already_bent_to() {
    for lado in [1.0_f64, -1.0] {
        let j = [[0.0, 0.0], [10.0, 5.0 * lado], [20.0, 0.0]];
        let l = comprimentos(&j);
        // Varre a mão de um lado ao outro da recta raiz→alvo e confere que a dobra nunca inverte.
        for k in -20..=20 {
            let alvo = [15.0, f64::from(k)];
            let mut q = j;
            reach(&mut q, &l, alvo, Reach::default());
            let (v1, v2) = (
                [q[1][0] - q[0][0], q[1][1] - q[0][1]],
                [q[2][0] - q[1][0], q[2][1] - q[1][1]],
            );
            let cruz = v1[0] * v2[1] - v1[1] * v2[0];
            assert!(
                cruz * lado >= -1e-9,
                "o cotovelo trocou de lado no alvo {alvo:?} (cruz {cruz}, lado {lado})"
            );
        }
    }
}

/// ⭐⭐ **A SOFTNESS NUNCA DEIXA A CORRENTE TRAVAR, e não abre uma quina.**
///
/// Três propriedades, medidas por varredura: a distância efectiva **nunca** atinge o alcance total
/// (o joelho não estala), ela é **contínua** na dobra, e com `softness = 0` ela é o corte a seco
/// **ao bit**.
#[test]
fn softness_slows_the_chain_before_full_stretch_without_a_kink() {
    const R: f64 = 100.0;
    const S: f64 = 20.0;
    // 1) Nunca chega ao alcance total, por mais longe que o alvo esteja.
    for d in [80.0, 100.0, 200.0, 1e6] {
        let e = softened_distance(d, R, S);
        assert!(e < R, "a {d} a corrente TRAVOU em {e} (alcance {R})");
    }
    // 2) Contínua: o maior salto entre amostras vizinhas é da ordem do passo, não um degrau.
    let (mut ant, mut maior) = (None::<f64>, 0.0_f64);
    for i in 0..=20_000 {
        let d = f64::from(i) * 0.01; // 0 .. 200
        let e = softened_distance(d, R, S);
        if let Some(a) = ant {
            maior = maior.max((e - a).abs());
        }
        ant = Some(e);
    }
    assert!(
        maior < 2e-2,
        "a distancia efectiva saltou {maior} entre amostras a 0,01 - a lei ganhou uma quina"
    );
    // 3) Sem softness é o corte a seco, ao bit.
    for d in [10.0, 99.0, 100.0, 1e6] {
        assert_eq!(softened_distance(d, R, 0.0), d.min(R));
    }
}

/// ⛔ **Uma corrente perfeitamente RECTA resolve-se** — o estado em que um esqueleto acabado de
/// desenhar nasce. Sem o arqueamento, cada passagem do FABRIK devolve a mesma recta e a mão nunca
/// sai do sítio.
#[test]
fn a_dead_straight_chain_still_bends_because_nothing_gives_it_a_side_otherwise() {
    let (mut j, l) = corrente(5, 10.0);
    let alvo = [10.0, 20.0];
    reach(&mut j, &l, alvo, Reach::default());
    let erro = (j[5][0] - alvo[0]).hypot(j[5][1] - alvo[1]);
    assert!(erro < 0.05, "a corrente recta nao dobrou: erro {erro}");
}

/// ⭐ **QUANTAS PASSAGENS o FABRIK de facto precisa** — a sonda que decide o
/// [`DEFAULT_ITERATIONS`], para ele não ser um número escolhido (o defeito que a folha 16 da
/// conferência do Motion nomeia na família de onde esta lei veio).
#[test]
#[ignore = "sonda: imprime, não julga"]
fn measure_how_many_passes_fabrik_actually_needs() {
    for n in [3usize, 4, 6, 12, 24] {
        let alcance = f64::from(u16::try_from(n).unwrap_or(1)) * 10.0;
        let mut pior = 0usize;
        for k in 1..=12 {
            let t = f64::from(k) / 12.0;
            let alvo = [alcance * t * 0.7, alcance * t * 0.5];
            let mut precisou = usize::MAX;
            for it in 1..=MAX_ITERATIONS {
                let (mut j, l) = corrente(n, 10.0);
                reach(
                    &mut j,
                    &l,
                    alvo,
                    Reach {
                        iterations: it,
                        softness: 0.0,
                    },
                );
                if (j[n][0] - alvo[0]).hypot(j[n][1] - alvo[1]) < 1e-4 * alcance {
                    precisou = it;
                    break;
                }
            }
            pior = pior.max(precisou.min(MAX_ITERATIONS));
        }
        println!("[reach] {n} ossos: pior caso {pior} passagem(ns)");
    }
}

/// ⭐ **O PREÇO de um alcance** — a sonda que autoriza o tecto.
#[test]
#[ignore = "sonda de relógio: imprime, não julga"]
fn measure_the_price_of_one_reach() {
    for n in [2usize, 6, 24] {
        let (j0, l) = corrente(n, 10.0);
        let t0 = std::time::Instant::now();
        const N: u32 = 20_000;
        for i in 0..N {
            let mut j = j0.clone();
            let t = f64::from(i % 100) * 0.01;
            reach(&mut j, &l, [40.0 * t, 30.0 * t], Reach::default());
            std::hint::black_box(&j);
        }
        let us = t0.elapsed().as_secs_f64() * 1e6 / f64::from(N);
        println!(
            "[reach] {n} ossos x {DEFAULT_ITERATIONS} passagens: {us:.3} us ({:.4} % de 16,7 ms)",
            us / 16_700.0 * 100.0
        );
    }
}
