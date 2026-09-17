//! ⭐⭐⭐ **NENHUMA PALAVRA DOS VERBOS DO FLIP É ESCRITA NO FONTE** — o HR-15 na crate de família da
//! animação 2D.
//!
//! Medido em 2026-09-16: **12** textos fora das cenas de smoke, todos **recusas** do balde, do
//! colorize, da escultura de traço e da edição de pontos (*«Fill leaked — raise Gap Closure to seal
//! the outline»*). Migrados para `ph2d-i18n/src/app_flip.rs`.
//!
//! ⚠️ Os textos dos dois PAINÉIS do Flip já viviam na tabela (`panel.flip.*`, `panel.flip_frames.*`)
//! — o que faltava era o que os VERBOS dizem quando recusam.

use ph2d_label_census::gate::{self, Excecao, Isento};

const TABLE: &str = "crates/ph2d-i18n/src/app_flip.rs";

/// Um ficheiro isento inteiro, com o mecanismo.
const FORA: &[Isento] = &[(
    "pass.rs",
    "os RÓTULOS de depuração do wgpu (`label:` de um passe e de um encoder) — o que aparece no \
     RenderDoc, nunca no ecrã do artista",
)];

/// Um literal isento, com o mecanismo — nomes que são IDENTIDADE.
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "layers.rs",
        "Layer {}",
        "o nome por omissão de uma camada NOVA — conteúdo que o artista renomeia e que o documento \
         grava, não vocabulário do chrome",
    ),
    (
        "demo.rs",
        "Demo Flip",
        "o nome do objecto que a cena de demonstração põe na Hierarquia — dado da cena",
    ),
];

#[test]
fn every_word_this_family_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte da família do Flip (HR-15):\n  {}\n\n\
         A cura é uma chave `app.flip.<ficheiro>.<frase>` em `{TABLE}` e um `tr(\"…\")` no sítio \
         (uma frase com peças do código: `tr_with`). ⚠️ Se o texto NÃO é língua (um NOME, um rótulo \
         de GPU), a cura é uma linha em `FORA` ou em `NOT_LANGUAGE` **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A metade justa**, com o piso de população das cenas (a régua delas é o NOME do ficheiro).
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
        em_cena >= 25,
        "as cenas de smoke abrigam {em_cena} textos — em 2026-09-16 eram 38. Ou elas saíram desta \
         crate, ou a régua por NOME deixou de casar e este gate mede o nada"
    );
}

/// ⭐⭐ **As chaves existem dos dois lados.**
#[test]
fn every_key_of_this_family_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, "app.flip.", &[TABLE]);
    assert!(
        c.declaradas >= 10 && c.usadas >= 10,
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
