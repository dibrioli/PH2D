//! ⭐⭐⭐ **NENHUMA PALAVRA DE TIMELINE É ESCRITA NO FONTE** — o HR-15, por crate.
//!
//! Em 2026-09-16 o painel ainda escrevia 2 textos no fonte (os chips do buffer do grafo); as chaves moram no `lib.rs` da tabela. O corpo do gate é o [`ph2d_label_census::gate`]; aqui ficam a LISTA desta crate e as
//! mensagens.

use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "panel.timeline.";
const TABLE: &str = "crates/ph2d-i18n/src/timeline.rs";
/// ⚠️⚠️ **SÃO DUAS tabelas desde a integração de 2026-09-20**, e o `lib.rs` fica na lista de
/// propósito: as `panel.timeline.*` saíram dele para o irmão `timeline.rs` por um tecto de LOC que
/// só a árvore COMBINADA acendeu (o `main` estava em `696` de `700` e a `line/Vector` pôs-lhe as
/// cinco chaves das alças do osso) — e uma chave escrita de volta no `lib.rs` amanhã tem de
/// continuar a contar.
///
/// ⭐ **Quem apanhou o corte foi o CONTROLO DE VACUIDADE abaixo**, que leu `0` declaradas: *um censo
/// que lê a tabela por CAMINHO fica verde a medir o nada quando ela muda de ficheiro*, e o piso é a
/// única metade dele que sabe a diferença entre «esta crate não declara chaves» e «eu perdi-as».
const TABLES: &[&str] = &[TABLE, "crates/ph2d-i18n/src/lib.rs"];

/// ⭐ As excepções, **com o mecanismo**.
const NOT_LANGUAGE: &[Excecao] = &[];

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
    let c = gate::chaves(&repo, PREFIX, TABLES);
    // ⛔ Controlo de vacuidade: o vocabulário medido na migração, menos folga para encolher.
    assert!(
        c.declaradas >= 40 && c.usadas >= 40,
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
