//! ⭐ **NENHUMA PALAVRA DAS CONSTRAINTS É ESCRITA NO FONTE** — o HR-15 na crate da família. As cenas
//! de smoke (`smoke.rs`) são DADOS da cena (nomes de objeto) e ficam fora pela régua.

use ph2d_label_census::gate;

const TABLE: &str = "crates/ph2d-i18n/src/constraints.rs";

#[test]
fn every_word_this_family_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, &[], &[], gate::CENAS);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte das constraints (HR-15):\n  {}\n\n\
         A cura é uma chave `constraints.<assunto>.<frase>` em `{TABLE}` e um `tr(\"…\")` no sítio.",
        intrusos.join("\n  ")
    );
}
