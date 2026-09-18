//! ⭐⭐⭐ **NENHUMA PALAVRA DO PAINEL EQUALIZE SIZES É ESCRITA NO FONTE** — o HR-15, por crate.
//!
//! Migrado em 2026-09-16 (`scripts/migrar-texto-pintado.py`, mapa
//! `docs/UI_New_and_Simple/ferramentas/seccoes_equalize_sizes.tsv`): o painel escrevia **17** textos no fonte, medidos
//! pela régua da `ph2d-label-census`, e nem dependia da `ph2d-i18n`. O corpo do gate é o
//! [`ph2d_label_census::gate`]; aqui ficam a LISTA desta crate e as mensagens.

use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "panel.equalize_sizes.";
const TABLE: &str = "crates/ph2d-i18n/src/image_tools.rs";

/// ⭐ As excepções, **com o mecanismo**.
const NOT_LANGUAGE: &[Excecao] = &[];

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
        c.declaradas >= 12 && c.usadas >= 12,
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

/// ⭐⭐⭐ **AS LETRAS SOLTAS — o que a régua lexical não pode ver, e por isso precisa de gate.**
///
/// ⛔⛔ **Medido em 2026-09-18:** o gate acima estava VERDE e os dois chips do modo `Fixed`
/// pintavam `"W"` e `"H"` escritos no fonte. A cegueira é **por construção**: o
/// [`ph2d_label_census::is_language`] exige duas letras SEGUIDAS, senão acusaria todo
/// identificador. *Uma letra sozinha não tem forma que a distinga de um — quem a distingue é o
/// TIPO*, e o `paint_labeled_chip` passou a receber [`ph2d_i18n::TextKey`].
///
/// Este gate dá a metade que o tipo não pode dar: **a chave existe na tabela**. A população é
/// LIDA do pintor, nunca escrita outra vez aqui.
#[test]
fn cada_letra_solta_deste_painel_vem_da_tabela() {
    const PINTOR: &str = include_str!("../../src/paint.rs");
    let chaves: Vec<&str> = PINTOR
        .split("TextKey::new(\"")
        .skip(1)
        .filter_map(|p| p.split('"').next())
        .collect();
    // ⛔ Piso de população: o modo `Fixed` tem DOIS chips (largura e altura).
    assert_eq!(
        chaves.len(),
        2,
        "o pintor declara {} chave(s) de letra e os chips do `Fixed` são dois: {chaves:?}",
        chaves.len()
    );
    let cruas: Vec<&str> = chaves
        .iter()
        .copied()
        .filter(|k| ph2d_i18n::tr(k) == *k)
        .collect();
    assert!(
        cruas.is_empty(),
        "estas chaves de LETRA não existem na tabela — o chip pinta o identificador: {cruas:?}"
    );
}
