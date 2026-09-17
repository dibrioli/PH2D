//! ⭐⭐⭐ **NENHUMA PALAVRA DA FÍSICA É ESCRITA NO FONTE** — o HR-15 na crate de família.
//!
//! Medido em 2026-09-16: **9** textos fora das cenas de smoke — as cinco recusas de DESENHAR uma
//! junta (*«A joint binds two DIFFERENT bodies»*), o aviso de uma junta que partiu, o rótulo do
//! corpo dono no cartão do Inspector e as duas palavras do **leitor de carga** desenhado no canvas
//! (`no route`, `max {force}`). Migrados para `ph2d-i18n/src/app_physics.rs`.
//!
//! ⛔ **O resto são NOMES**, e nomes aqui são identidade dura: uma junta encontra os corpos dela
//! **pelo nome** (`stable_name_id`), e o `physics_ecs_c9` — o hash de replay que corre na matriz de
//! três sistemas — é fechado sobre o mundo que esses nomes identificam. Traduzir `Body` mudaria o
//! hash com a língua.

use ph2d_label_census::gate::{self, Excecao, Isento};

const TABLE: &str = "crates/ph2d-i18n/src/app_physics.rs";

/// Um ficheiro isento inteiro, com o mecanismo.
const FORA: &[Isento] = &[(
    "physics_tests.rs",
    "os nomes dos COMPONENTES que o arnês liga e desliga (`Ccd`, `LockRotation`) — argumentos de \
     uma fixtura de teste, nunca pintados",
)];

/// Um literal isento, com o mecanismo — todos NOMES de objecto.
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "bridge/dispatch.rs",
        "Joint",
        "o nome por omissão de uma junta sem `Name` — identidade durável (`stable_name_id`), que é \
         como a ponte reencontra a junta depois do respawn do undo",
    ),
    (
        "common.rs",
        "Floor",
        "o nome do chão que as cenas de física montam — dado da cena",
    ),
    (
        "common.rs",
        "Player",
        "o nome do jogador que as cenas de física montam — dado da cena",
    ),
    (
        "joint.rs",
        "Body",
        "o nome por omissão de um corpo sem `Name`, na porta que baptiza — identidade durável",
    ),
    (
        "joint_create.rs",
        "Body",
        "o mesmo nome por omissão, no caminho que cria a junta — identidade durável",
    ),
    (
        "joint_create.rs",
        "{label} Wheel {}",
        "o nome DERIVADO de uma roda nova (`<corpo> Wheel 2`) — identidade, não frase",
    ),
    (
        "joint_wheel.rs",
        "{name} Wheel {}",
        "o mesmo nome derivado de roda, no construtor do conjunto — identidade",
    ),
    (
        "joint_wheel.rs",
        "Body {}",
        "o nome derivado de um corpo numerado do conjunto — identidade",
    ),
    (
        "joint_wheel.rs",
        "Rope {}",
        "o nome derivado de uma corda numerada do conjunto — identidade",
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
];

#[test]
fn every_word_this_family_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA, gate::CENAS);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte da física (HR-15):\n  {}\n\n\
         A cura é uma chave `app.physics.<ficheiro>.<frase>` em `{TABLE}` e um `tr(\"…\")` no sítio \
         (uma frase com peças do código: `tr_with`). ⚠️ Se é um NOME de objecto, a cura é uma linha \
         em `NOT_LANGUAGE` **com o mecanismo** — e lembre-se de que um nome aqui entra no hash de \
         replay.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A metade justa**, com o piso de população das cenas de smoke.
#[test]
fn every_named_exemption_still_shelters_what_it_names() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas: Vec<String> = gate::excecoes_mortas(&src, NOT_LANGUAGE)
        .into_iter()
        .chain(gate::isentos_mortos(&src, FORA))
        .collect();
    assert!(
        mortas.is_empty(),
        "isenções mortas:\n  {}",
        mortas.join("\n  ")
    );
    let em_cena = gate::literais_de_cena(&src, gate::CENAS);
    assert!(
        em_cena >= 600,
        "as cenas de smoke abrigam {em_cena} textos — em 2026-09-16 eram 816 (117 cenas). Ou elas \
         saíram desta crate, ou a régua por NOME deixou de casar e este gate mede o nada"
    );
}

/// ⭐⭐ **As chaves existem dos dois lados.**
#[test]
fn every_key_of_this_family_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, "app.physics.", &[TABLE]);
    assert!(
        c.declaradas >= 8 && c.usadas >= 8,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas — o `tr` pinta o identificador cru:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "declaradas e ninguém as usa — apague-as:\n  {:?}",
        c.orfas
    );
}
