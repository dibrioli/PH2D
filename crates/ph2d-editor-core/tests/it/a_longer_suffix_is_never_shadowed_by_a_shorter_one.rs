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
    assert!(ordem.len() >= 11, "a lista encolheu: {}", ordem.len());
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
        // ⭐⭐ **As três de 2026-09-15, e as duas colisões que elas trazem.**
        ("1200 N", 1200.0, Some(Unit::Newtons)),
        // ⛔ `N.m` acaba em `m`: com a ordem trocada isto lê-se como `Meters`.
        ("35N.m", 35.0, Some(Unit::NewtonMetres)),
        // ⛔ `deg/s` acaba em `s`: com a ordem trocada isto lê-se como `Seconds`.
        ("180deg/s", 180.0, Some(Unit::DegreesPerSecond)),
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

/// ⭐⭐⭐ **O QUE O CAMPO MOSTRA, O CAMPO VOLTA A LER — em qualquer caixa.**
///
/// ⛔⛔ **É a dívida que o `MetersPerSecondSquared` nomeia, e que o `Newtons` cobrou.** Até
/// 2026-09-15 o [`Unit::parse_suffix`] comparava o texto **já em minúsculas** contra o sufixo, logo
/// qualquer sufixo com maiúscula era **inalcançável**: o campo pintava `1200 N` e recusava
/// `1200 N` de volta — *um campo que não sabe ler o que ele próprio escreveu*.
///
/// ⚠️ **A régua varre `Unit::ALL`**, não uma lista escrita à mão: uma unidade nova entra aqui
/// sozinha, e uma que deixe de fazer a ida-e-volta reprova no dia em que nasce.
#[test]
fn every_unit_reads_back_what_it_paints() {
    for u in Unit::ALL {
        for texto in [
            format!("7{}", u.suffix()),
            format!("7 {}", u.suffix()),
            format!("7{}", u.suffix().to_ascii_uppercase()),
            format!("7{}", u.suffix().to_ascii_lowercase()),
        ] {
            let lido = ph2d_editor_core::widget::parse_numeric_with_unit(&texto)
                .unwrap_or_else(|| panic!("`{texto}` ({u:?}) nao foi lido de todo"));
            assert!(
                (lido.0 - 7.0).abs() < 1e-9,
                "`{texto}` ({u:?}): valor {} em vez de 7",
                lido.0
            );
            assert_eq!(
                lido.1,
                Some(u),
                "`{texto}`: o campo pinta `{}` e le' de volta outra unidade",
                u.suffix()
            );
        }
    }
}

/// ⭐⭐ **O CONTROLO** — sem ele, uma comparação que aceitasse QUALQUER coisa passaria nas duas
/// metades acima.
#[test]
fn the_parser_still_refuses_what_is_not_a_unit() {
    for texto in ["7zz", "7 metros", "7N.mm"] {
        let lido = ph2d_editor_core::widget::parse_numeric_with_unit(texto);
        assert!(
            lido.is_none_or(|(_, u)| u.is_none()),
            "`{texto}` foi lido como uma unidade — o parser aceita qualquer cauda"
        );
    }
}
