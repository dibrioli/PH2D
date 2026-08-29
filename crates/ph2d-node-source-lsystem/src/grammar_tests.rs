//! Gates da **gramática** — o que uma linha de texto vira.

use super::*;

fn one(src: &str) -> Rule {
    let r = parse_rules(src);
    assert_eq!(r.len(), 1, "esperava UMA regra de {src:?}, deu {}", r.len());
    r.into_iter().next().unwrap()
}

fn syms(succ: &[SuccModule]) -> String {
    succ.iter().map(|m| m.sym as char).collect()
}

/// A produção mais simples que existe.
#[test]
fn a_plain_production_parses() {
    let r = one("F -> FF");
    assert_eq!(r.pred.sym, b'F');
    assert!(r.pred.formals.is_empty());
    assert_eq!(syms(&r.succ), "FF");
    assert!(r.left.is_none() && r.right.is_none() && r.cond.is_none());
    assert_eq!(r.weight, 1.0);
}

/// **Paramétrica**: os nomes do predecessor são FORMAIS, os do sucessor são EXPRESSÕES.
#[test]
fn a_parametric_production_binds_formals_and_compiles_argument_expressions() {
    let r = one("A(x) -> F(x)[+A(x*0.7)]");
    assert_eq!(r.pred.formals, vec!["x".to_string()]);
    assert_eq!(syms(&r.succ), "F[+A]");
    assert_eq!(r.succ[0].args.len(), 1, "o F leva um argumento");
    assert_eq!(r.succ[3].args.len(), 1, "o A leva um argumento");
    // ⚠️ E ele é uma EXPRESSÃO, não a constante que um parser preguiçoso devolveria.
    assert!(
        !matches!(r.succ[3].args[0], ph2d_expr::Expr::Const(_)),
        "x*0.7 tem de compilar para uma expressao, deu {:?}",
        r.succ[3].args[0]
    );
}

/// **Contexto dos dois lados** (2L).
#[test]
fn a_two_sided_context_parses() {
    let r = one("A < B > C -> D");
    assert_eq!(r.left.as_ref().map(|p| p.sym), Some(b'A'));
    assert_eq!(r.pred.sym, b'B');
    assert_eq!(r.right.as_ref().map(|p| p.sym), Some(b'C'));
    assert_eq!(syms(&r.succ), "D");
}

/// ⭐ **O `>` de uma CONDIÇÃO não é lido como contexto** — a ambiguidade que a ordem de
/// análise resolve (o contexto vive só à ESQUERDA do primeiro `:`).
///
/// Sem isto, `A(x) : x > 0.1 -> F` sairia com um contexto direito chamado `0` e a condição
/// truncada — a regra deixaria de casar e o ramo simplesmente não cresceria, sem uma palavra.
#[test]
fn a_condition_with_a_greater_than_is_not_read_as_a_context() {
    let r = one("A(x) : x > 0.1 -> F");
    assert!(
        r.right.is_none(),
        "o `>` da condicao virou contexto: {:?}",
        r.right
    );
    assert!(r.cond.is_some(), "a condicao tem de compilar");
    assert_eq!(r.pred.sym, b'A');
    assert_eq!(r.pred.formals, vec!["x".to_string()]);
}

/// ⭐ **O peso vem depois da seta e NÃO é um módulo.**
///
/// A posição é o que o desambigua: um sucessor é uma sequência de módulos, e um módulo nunca
/// COMEÇA por `(`. Sem esta leitura, `F -> (0.4) FF` daria um sucessor de três módulos, um
/// deles a letra `0`.
#[test]
fn a_leading_weight_is_a_weight_and_never_a_module() {
    let r = one("F -> (0.4) F[+F]F");
    assert!((r.weight - 0.4).abs() < 1e-6, "peso {}", r.weight);
    assert_eq!(syms(&r.succ), "F[+F]F");
}

/// Várias regras numa linha só, separadas por `;` — que é o formato, não o gosto (um `\n`
/// corrompe o ficheiro do projeto).
#[test]
fn rules_are_separated_by_semicolons_on_one_line() {
    let rs = parse_rules("F -> FF ; X -> F[+X]F[-X]+X ; A -> B");
    assert_eq!(rs.len(), 3);
    assert_eq!(
        rs.iter().map(|r| r.pred.sym as char).collect::<String>(),
        "FXA"
    );
}

/// ⚠️ **Uma regra malformada é descartada e as outras SOBREVIVEM.**
///
/// É o estado normal de quem está a autorar: a segunda regra está a meio de ser escrita
/// enquanto a primeira já funciona. Recusar a gramática inteira apagaria a planta a cada
/// tecla.
#[test]
fn a_malformed_rule_is_dropped_and_the_others_survive() {
    let rs = parse_rules("F -> FF ; isto nao e uma regra ; X -> Y");
    assert_eq!(
        rs.iter().map(|r| r.pred.sym as char).collect::<String>(),
        "FX",
        "a do meio nao tem seta e cai; as outras ficam"
    );
}

/// O tecto de argumentos por módulo é aplicado no PARSE — ver [`MAX_ARGS`] para de que
/// recurso ele é.
#[test]
fn a_module_never_carries_more_arguments_than_the_budget() {
    let r = one("A -> F(1,2,3,4,5,6,7)");
    assert_eq!(r.succ[0].args.len(), MAX_ARGS);
}

/// Um contexto pode ele próprio ser paramétrico (ABOP §1.10.2) — os formais dele entram nas
/// ligações da regra.
#[test]
fn a_context_can_carry_its_own_formals() {
    let r = one("A(a) < B(b) -> F(a+b)");
    assert_eq!(r.left.as_ref().unwrap().formals, vec!["a".to_string()]);
    assert_eq!(r.pred.formals, vec!["b".to_string()]);
}
