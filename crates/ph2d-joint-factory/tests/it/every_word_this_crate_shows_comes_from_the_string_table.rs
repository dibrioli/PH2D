//! **O HR-15 na crate das peças de fábrica** — viajou com o código quando ele saiu da
//! `ph2d-joint-assembly` (tecto de linhas, 16/11). Os nomes das peças vêm do gabarito; aqui só se compõem
//! sufixos de identidade (` (2)`), que não são língua.

use ph2d_label_census::gate::{self, Excecao};

const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn every_word_this_crate_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte das peças de fábrica (HR-15):\n  {}\n\n\
         A cura é uma chave em `ph2d-i18n` e um `tr(\"…\")` no sítio; se é um NOME de objecto, uma linha \
         em `NOT_LANGUAGE` **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}
