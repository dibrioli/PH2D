//! ⭐⭐⭐ **NENHUMA PALAVRA DO MOTOR DO PINCEL É ESCRITA NO FONTE** — o HR-15, no 2.º motor.
//!
//! ⛔⛔ **O mesmo defeito do catálogo de componentes, noutra crate:** o `ph2d-panel-painter-layers`
//! tem censo e fecha **VERDE** enquanto os nomes que ele pinta — os `24` modos de mistura, as `11`
//! quedas, os `29` tipos de pincel, as amarrações de textura — eram literais aqui, numa crate **sem
//! um único teste de texto**. *Um censo cuja crate não é DONA do texto que ela pinta fica verde
//! sobre texto cru.*
//!
//! # ⚠️ O que este censo NÃO afirma
//!
//! Ele não diz que tudo o que parece língua nesta crate é UI. Diz que o que sobra tem de ter um
//! nome e um mecanismo ao lado — e a lista abaixo é essa, não uma tolerância.

use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "paint_brush.";
const TABLE: &str = "crates/ph2d-i18n/src/paint_engines.rs";

/// ⭐ As excepções, **com o mecanismo**.
const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn every_word_this_engine_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte do motor e nunca chegam à tabela \
         (HR-15):\n  {}\n\nA cura é a chave `{PREFIX}<enum>.<variante>` em `{TABLE}`, e o painel a \
         chamar `tr(…)` — nunca um literal a atravessar a fronteira.",
        intrusos.join("\n  ")
    );
}

#[test]
fn every_named_exception_still_shelters_a_real_literal() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas = gate::excecoes_mortas(&src, NOT_LANGUAGE);
    assert!(
        mortas.is_empty(),
        "excepções mortas:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐ **As duas pontas.** Uma chave em falta pinta o identificador cru **numa fileira de chips de
/// modo de mistura** — e o `tr` de uma chave desconhecida faz `leak_key` por quadro.
#[test]
fn every_key_of_this_engine_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, PREFIX, &[TABLE]);
    assert!(
        c.declaradas >= 70 && c.usadas >= 70,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas em `{TABLE}`:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "declaradas e ninguém as usa — uma variante morreu e deixou a palavra dela:\n  {:?}",
        c.orfas
    );
}
