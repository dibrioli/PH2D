//! **NENHUMA PALAVRA DESTAS JANELAS É ESCRITA NO FONTE** — o HR-15, com a régua partilhada.
use ph2d_label_census::gate::{self, Excecao};

const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte (HR-15):\n  {}",
        intrusos.join("\n  ")
    );
}
