//! ⭐⭐⭐ **A CERCA CONTRA O LAÇO DO RECOMEÇO** — *a corrida tem de ter CORRIDO*.
//!
//! # ⛔⛔ O defeito que ela impede, e porque ele é MUDO
//!
//! O [`ph2d_ecs::SignalVerb::RestartRun`] rebobina a corrida. Uma condição que **já é verdade
//! quando a corrida começa** — um `Counter Watch` escrito `pontos AtLeast 0`, que é fácil de
//! escrever por engano — pede o recomeço em **todo** quadro: o relógio nunca passa do primeiro
//! tique, nada avança, e o dono vê um app **congelado sem uma linha de erro**.
//!
//! É a mesma classe do laço `a → b → a` que o doc do `SignalActions` recusa por escrito, com o
//! **relógio** no lugar do sinal.
//!
//! # ⭐ E o número é DERIVADO, não escolhido
//!
//! A unidade de uma corrida é o **passo fixo**. Uma corrida cuja vida inteira é o tique que acabou
//! de andar não é uma corrida — e é essa, exactamente, a assinatura do laço. ⛔ Um segundo
//! escolhido, ou um contador de recomeços por janela, seriam palpites à espera de um smoke.

use super::a_corrida_ja_correu;

/// O passo fixo desta casa, em segundos.
const PASSO: f64 = 1.0 / 60.0;

/// ⭐⭐⭐ **As duas metades: o laço é recusado, e o jogo legítimo passa.**
///
/// ⚠️ **Sem a segunda, uma cerca que recusasse SEMPRE passaria** — e o verbo inteiro ficaria morto
/// com o gate verde. *É a metade que separa «a cerca funciona» de «a cerca apagou a feature».*
///
/// **Mutações que devem sangrar:** `vida >= passo` · `true` · `false`.
#[test]
fn a_cerca_recusa_o_laco_e_deixa_passar_o_jogo() {
    // ⛔ O LAÇO: a corrida inteira é o tique que acabou de andar.
    assert!(
        !a_corrida_ja_correu(PASSO, PASSO),
        "um recomeco pedido no primeiro tique e' o laco, e servi-lo congela o app"
    );
    assert!(
        !a_corrida_ja_correu(0.0, PASSO),
        "uma corrida que ainda nao andou nao tem o que rebobinar"
    );
    // ⭐ O JOGO: o dono jogou e perdeu.
    assert!(
        a_corrida_ja_correu(3.0, PASSO),
        "tres segundos de jogo sao uma corrida — a cerca apagou a feature"
    );
    // ⚠️ E a fronteira é **estrita**: dois tiques já são uma corrida.
    assert!(a_corrida_ja_correu(2.0 * PASSO, PASSO));
}

/// ⚠️ **E ela é do PASSO e não de um segundo** — a régua muda com o passo, que é o que torna o
/// número derivado em vez de escolhido.
///
/// **Mutação que deve sangrar:** cravar `vida > 1.0 / 60.0` em vez de ler o passo.
#[test]
fn a_cerca_le_o_passo_e_nao_um_numero() {
    let passo_lento = 1.0 / 10.0;
    assert!(
        !a_corrida_ja_correu(passo_lento, passo_lento),
        "com um passo de 100 ms, 100 ms de vida continuam a ser UM tique"
    );
    // ⛔ E o mesmo relógio, com o passo de sempre, JÁ é uma corrida.
    assert!(a_corrida_ja_correu(passo_lento, PASSO));
}
