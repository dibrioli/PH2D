//! ⭐ **NENHUMA PALAVRA DA LUZ 2D É ESCRITA NO FONTE** — o HR-15 na crate da família (W4).
//!
//! As palavras que o artista lê (o aviso da recusa do solver, os rótulos do menu Add) vivem em
//! `ph2d-i18n` (`luz2d.rs`, `object_add.rs`); as da secção do Inspector são do painel e o censo dele as
//! conta. As cenas de smoke (`smoke.rs`) são DADOS da cena (nomes de objeto) e ficam fora pela régua.

use ph2d_label_census::gate::{self, Excecao, Isento};

const TABLE: &str = "crates/ph2d-i18n/src/luz2d.rs";

const FORA: &[Isento] = &[];

const NOT_LANGUAGE: &[Excecao] = &[
    (
        "quadro.rs",
        "luz2d quadro do editor",
        "o rótulo de DEPURAÇÃO do encoder do wgpu (o que o RenderDoc mostra) — nunca pintado",
    ),
    (
        "quadro_leituras.rs",
        "luz2d quadro do editor",
        "o mesmo rótulo de DEPURAÇÃO do encoder, no instrumento (`PH2D_LIGHT2D_CUSTO=1`) — nunca pintado",
    ),
    (
        "quadro_leituras.rs",
        "luz2d sonda calibração",
        "o rótulo de DEPURAÇÃO da textura de calibração da régua de memória (W5y) — nunca pintado",
    ),
    (
        "quadro_leituras.rs",
        "calibração",
        "a etiqueta da linha do instrumento no TERMINAL (`PH2D_LIGHT2D_MEMORIA`, W5y) — nunca pintada",
    ),
];

#[test]
fn every_word_this_family_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA, gate::CENAS);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte da luz 2D (HR-15):\n  {}\n\n\
         A cura é uma chave `light2d.<assunto>.<frase>` em `{TABLE}` e um `tr(\"…\")` no sítio.",
        intrusos.join("\n  ")
    );
}

/// A metade justa, com o piso de população das cenas de smoke.
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
    eprintln!("textos das cenas de smoke: {em_cena}");
    // Medido em 2026-10-09 (W4, cenas 1, 2 e 4): 17.
    assert!(
        em_cena >= 15,
        "as cenas de smoke abrigam {em_cena} textos — ou saíram da crate, ou a régua por NOME \
         deixou de casar e este gate mede o nada"
    );
}

/// ⭐ As chaves `light2d.*` existem dos dois lados.
#[test]
fn every_key_of_this_family_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, "light2d.", &[TABLE]);
    assert!(
        c.declaradas >= 2 && c.usadas >= 2,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "declaradas e ninguém as usa:\n  {:?}",
        c.orfas
    );
}
