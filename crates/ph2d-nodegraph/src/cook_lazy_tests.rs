//! **As provas do SUBSTRATO da preguiça** — as que faltavam na crate que o possui.
//!
//! ⚠️ **`cook_lazy.rs` shipou com 187 linhas e ZERO testes aqui**, enquanto os dois irmãos
//! nascidos na mesma linha os têm (`cook_scope_tests.rs`, `cook_fan_tests.rs`). Todos os gates
//! viviam em `ph2d-node-value-switch`, o que tem uma consequência exacta: **o comando do fecho
//! (`cargo-test-narrow.sh ph2d-nodegraph`) não provava nada sobre esta feature** — a auditoria
//! de 2026-08-27 correu dez mutações no substrato e as dez sobreviveram a `-p ph2d-nodegraph`.
//!
//! ⚠️ **O que fica aqui é o que só este crate pode responder**: a lei de `uniform_scalar`, que é
//! a primeira das três condições e a única que não depende de conhecer o `value.switch`. As duas
//! cláusulas abaixo eram, das dez, as que **mutação nenhuma matava** — nem lá nem aqui.

use super::*;
use crate::attr::{Column, Stream};

fn field(xs: Vec<f32>) -> CookValue {
    CookValue::Instances(Stream::new(xs.len()).with("v", Column::Scalar(xs)))
}

/// **A REGRA 1→N e o campo vazio** — o caso comum, e o controle de que a lei mede alguma coisa.
#[test]
fn a_broadcast_or_absent_field_is_uniform_and_a_varying_one_is_not() {
    assert_eq!(uniform_scalar(&CookValue::Empty, "v"), Some(0.0));
    assert_eq!(uniform_scalar(&field(vec![]), "v"), Some(0.0));
    assert_eq!(uniform_scalar(&field(vec![2.0]), "v"), Some(2.0));
    assert_eq!(uniform_scalar(&field(vec![2.0, 2.0, 2.0]), "v"), Some(2.0));
    assert_eq!(uniform_scalar(&field(vec![2.0, 3.0]), "v"), None);
    // Sem a coluna pedida, o nó lê o campo vazio.
    assert_eq!(uniform_scalar(&field(vec![1.0, 9.0]), "outra"), Some(0.0));
}

/// ⛔⛔ **A CLÁUSULA DO `NaN`: compara por BITS, não por `==`.**
///
/// O doc dela tem um parágrafo inteiro (*"dois `NaN` não são iguais, e um campo inteiro de `NaN`
/// é tão uniforme quanto um de zeros"*) e **nenhuma mutação a matava** — trocar `to_bits() ==`
/// por `==` deixava a suíte verde dos dois lados (auditoria de 2026-08-27).
///
/// ⚠️ O `-0.0` é a outra metade, e é a que mostra que a régua é mesmo de BITS: `-0.0 == 0.0` é
/// verdadeiro e os bits diferem. *Um campo que mistura os dois zeros não é uniforme para esta
/// lei, e isso é deliberado — o valor que sairia dali decidiria um ramo.*
#[test]
fn a_field_of_nans_is_uniform_and_the_two_zeros_are_not_the_same_bits() {
    let n = f32::NAN;
    assert!(
        uniform_scalar(&field(vec![n, n, n]), "v").is_some_and(f32::is_nan),
        "um campo inteiro de NaN e' uniforme — recusa-lo mandaria cozinhar tudo por causa de \
         uma aritmetica ja' partida a montante"
    );
    assert_eq!(
        uniform_scalar(&field(vec![n, 1.0]), "v"),
        None,
        "um NaN misturado com um numero NAO e' uniforme"
    );
    assert_eq!(
        uniform_scalar(&field(vec![0.0, -0.0]), "v"),
        None,
        "a regua e' de BITS: `-0.0 == 0.0` e' verdade e os bits diferem"
    );
}

/// ⛔⛔ **A CERCA DO `Opaque`, e ela NÃO falha para o lado seguro.**
///
/// Um payload opaco é apagado pelo tipo: não há escalar para ler. `None` = *não sei* = coza tudo.
/// ⚠️ **A mutação que a auditoria provou a sobreviver era `Opaque => Some(0.0)`**, e ela é pior
/// que um recuo: faria a preguiça **rotear como se fosse o ramo 0** um valor que ela não
/// consegue ver. *Uma cerca que devolve um palpite em vez de um «não sei» inverte o sentido do
/// recuo.*
#[test]
fn an_opaque_select_is_not_a_zero_it_is_an_unknown() {
    let v = CookValue::Opaque(std::sync::Arc::new(7_u32));
    assert_eq!(
        uniform_scalar(&v, "v"),
        None,
        "um valor opaco tem de recuar para «coza tudo», nunca para o ramo 0"
    );
}
