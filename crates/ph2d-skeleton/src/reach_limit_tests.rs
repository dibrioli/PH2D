//! ⭐⭐⭐ **OS GATES DA PAREDE DE UMA JUNTA** (`clamp_to_limit`) — irmão do [`super::reach_tests`]
//! pelo teto de LOC, e coeso pelo assunto: aqui mede-se *«até onde este osso dobra»*, que é uma
//! propriedade da JUNTA (vale com IK e sem ela) e não do alcance.
//!
//! ⚠️ **O gate que importa mais é o da IDENTIDADE EXACTA**: um ângulo já dentro da faixa sai com os
//! MESMOS BITS. O undo desta casa regista por DIFF de bytes, então uma junta parada que fosse
//! normalizada (`-6,3` → `-0,016815`) escreveria um valor novo por quadro — e cada clique passaria
//! a empilhar um passo de undo fantasma.

use super::reach::*;

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
    assert!(
        (clamp_to_limit(0.3, min, max) - 0.3).abs() < 1e-12,
        "dentro"
    );
    assert!((clamp_to_limit(2.0, min, max) - max).abs() < 1e-12, "acima");
    assert!(
        (clamp_to_limit(-1.2, min, max) - min).abs() < 1e-12,
        "abaixo"
    );
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
    assert!(
        (saiu - 0.0).abs() < 1e-12,
        "devia travar no centro, deu {saiu}"
    );
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
