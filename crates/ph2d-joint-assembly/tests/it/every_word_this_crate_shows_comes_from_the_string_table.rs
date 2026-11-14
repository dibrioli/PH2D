//! **O HR-15 na crate da montagem de juntas** — as isenções viajaram com o código quando ele saiu da
//! `ph2d-app-physics` (corte de LOC do JointEditor, 13/11). Tudo o que sobra são NOMES de objecto, e um
//! nome aqui é identidade dura: a junta reencontra os corpos dela pelo nome (`stable_name_id`), e o hash de
//! replay é fechado sobre o mundo que esses nomes identificam — traduzir `Body` mudaria o hash com a língua.

use ph2d_label_census::gate::{self, Excecao};

/// Um literal isento, com o mecanismo — todos NOMES de objecto.
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "joint_create.rs",
        "Body",
        "o nome por omissão de um corpo sem `Name`, no caminho que cria a junta — identidade durável",
    ),
    (
        "joint_create.rs",
        "{label} Wheel {}",
        "o nome DERIVADO de uma roda nova (`<corpo> Wheel 2`) — identidade, não frase",
    ),
    (
        "joint_world.rs",
        "Body",
        "o mesmo nome por omissão, no pino de mundo — identidade durável",
    ),
    (
        "joint_world.rs",
        "{name_a} : World",
        "o nome DERIVADO de um pino de mundo (`<corpo> : World`) — identidade",
    ),
    (
        "joint_assembly.rs",
        "Body",
        "o mesmo nome por omissão, ao pôr uma peça num papel — identidade durável",
    ),
];

#[test]
fn every_word_this_crate_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte da montagem de juntas (HR-15):\n  {}\n\n\
         A cura é uma chave em `ph2d-i18n` e um `tr(\"…\")` no sítio; se é um NOME de objecto, uma linha \
         em `NOT_LANGUAGE` **com o mecanismo** (um nome aqui entra no hash de replay).",
        intrusos.join("\n  ")
    );
}

#[test]
fn every_named_exemption_still_shelters_what_it_names() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas = gate::excecoes_mortas(&src, NOT_LANGUAGE);
    assert!(
        mortas.is_empty(),
        "isenções mortas:\n  {}",
        mortas.join("\n  ")
    );
}
