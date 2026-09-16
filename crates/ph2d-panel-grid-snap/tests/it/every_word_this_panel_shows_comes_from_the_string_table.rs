//! ⭐⭐⭐ **NENHUMA PALAVRA DO PAINEL GRID SETTINGS É ESCRITA NO FONTE** — o HR-15, por crate.
//!
//! Migrado em 2026-09-16 (`scripts/migrar-texto-pintado.py`, mapa
//! `docs/UI_New_and_Simple/ferramentas/seccoes_grid_snap.tsv`): o painel escrevia **87** textos no fonte (~20 deles com o sufixo da unidade no nome, que a spec §7 manda para dentro da caixa), medidos
//! pela régua da `ph2d-label-census`, e nem dependia da `ph2d-i18n`. O corpo do gate é o
//! [`ph2d_label_census::gate`]; aqui ficam a LISTA desta crate e as mensagens.

use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "panel.grid_snap.";
const TABLE: &str = "crates/ph2d-i18n/src/grid_snap.rs";

/// ⭐ As excepções, **com o mecanismo**.
const NOT_LANGUAGE: &[Excecao] = &[(
    "lib.rs",
    "Grid Settings",
    "o `Panel::TITLE` e' um `const &'static str` que o registo le para a ABA, e o `tr` nao e' \
     `const fn` -- a mesma excepcao dos paineis Painter, Vector e Inspector.",
)];

#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte do painel e nunca chegam à tabela \
         de strings (HR-15):\n  {}\n\nA cura é uma chave `{PREFIX}<secção>.<nome>` em `{TABLE}` \
         e um `tr(\"…\")` no sítio (uma frase com peças do código: `tr_with`).",
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
fn every_key_of_this_panel_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, PREFIX, &[TABLE]);
    // ⛔ Controlo de vacuidade: o vocabulário medido na migração, menos folga para encolher.
    assert!(
        c.declaradas >= 60 && c.usadas >= 60,
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
