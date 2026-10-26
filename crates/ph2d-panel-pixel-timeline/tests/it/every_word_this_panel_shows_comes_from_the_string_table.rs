//! **NENHUMA PALAVRA DA LINHA DO TEMPO DO PIXEL É ESCRITA NO FONTE** — o HR-15, com a régua partilhada
//! ([`ph2d_label_census`]); só a lista, o prefixo e as excepções são desta crate.
use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "panel.pixel.timeline.";
const TABLES: &[&str] = &["crates/ph2d-i18n/src/pixel.rs"];

/// ⭐ As excepções, **com o mecanismo** — nunca uma lista aberta.
const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte (HR-15):\n  {}\n\nA cura é uma \
         chave `{PREFIX}<nome>` em {TABLES:?} e um `tr(\"…\")` no sítio.",
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

/// O censo dos DOIS lados: usada-e-não-declarada pinta o identificador cru; declarada-e-não-usada
/// é uma órfã.
#[test]
fn every_key_of_this_panel_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, PREFIX, TABLES);
    // ⛔ Controlo de vacuidade: um caminho errado dá dois conjuntos vazios, que concordam sempre.
    assert!(
        c.declaradas >= 30 && c.usadas >= 30,
        "o censo achou {} declaradas e {} usadas — o piso é 30",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas em {TABLES:?}:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(c.orfas.is_empty(), "declaradas e sem uso:\n  {:?}", c.orfas);
}
