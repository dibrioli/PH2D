//! ⭐⭐ **NENHUMA PALAVRA DESTE PAINEL É ESCRITA NO FONTE** — o HR-15, com a régua que vê a crate
//! inteira (os pincéis, os passes e os botões da escultura).
//!
//! ⚠️ **Esta crate NUNCA teve régua** até 2026-09-17: ela era uma das OITO crates de UI sem o
//! `every_word_this_panel_shows_comes_from_the_string_table` — *um censo que não corre sobre uma
//! crate não afirma nada sobre ela*, e é ali que o próximo literal nasce calado. A régua é a folha
//! partilhada ([`ph2d_label_census`]); só a lista, o prefixo e as excepções são desta crate.
//!
//! ⭐ Ela já estava a **ZERO** quando a régua chegou — o que faltava era quem a mantivesse assim.
use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "panel.sculpt3d.";
const TABLES: &[&str] = &["crates/ph2d-i18n/src/sculpt3d.rs"];

/// ⭐ As excepções, **com o mecanismo** — nunca uma lista aberta.
const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte e nunca chegam à tabela de \
         strings (HR-15):\n  {}\n\nA cura é uma chave `{PREFIX}<nome>` numa das {TABLES:?} e um \
         `tr(\"…\")` no sítio (uma frase com peças do código: `tr_with`). ⚠️ Se o texto NÃO é \
         língua, a cura é uma linha em `NOT_LANGUAGE` **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}

/// ⭐ A metade justa — e o controlo de vacuidade: uma régua cega não acha as excepções.
#[test]
fn every_named_exception_still_shelters_a_real_literal() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas = gate::excecoes_mortas(&src, NOT_LANGUAGE);
    assert!(
        mortas.is_empty(),
        "excepções sem mecanismo ou sem literal:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐⭐ **UMA CHAVE COM ERRO DE ESCRITA PINTA O IDENTIFICADOR CRU NA TELA** — e vaza uma string por
/// quadro (`leak_key`). O censo é dos DOIS lados: usada-e-não-declarada pinta o identificador;
/// declarada-e-não-usada é uma órfã, que é onde alguém escreve, um dia, uma frase sobre um
/// controlo que já não existe.
#[test]
fn every_key_of_this_panel_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, PREFIX, TABLES);
    // ⛔ Controlo de vacuidade: um caminho errado dá dois conjuntos vazios, que concordam sempre.
    assert!(
        c.declaradas >= 100 && c.usadas >= 100,
        "o censo achou {} declaradas e {} usadas — o piso é 100 e dois conjuntos vazios \
         concordam sempre",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "estas chaves são usadas e NÃO existem em {TABLES:?} — o `tr` faz `leak_key` e pinta o \
         identificador cru:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "estas chaves estão na tabela e ninguém as usa — apague-as:\n  {:?}",
        c.orfas
    );
}
