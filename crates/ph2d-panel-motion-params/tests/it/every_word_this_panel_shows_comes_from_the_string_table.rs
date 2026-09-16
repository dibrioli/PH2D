//! ⭐⭐⭐ **NENHUMA PALAVRA DE MOTION PARAMS É ESCRITA NO FONTE** — o HR-15, por crate.
//!
//! Migrado em 2026-09-16 (mapa `docs/UI_New_and_Simple/ferramentas/seccoes_motion_params.tsv`): 14 textos no fonte, mais 4 de um ficheiro de TESTE que a régua lia como produção (um comentário entre o `#[cfg(test)]` e o `#[path]`, curado na régua). O corpo do gate é o [`ph2d_label_census::gate`]; aqui ficam a LISTA desta crate e as
//! mensagens.

use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "panel.motion_params.";
const TABLE: &str = "crates/ph2d-i18n/src/motion_panels.rs";

/// ⭐ As excepções, **com o mecanismo**.
const NOT_LANGUAGE: &[Excecao] = &[(
    "lib.rs",
    "Motion Params",
    "o `Panel::TITLE` e' um `const &'static str` que o registo le para a ABA, e o `tr` nao e' `const fn` -- a mesma excepcao dos outros paineis migrados.",
)];

#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte e nunca chegam à tabela de \
         strings (HR-15):\n  {}\n\nA cura é uma chave `{PREFIX}<secção>.<nome>` em `{TABLE}` e \
         um `tr(\"…\")` no sítio (uma frase com peças do código: `tr_with`).",
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

#[test]
fn every_key_of_this_crate_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, PREFIX, &[TABLE]);
    // ⛔ Controlo de vacuidade: o vocabulário medido na migração, menos folga para encolher.
    assert!(
        c.declaradas >= 10 && c.usadas >= 10,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas em `{TABLE}` — o `tr` pinta o identificador cru:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "declaradas e ninguém as usa — apague-as:\n  {:?}",
        c.orfas
    );
}
