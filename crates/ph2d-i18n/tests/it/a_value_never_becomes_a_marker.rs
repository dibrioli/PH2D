//! ⛔⛔ **Um VALOR nunca vira marcador** — o `tr_with` substitui a frase numa passagem só.
//!
//! Medido em 2026-09-16, quando a moldura inteira passou à tabela e começaram a atravessar o
//! `tr_with` nomes que o ARTISTA escreve (o prefab aberto, o ficheiro largado no canvas, a peça de
//! uma cópia). A 1.ª redacção substituía marcador a marcador, em sequência: um prefab chamado
//! `{follows}` entrava pelo `{name}` e era reescrito pelo `{follows}` seguinte — a barra dizia
//! *Editing prefab “3 copies follow” — 3 copies follow*. ⚠️ *Um nome é texto do artista, e o app não
//! o reescreve* (a mesma lei do `provenance` do cartão de instância).

use ph2d_i18n::tr_with;

#[test]
fn a_name_that_looks_like_a_marker_is_written_verbatim() {
    let got = tr_with(
        "chrome.prefab.editing",
        &[("name", &"{follows}"), ("follows", &"3 copies follow")],
    );
    assert_eq!(
        got, "Editing prefab \u{201c}{follows}\u{201d} \u{2014} 3 copies follow",
        "o nome do artista foi reescrito pelo marcador seguinte"
    );
}

/// ⭐ **O controlo** — a frase normal continua a sair igual, e um marcador SEM valor continua
/// escrito (é assim que o erro se vê na tela, e a doc do `tr_with` promete-o).
#[test]
fn the_ordinary_sentence_and_the_missing_marker_are_unchanged() {
    assert_eq!(
        tr_with(
            "chrome.canvas.drop_many",
            &[("count", &3), ("name", &"a.png")]
        ),
        "Drop to import 3 files (first: a.png)"
    );
    assert_eq!(
        tr_with("chrome.canvas.drop_many", &[("count", &3)]),
        "Drop to import 3 files (first: {name})"
    );
    // uma chaveta solta no modelo não é marcador, e um valor com chavetas passa inteiro
    assert_eq!(tr_with("chrome.hud.sprites", &[("n", &"{x")]), "{x sprites");
}
