//! ⭐⭐ **A ORDEM do parser de unidades é a LEI — `m/s` não pode ler-se como `s`.**
//!
//! Em 2026-09-14 o [`Unit`] ganhou `Seconds` (`"s"`) e `MetersPerSecond` (`"m/s"`) porque o censo
//! dos rótulos do Inspector mediu **oito** campos em segundos e **onze** em metros por segundo.
//! ⛔ E o sufixo de um **contém** o do outro: `"5m/s"` termina em `"s"`.
//!
//! ⚠️⚠️ **O modo de falha é MUDO:** com a ordem trocada, o parser devolve o número CERTO e a
//! unidade ERRADA — nada estoura, nada avisa, e a velocidade de um personagem entra no documento
//! como um tempo.
//!
//! ⚠️ **O gate deriva a ordem de `Unit::ALL`**, que é a lista que o parser percorre. Uma cópia
//! escrita aqui provaria que a cópia está ordenada — *um gate que compara duas construções é cego
//! a uma mutação partilhada*, e uma cópia é a construção mais fácil de esquecer.

use ph2d_editor_core::widget::Unit;

/// ⭐ **Nenhum sufixo é testado depois de outro que o contém.**
#[test]
fn a_longer_suffix_is_never_shadowed_by_a_shorter_one() {
    let ordem = Unit::ALL;
    assert!(ordem.len() >= 8, "a lista encolheu: {}", ordem.len());
    for (i, a) in ordem.iter().enumerate() {
        for b in &ordem[i + 1..] {
            // `b` é testado DEPOIS de `a`. Se o sufixo de `b` terminar com o de `a`, todo valor de
            // `b` é apanhado por `a` primeiro — e `b` fica inalcançável.
            assert!(
                !b.suffix().ends_with(a.suffix()),
                "`{}` ({:?}) e' testado depois de `{}` ({:?}), que e' o fim dele — \
                 todo valor em `{}` seria lido como `{}`, com o numero certo e a unidade errada",
                b.suffix(),
                b,
                a.suffix(),
                a,
                b.suffix(),
                a.suffix()
            );
        }
    }
}

/// ⭐ **E o oráculo: o que o artista escreve volta como o que ele escreveu.**
///
/// ⚠️ A lista é de PARES esperados, não uma varredura do enum: ela é o que um humano leria no
/// campo, e é ela que apanha uma unidade nova cujo sufixo colida com uma antiga.
#[test]
fn what_the_artist_types_comes_back_as_what_they_typed() {
    let casos: &[(&str, f64, Option<Unit>)] = &[
        ("5m/s", 5.0, Some(Unit::MetersPerSecond)),
        ("9.8m/s2", 9.8, Some(Unit::MetersPerSecondSquared)),
        ("5 m/s", 5.0, Some(Unit::MetersPerSecond)),
        ("0.25s", 0.25, Some(Unit::Seconds)),
        ("1.5 m", 1.5, Some(Unit::Meters)),
        ("90deg", 90.0, Some(Unit::Degrees)),
        ("1.57rad", 1.57, Some(Unit::Radians)),
        ("12 px", 12.0, Some(Unit::Px)),
        ("50%", 50.0, Some(Unit::Percent)),
        ("42", 42.0, None),
    ];
    for (texto, valor, unidade) in casos {
        let lido = ph2d_editor_core::widget::parse_numeric_with_unit(texto)
            .unwrap_or_else(|| panic!("`{texto}` nao foi lido"));
        assert!(
            (lido.0 - valor).abs() < 1e-9,
            "`{texto}`: valor {} em vez de {valor}",
            lido.0
        );
        assert_eq!(&lido.1, unidade, "`{texto}`: unidade errada");
    }
}
