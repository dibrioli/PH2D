//! **OS GATES DO RELÓGIO** — o que segura o campo e o que o faz andar.
//!
//! ⚠️ Cortado do `lib_tests.rs` pelo teto de LOC (700) e por assunto: lá mora a lei do passo e
//! o pino de Dirichlet, aqui a pergunta *"o relógio andou?"* — que é a única coisa que a
//! comparação de `sim_t` compra, e onde um epsilon custou uma mutação sobrevivente.

use super::*;

/// **Um relógio que não andou SEGURA o campo** — o mesmo tique, o wrap de um laço, e um scrub
/// para TRÁS. É o que a comparação de relógio de facto compra, e a única coisa que ela compra.
///
/// ⚠️ **O `MIN_STEP = 1e-6` MORREU aqui** (auditoria de 2026-08-27): a mutação `1e-6 → 1e-2`
/// sobrevivia — só a existência da guarda estava gateada. `t_prev` é um `playhead` anterior,
/// logo os dois `f32` são o mesmo valor ou diferem por ≥1 ULP: `playhead > t_prev` é exacto.
#[test]
fn a_clock_that_did_not_advance_holds_the_field() {
    let p = params(11, 11, 0.4, 0.0);
    let mut state = Stream::new(0);
    for k in 0..6 {
        state = simulate((k == 1).then_some(1.0), &state, &[], k as f32 / 60.0, &p);
    }
    let before = scalar_col(&state, "wave_h");
    // O MESMO instante outra vez…
    let same = simulate(None, &state, &[], 5.0 / 60.0, &p);
    assert_eq!(before, scalar_col(&same, "wave_h"), "o mesmo tique segura");
    // …e um instante ANTERIOR (um scrub para trás).
    let back = simulate(None, &state, &[], 1.0 / 60.0, &p);
    assert_eq!(
        before,
        scalar_col(&back, "wave_h"),
        "um scrub para tras segura"
    );
    // ⚠️ O CONTROLE: um instante seguinte AVANÇA — senão isto mediria um nó parado.
    let ahead = simulate(None, &state, &[], 6.0 / 60.0, &p);
    assert_ne!(
        before,
        scalar_col(&ahead, "wave_h"),
        "o tique seguinte avanca"
    );
}

/// ⛔⛔ **UM AVANÇO DE UM ULP JÁ É UM AVANÇO — é isto que pina a AUSÊNCIA de epsilon.**
///
/// ⚠️ **O gate irmão acima não conseguia ver a diferença**, e a auditoria de 2026-08-27 mediu-o:
/// a fixtura dele anda `1/60 ≈ 0,0167` por tique, então repor um epsilon de `1e-2` **sobrevivia**
/// — ele é menor que o passo da fixtura e não muda nada ali. *Uma fixtura cujo passo é maior que
/// o epsilon não pode medir o epsilon.*
///
/// `t_prev` é um `playhead` anterior carimbado pelo próprio nó, logo os dois `f32` ou são o mesmo
/// valor ou diferem por ≥1 ULP. Este gate encena exactamente o segundo caso: qualquer constante
/// absoluta maior que zero o reprova.
#[test]
fn one_ulp_of_clock_is_already_an_advance() {
    let p = params(11, 11, 0.4, 0.0);
    let mut state = Stream::new(0);
    for k in 0..6 {
        state = simulate((k == 1).then_some(1.0), &state, &[], k as f32 / 60.0, &p);
    }
    let before = scalar_col(&state, "wave_h");
    let t = 5.0f32 / 60.0;
    let one_ulp = f32::from_bits(t.to_bits() + 1);
    assert!(
        one_ulp > t && one_ulp - t < 1e-7,
        "a fixtura tem de ser 1 ULP"
    );
    let stepped = simulate(None, &state, &[], one_ulp, &p);
    assert_ne!(
        before,
        scalar_col(&stepped, "wave_h"),
        "um avanco de 1 ULP foi engolido — ha' um epsilon absoluto no caminho, e ele so' \
         cresce em importancia a medida que o playhead cresce (o ULP a 100 s ja' e' 7,6e-6)"
    );
}
