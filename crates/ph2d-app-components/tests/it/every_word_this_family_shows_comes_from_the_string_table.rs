//! ⭐⭐⭐ **NENHUMA PALAVRA DOS VERBOS DE PREFAB É ESCRITA NO FONTE** — o HR-15 na crate de família
//! das instâncias.
//!
//! > `CLAUDE.md` §0.3: *«UI canônica: … **zero string hardcoded** — tudo via tokens / i18n (HR-15).»*
//!
//! Medido em 2026-09-16: **34** textos fora das cenas de smoke — quase todos **avisos de recusa**
//! (*«Not a prefab — pick the prefab row»*), que é a superfície onde o artista aprende o que o verbo
//! queria. Migrados para `ph2d-i18n/src/app_components.rs`.
//!
//! ⛔ **O que fica de fora é NOME, não vocabulário.** Um nome por omissão (`"Prefab"`, `"Instance"`)
//! entra no `Name` da entidade, e o `Name` é **identidade durável** neste repo — o
//! `stable_name_id` fecha um hash sobre ele, e é por esse hash que uma junta, uma faixa de
//! timeline e um elo de instância reencontram o objecto depois de o undo o respawnar. Traduzir
//! um deles muda a identidade **com a língua**; isso é decisão do dono, não desta migração.

use ph2d_label_census::gate::{self, Excecao, Isento};

const TABLE: &str = "crates/ph2d-i18n/src/app_components.rs";

/// Um ficheiro isento inteiro, com o mecanismo.
const FORA: &[Isento] = &[
    (
        "instance_docs.rs",
        "as RAZÕES declaradas de cada documento possuído que a cópia profunda dropa — prosa lida por \
         um gate (o censo de dois lados), nunca pintada",
    ),
    (
        "test_support.rs",
        "os nomes das fixturas do arnês (`Image`, `Rig`, `Arm`) — cena de teste, atrás da feature \
         `test-support`",
    ),
];

/// Um literal isento, com o mecanismo — nomes que são IDENTIDADE, e consola.
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "instance_verbs.rs",
        "Prefab",
        "o nome por omissão de uma receita sem `Name` — entra no `Name`, que é identidade durável \
         (`stable_name_id`)",
    ),
    (
        "instance_verbs.rs",
        "{base_name} Variant",
        "o nome da variante, DERIVADO do nome da base (o idioma do Unity) — identidade, não frase",
    ),
    (
        "instantiate.rs",
        "Instance",
        "o nome por omissão de uma cópia sem `Name` — identidade durável",
    ),
    (
        "instantiate.rs",
        "Entity",
        "o nome por omissão de uma duplicação sem `Name` — identidade durável",
    ),
    (
        "instantiate.rs",
        "materializar peca",
        "o VERBO que o aviso de consola do `instance_docs::warn` nomeia — terminal, nunca ecrã",
    ),
    (
        "instantiate.rs",
        "promover peca",
        "o mesmo verbo do aviso de consola, no caminho da promoção — terminal, nunca ecrã",
    ),
];

#[test]
fn every_word_this_family_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte da família das instâncias (HR-15):\n  {}\n\n\
         A cura é uma chave `app.components.<ficheiro>.<frase>` em `{TABLE}` e um `tr(\"…\")` no \
         sítio (uma frase com peças do código: `tr_with`, com marcadores NOMEADOS). ⚠️ Se o texto \
         NÃO é língua (um NOME de objecto, uma linha de terminal), a cura é uma linha em `FORA` ou \
         em `NOT_LANGUAGE` **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A metade justa** — cada isenção ainda abriga o que nomeia, e o piso das cenas de smoke é o
/// controlo de vacuidade da régua por NOME DE FICHEIRO (HOWTO §2.7).
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
    let em_cena = gate::literais_de_cena(&src);
    assert!(
        em_cena >= 40,
        "as cenas de smoke abrigam {em_cena} textos — em 2026-09-16 eram 57. Ou elas saíram desta \
         crate (apague a regra), ou a régua por NOME deixou de casar e este gate passou a medir o \
         nada"
    );
}

/// ⭐⭐ **As chaves existem dos dois lados** — uma chave sem braço pinta o identificador cru e vaza.
#[test]
fn every_key_of_this_family_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, "app.components.", &[TABLE]);
    assert!(
        c.declaradas >= 30 && c.usadas >= 30,
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
