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
///
/// ⛔⛔ **A 1.ª redacção deste gate CODIFICAVA a inversão que ele diz proibir, e ficou verde sobre
/// ela** (achado 2026-09-07). Ela chamava `lado = +1` a um cotovelo em `+y` e exigia que a saída
/// tivesse produto vectorial **positivo** — mas a entrada com o cotovelo em `+y` tem produto
/// **negativo** (`v1 = [10,5]`, `v2 = [10,−5]` ⇒ `−100`). O gate pedia, à letra, que a dobra
/// trocasse. E o produto fazia-o: a álgebra dá `v1 × v2 = −l1·d·sin a`, e o solver igualava os dois
/// sinais em vez de os opor. *Um gate verde pode pinar um defeito de produto.*
///
/// ⇒ o lado da entrada **deriva-se da própria fixtura**, nunca de um nome que alguém escolheu.
#[test]
fn the_elbow_keeps_the_side_it_is_already_bent_to() {
    let cruz_de = |q: &[[f64; 2]; 3]| {
        let (v1, v2) = (
            [q[1][0] - q[0][0], q[1][1] - q[0][1]],
            [q[2][0] - q[1][0], q[2][1] - q[1][1]],
        );
        v1[0].mul_add(v2[1], -(v1[1] * v2[0]))
    };
    for lado in [1.0_f64, -1.0] {
        let j = [[0.0, 0.0], [10.0, 5.0 * lado], [20.0, 0.0]];
        let l = comprimentos(&j);
        let entrada = cruz_de(&j);
        // Varre a mão de um lado ao outro da recta raiz→alvo e confere que a dobra nunca inverte.
        for k in -20..=20 {
            let alvo = [15.0, f64::from(k)];
            let mut q = j;
            reach(&mut q, &l, alvo, Reach::default());
            let cruz = cruz_de(&q);
            assert!(
                cruz * entrada >= -1e-9,
                "o cotovelo trocou de lado no alvo {alvo:?} (entrada {entrada}, saida {cruz})"
            );
        }
    }
}

/// ⭐⭐⭐ **E RESOLVER DUAS VEZES DÁ A MESMA POSE** — a metade que a 1.ª redacção do gate acima não
/// tinha, e é ela que apanha a inversão sem depender de convenção nenhuma.
///
/// ⚠️ Uma restrição de IK re-resolve **todo quadro** a partir da própria saída. Um solver que
/// inverta a dobra a cada passagem produz uma corrente a **vibrar** — e nenhum gate que resolva
/// UMA vez o vê.
#[test]
fn solving_twice_from_its_own_output_gives_the_same_pose() {
    for lado in [1.0_f64, -1.0] {
        for k in [-14, -5, 0, 7, 16] {
            let alvo = [15.0, f64::from(k)];
            let mut q = [[0.0, 0.0], [10.0, 5.0 * lado], [20.0, 0.0]];
            let l = comprimentos(&q);
            reach(&mut q, &l, alvo, Reach::default());
            let uma = q;
            reach(&mut q, &l, alvo, Reach::default());
            let d = (q[1][0] - uma[1][0]).hypot(q[1][1] - uma[1][1]);
            assert!(
                d < 1e-9,
                "a 2.a passagem moveu o cotovelo {d} no alvo {alvo:?} - a pose nao e' um ponto fixo"
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
                        ..Reach::default()
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

/// ⭐⭐⭐ **A MISTURA DESLIGADA É O NO-OP AO BIT** — a lei da casa (*todo motor novo é no-op no ponto
/// neutro*), aplicada ao número que o artista vai animar de `0` a `1`.
#[test]
fn a_mix_of_zero_returns_the_pose_that_was_already_there() {
    for de in [-3.0, -1.0, 0.0, 0.7, 3.1] {
        for para in [-2.5, 0.0, 1.9, std::f64::consts::PI] {
            for mix in [0.0, -0.5, f64::NAN] {
                let r = super::blend_angle(de, para, mix);
                assert!(
                    r.to_bits() == de.to_bits(),
                    "mix={mix} moveu o angulo de {de} para {r} - a restricao desligada tem de ser \
                     o no-op AO BIT"
                );
            }
        }
    }
}

/// E a mistura CHEIA entrega o alvo ao bit — sem resíduo de `de + (para − de)·1`, que em `f64` não
/// devolve `para` exactamente.
#[test]
fn a_full_mix_returns_the_goal_exactly() {
    for (de, para) in [(0.3, 1.7), (-3.0, 2.9), (1e-9, 1e9)] {
        for mix in [1.0, 1.5, 2.0] {
            let r = super::blend_angle(de, para, mix);
            assert!(
                r.to_bits() == para.to_bits(),
                "mix={mix} entregou {r} em vez de {para} ao bit"
            );
        }
    }
}

/// ⭐⭐⭐ **PELO CAMINHO CURTO** — a meia-mistura entre `+179°` e `−179°` cai em `180°`, e não em `0°`.
///
/// ⚠️ Sem o embrulho o braço **dá uma volta completa** a meio de uma animação, e é o defeito
/// clássico da interpolação de ângulos. A meia-mistura é onde ele é máximo, e por isso é aqui que
/// se mede.
#[test]
fn the_mix_takes_the_short_way_around() {
    let de = 179.0_f64.to_radians();
    let para = (-179.0_f64).to_radians();
    let meio = super::blend_angle(de, para, 0.5);
    // O caminho curto são 2°, então o meio está a 1° de cada um — em 180°.
    let esperado = 180.0_f64.to_radians();
    let erro = super::wrap_pi(meio - esperado).abs();
    assert!(
        erro < 1e-12,
        "a meia-mistura deu {:.4}° em vez de 180° - ela foi pelo caminho LONGO (358°)",
        meio.to_degrees()
    );
    // A metade que prova que não é coincidência: o caminho longo passaria por 0°.
    assert!(
        super::wrap_pi(meio).abs() > 3.0,
        "a meia-mistura passou perto de 0° - isso E' o caminho longo"
    );
}

/// ⚠️ **O embrulho devolve `(-π, π]` e não `[-π, π)`** — os dois leem-se iguais até alguém comparar
/// com o valor exacto. `rem_euclid` sozinho cai no extremo errado, e é a linha que este gate mede.
#[test]
fn the_wrap_lands_on_the_positive_end_of_the_turn() {
    use std::f64::consts::{PI, TAU};
    for k in -3..=3 {
        let r = super::wrap_pi(PI + f64::from(k) * TAU);
        assert!(
            (r - PI).abs() < 1e-12,
            "meia volta + {k} voltas deu {r} em vez de +pi"
        );
    }
}

/// **A MISTURA NÃO MEXE NO COMPRIMENTO** — a razão de ela ser sobre o ÂNGULO.
///
/// Interpolar as POSIÇÕES das juntas encolhe o osso (a corda é mais curta que o arco); aqui o
/// comprimento nem entra na conta, e o gate mede-o sobre a excursão inteira.
#[test]
fn blending_the_angle_never_changes_the_length_of_a_bone() {
    let comp = 37.5_f64;
    let de = 0.4_f64;
    let para = 2.9_f64;
    for i in 0..=20 {
        let mix = f64::from(i) / 20.0;
        let a = super::blend_angle(de, para, mix);
        let ponta = [comp * a.cos(), comp * a.sin()];
        let medido = ponta[0].hypot(ponta[1]);
        assert!(
            (medido - comp).abs() < 1e-12,
            "com mix={mix} o osso mede {medido} e devia medir {comp}"
        );
    }
}

/// ⭐⭐⭐ **UMA CORRENTE RECTA NÃO SORTEIA UM LADO** — o desempate determinístico.
///
/// ⚠️ **O caso não é exótico: é o estado em que um esqueleto acabado de desenhar NASCE.** Com a
/// corrente recta o produto vectorial das duas metades é o de dois vectores paralelos — **ruído** —,
/// e ler o sinal dele faz o cotovelo cair para um lado diferente a cada quadro. Uma restrição de IK
/// re-resolve todo quadro, então isso é uma corrente a **vibrar**.
///
/// ⚠️ **O gate mede a ESTABILIDADE, não qual lado** (*não se escolhe um desempate melhor, não se
/// tem empate*): o que ele exige é que dez passagens seguidas devolvam a MESMA resposta, e que ela
/// seja a mesma para um resíduo positivo e um negativo.
#[test]
fn a_straight_chain_never_draws_lots_for_the_bend_side() {
    let (l1, l2) = (10.0, 10.0);
    // O alvo bem DENTRO do alcance (12 de 20): o atalho da recta não dispara, e a corrente tem de
    // dobrar de verdade.
    let goal = [0.0, 12.0];
    // Duas correntes rectas que diferem só no ULP — o «lado» que o resíduo sugere é oposto.
    let lados: Vec<f64> = [1.0, -1.0]
        .into_iter()
        .map(|s| {
            let mut juntas = [[0.0, 0.0], [l1, s * 1e-12], [l1 + l2, 0.0]];
            let mut anterior = 0.0_f64;
            for i in 0..10 {
                super::reach(&mut juntas, &[l1, l2], goal, super::Reach::default());
                let lado = juntas[1][0].mul_add(goal[1], -(juntas[1][1] * goal[0]));
                if i > 0 {
                    assert!(
                        lado.signum() == anterior.signum(),
                        "a corrente trocou de lado entre passagens ({anterior} -> {lado}) - ela \
                         VIBRA, e uma restricao de IK re-resolve todo quadro"
                    );
                }
                anterior = lado;
            }
            anterior.signum()
        })
        .collect();
    assert_eq!(
        lados[0], lados[1],
        "duas correntes rectas que diferem num ULP cairam para lados OPOSTOS - o desempate esta' a \
         ler o sinal do ruido"
    );
}

/// Uma corrente de dois ossos posta do lado OPOSTO ao desempate da recta, com o alvo que a mantém
/// lá.
///
/// ⚠️⚠️ **A partida tem de ser o lado oposto, e a 1.ª redacção desta fixtura não era** — com o
/// cotovelo já do lado que o desempate escolhe, esticar e voltar devolve **o mesmo sinal**
/// (medido: `antes=+99,498744  depois=+99,498744`) e o gate ficava verde sobre o defeito. É a
/// quarta fixtura desta linha a não produzir o fenómeno que o nome dela promete.
///
/// ⚠️ **A régua destes gates é o [`dominant_side`], nunca o `v1 × v2`** — e a 1.ª redacção deles
/// usava o segundo, o que os fez acusar a implementação CERTA de inverter. Os dois medem o mesmo
/// facto com **sinais opostos**, que é precisamente por que [`side_of`] existe: escrevi a porta
/// para não cair nisto e caí no mesmo turno.
fn cotovelo_do_lado_anti_horario() -> (Vec<[f64; 2]>, Vec<f64>, [f64; 2]) {
    (
        vec![[0.0, 0.0], [6.0, 8.0], [16.0, 8.0]],
        vec![10.0, 10.0],
        [12.0, 6.0],
    )
}

/// ⛔⛔ **O DEFEITO QUE O LADO AUTORADO CURA** — e ele é determinístico, não é ruído.
///
/// O artista dobra o cotovelo para um lado, estica o braço até ele ficar direito, e traz a mão de
/// volta **ao mesmo sítio**: o cotovelo aparece do **outro** lado. Medido antes da cura:
/// `-99,498744` → `0,000000` (a recta, que não tem lado) → `+99,498744`.
///
/// ⇒ este gate afirma as duas metades: que [`BendSide::Keep`] ainda inverte (é o comportamento de
/// um GESTO, e mudá-lo seria mudar o arrasto que já existe) e que um lado **travado** não inverte.
#[test]
fn the_elbow_flips_when_the_chain_passes_through_straight() {
    let (mut j, l, alvo) = cotovelo_do_lado_anti_horario();
    reach(&mut j, &l, alvo, Reach::default());
    let antes = dominant_side(&j, alvo);
    reach(&mut j, &l, [40.0, 0.0], Reach::default());
    reach(&mut j, &l, alvo, Reach::default());
    let depois = dominant_side(&j, alvo);
    assert!(
        antes > 0.0 && depois < 0.0,
        "a fixtura tem de PRODUZIR a inversão com `Keep`, senão o gate de baixo é vácuo \
         (antes={antes}, depois={depois})"
    );
}

/// ⭐⭐⭐ **UM LADO TRAVADO SOBREVIVE À RECTA** — a cura, sobre a mesma fixtura.
#[test]
fn a_locked_bend_survives_the_chain_passing_through_straight() {
    let (mut j, l, alvo) = cotovelo_do_lado_anti_horario();
    let opts = Reach {
        bend: BendSide::Ccw,
        ..Reach::default()
    };
    reach(&mut j, &l, alvo, opts);
    let antes = dominant_side(&j, alvo);
    reach(&mut j, &l, [40.0, 0.0], opts);
    reach(&mut j, &l, alvo, opts);
    let depois = dominant_side(&j, alvo);
    assert!(
        antes > 0.0 && depois > 0.0,
        "o lado travado inverteu ao passar pela recta: {antes} -> {depois}"
    );
}

/// ⭐⭐ **O LADO TRAVADO É O QUE SE PEDIU, nas duas leis e em qualquer comprimento de corrente.**
///
/// ⚠️ A régua é o [`side_of`], **não** o `v1 × v2`: aquele existe numa corrente de cinco ossos e
/// este não, e os dois têm sinais opostos — a razão de a porta canónica existir.
#[test]
fn the_locked_side_is_the_one_that_was_asked_for_in_both_laws() {
    for n in [2usize, 3, 5, 8] {
        for (side, quero) in [(BendSide::Ccw, 1.0_f64), (BendSide::Cw, -1.0)] {
            // Parte do lado ERRADO de propósito: é o caso em que o arqueamento devolve cedo e só o
            // espelho morde.
            let (mut j, l) = corrente(n, 10.0);
            #[expect(clippy::cast_precision_loss, reason = "n é um punhado de ossos")]
            let alcance = n as f64 * 10.0;
            let alvo = [alcance * 0.4, alcance * 0.25];
            reach(&mut j, &l, alvo, Reach {
                bend: side.flipped(),
                ..Reach::default()
            });
            reach(&mut j, &l, alvo, Reach {
                bend: side,
                ..Reach::default()
            });
            let deu = dominant_side(&j, alvo);
            assert!(
                deu.signum() == quero.signum(),
                "com {n} ossos e {side:?} a corrente ficou do lado {deu}"
            );
        }
    }
}

/// ⛔ **O ESPELHO NÃO ESTICA NENHUM OSSO** — ele é uma isometria, e o invariante das duas leis não
/// pode depender de eu me lembrar disso.
#[test]
fn locking_the_side_never_stretches_a_bone() {
    for n in [2usize, 3, 5, 8] {
        for side in [BendSide::Keep, BendSide::Ccw, BendSide::Cw] {
            let (mut j, l) = corrente(n, 10.0);
            for alvo in [[15.0, 22.0], [-30.0, 4.0], [1e4, 1e4], [0.5, 0.0]] {
                reach(&mut j, &l, alvo, Reach {
                    bend: side,
                    ..Reach::default()
                });
                for (i, (saiu, pedido)) in comprimentos(&j).iter().zip(&l).enumerate() {
                    assert!(
                        (saiu - pedido).abs() < 1e-9 * pedido,
                        "com {n} ossos, {side:?} e alvo {alvo:?} o osso {i} mede {saiu} e devia \
                         medir {pedido}"
                    );
                }
            }
        }
    }
}

/// ⭐ **`Keep` É O COMPORTAMENTO DE SEMPRE, AO BIT** — o caminho de omissão desta wave não move um
/// único bit de nada que já existia.
#[test]
fn keep_is_the_default_and_it_is_bit_identical_to_having_no_side_at_all() {
    assert_eq!(Reach::default().bend, BendSide::Keep);
    for n in [1usize, 2, 3, 5, 9] {
        for alvo in [[15.0, 22.0], [-30.0, 4.0], [1e4, 1e4], [7.0, 0.0]] {
            let (mut a, l) = corrente(n, 10.0);
            let mut b = a.clone();
            reach(&mut a, &l, alvo, Reach::default());
            reach(&mut b, &l, alvo, Reach {
                bend: BendSide::Keep,
                ..Reach::default()
            });
            assert_eq!(a, b, "com {n} ossos e alvo {alvo:?} o `Keep` divergiu do default");
        }
    }
}

/// ⭐⭐⭐ **UM LADO TRAVADO É ESTÁVEL, NÃO UM PISCA-PISCA.**
///
/// ⚠️ Mede um risco que só esta wave tem: o espelho reflecte a corrente **inteira**, e uma
/// restrição re-resolve **todo quadro a partir da própria saída**. Um espelho que disparasse de
/// novo sobre a pose que ele acabou de produzir poria a corrente a alternar entre dois lados a
/// 60 Hz — e nenhum gate que resolva uma vez o vê.
///
/// ⚠️⚠️ **A régua é o LADO e a CONVERGÊNCIA, não a igualdade ao bit** — e a 1.ª redacção exigia
/// `1e-9` do alcance, o que a fez acusar o FABRIK de vibrar com `2,47e-4` de movimento. Aquilo não
/// era vibração: o FABRIK **sai cedo** quando o erro cai abaixo da tolerância, então a passagem
/// seguinte continua a refinar. *Convergir e alternar são coisas diferentes, e só a segunda é o
/// defeito.*
#[test]
fn a_locked_side_is_stable_not_a_flip_flop() {
    for n in [2usize, 3, 5, 8] {
        for side in [BendSide::Ccw, BendSide::Cw] {
            let (mut j, l) = corrente(n, 10.0);
            #[expect(clippy::cast_precision_loss, reason = "n é um punhado de ossos")]
            let alcance = n as f64 * 10.0;
            let alvo = [alcance * 0.4, alcance * 0.25];
            let opts = Reach {
                bend: side,
                ..Reach::default()
            };
            reach(&mut j, &l, alvo, opts);
            let esperado = dominant_side(&j, alvo).signum();
            let (mut ant, mut primeiro, mut ultimo) = (j.clone(), 0.0_f64, 0.0_f64);
            for k in 0..8 {
                reach(&mut j, &l, alvo, opts);
                let mov = ant
                    .iter()
                    .zip(&j)
                    .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                    .fold(0.0, f64::max);
                if k == 0 {
                    primeiro = mov;
                }
                ultimo = mov;
                assert!(
                    dominant_side(&j, alvo).signum() == esperado,
                    "com {n} ossos a corrente ALTERNOU de lado na passagem {k} — o espelho está a \
                     disparar sobre a pose que ele próprio produziu"
                );
                ant = j.clone();
            }
            assert!(
                ultimo <= primeiro,
                "com {n} ossos e {side:?} o movimento CRESCEU entre passagens ({primeiro} -> \
                 {ultimo}): a pose está a divergir em vez de assentar"
            );
        }
    }
}

/// ⭐ **UM LIMITE LARGO NÃO APARA NADA** — a lei da casa (*todo motor novo é no-op no ponto
/// neutro*), e aqui ela é exacta: uma faixa de uma volta inteira contém todo ângulo.
#[test]
fn a_limit_as_wide_as_the_circle_changes_nothing() {
    for k in -30..=30 {
        let r = f64::from(k) * 0.21;
        let saiu = clamp_to_limit(r, -FULL_TURN / 2.0, FULL_TURN / 2.0);
        assert!(
            (saiu - r).abs() < 1e-12,
            "o limite da volta inteira moveu {r} para {saiu}"
        );
    }
}

/// ⭐⭐ **O LIMITE APARA, e apara para o extremo MAIS PRÓXIMO.**
#[test]
fn a_joint_stops_at_the_edge_it_is_pushed_against() {
    let (min, max) = (-0.5, 1.0);
    assert!((clamp_to_limit(0.3, min, max) - 0.3).abs() < 1e-12, "dentro");
    assert!((clamp_to_limit(2.0, min, max) - max).abs() < 1e-12, "acima");
    assert!((clamp_to_limit(-1.2, min, max) - min).abs() < 1e-12, "abaixo");
}

/// ⭐⭐⭐ **UM INTERVALO QUE ATRAVESSA A MEIA-VOLTA FUNCIONA** — e é aqui que um `clamp` cru estaria
/// errado.
///
/// ⚠️ Com `min = 170°` e `max = −170°` a faixa é de **20°** em torno de `±π`. Um `rot.clamp(min,
/// max)` com `min > max` entra em **pânico** em Rust; e mesmo trocando-os ele devolveria um dos
/// extremos para todo ângulo do meio do círculo, que é o oposto da faixa pedida.
#[test]
fn a_range_that_crosses_the_half_turn_still_holds() {
    use std::f64::consts::PI;
    let (min, max) = (PI - 0.17, PI + 0.17); // 20° a cavalo do ±π
    // Um ângulo DENTRO da faixa, escrito do outro lado do embrulho, fica onde está.
    let dentro = -PI + 0.10;
    let saiu = clamp_to_limit(dentro, min, max);
    assert!(
        wrap_pi(saiu - dentro).abs() < 1e-12,
        "um ângulo dentro da faixa foi movido: {dentro} -> {saiu}"
    );
    // E um bem fora dela é trazido para a borda, não para o meio do círculo.
    let fora = 0.0;
    let saiu = clamp_to_limit(fora, min, max);
    assert!(
        wrap_pi(saiu - min).abs() < 1e-12 || wrap_pi(saiu - max).abs() < 1e-12,
        "um ângulo fora da faixa parou em {saiu}, que não é nenhuma das duas bordas"
    );
}

/// ⛔ **Um intervalo INVERTIDO trava no centro em vez de entrar em pânico.**
///
/// ⚠️ `f64::clamp` com `min > max` **aborta o processo**. Um `.ph2dproj` editado à mão, ou um degrau
/// de migração com os campos trocados, chegaria aqui — e a diferença entre *«a junta ficou presa»* e
/// *«o app fechou»* é a diferença entre um defeito e uma perda de trabalho.
#[test]
fn an_inverted_range_locks_at_the_centre_instead_of_panicking() {
    let saiu = clamp_to_limit(3.0, 1.0, -1.0);
    assert!((saiu - 0.0).abs() < 1e-12, "devia travar no centro, deu {saiu}");
}

/// ⭐⭐ **APARAR DUAS VEZES DÁ O MESMO** — a lei é idempotente, e tem de ser: o solver re-resolve
/// todo quadro a partir da própria saída, e uma aparadela que continuasse a mover a pose poria a
/// junta a **deslizar** para o centro, um pouco por quadro.
#[test]
fn clamping_twice_is_the_same_as_clamping_once() {
    for k in -40..=40 {
        for (min, max) in [(-0.5, 1.0), (2.9, 3.4), (-3.0, 3.0)] {
            let r = f64::from(k) * 0.19;
            let uma = clamp_to_limit(r, min, max);
            let duas = clamp_to_limit(uma, min, max);
            assert!(
                (uma - duas).abs() < 1e-12,
                "aparar {r} deu {uma} e depois {duas} — a lei não é idempotente"
            );
        }
    }
}
