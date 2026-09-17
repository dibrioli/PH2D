//! ⭐⭐⭐ **NENHUMA PALAVRA DO CATÁLOGO DE COMPONENTES É ESCRITA NO FONTE** — o HR-15, num MOTOR.
//!
//! ⛔⛔ **Porque este gate não existia e a régua de ninguém o via.** O fecho de 2026-09-16 pôs 26
//! painéis, a moldura, a shell e 8 crates de família a falar por [`ph2d_i18n`], com 30 censos a
//! defendê-lo. Esta crate **não é nenhum deles** — e o texto que ela declara é pintado por uma que
//! É: a paleta *Add Component* faz, à letra,
//!
//! ```text
//! let mut label = desc.display_name.to_string();          // ⟵ literal CRU, desta crate
//! label.push_str(ph2d_i18n::tr("app.components.…brings")); // ⟵ e o resto traduzido
//! ```
//!
//! ⇒ o censo da `ph2d-app-components` fecha **VERDE** enquanto **86** nomes de componente — a
//! metade que o artista de facto lê numa linha daquela paleta — vivem crus na crate ao lado.
//! *Um censo cuja crate não é DONA do texto que ela pinta fica verde sobre texto cru.*
//!
//! # ⚠️ Os NOMES DE CAMPO ficam FORA, e a ausência é MEDIDA
//!
//! O [`crate::FieldDesc::name`] (103 deles) **não é pintado em lado nenhum**: os únicos leitores
//! fora desta crate são uma mensagem de `assert!` e o gate da §7 do Inspector, que compara rótulos
//! que o pintor lê do descritor **por `field_id`**. Convertê-los seria fabricar dívida sobre a
//! população errada — o defeito que a 1.ª redacção do censo do ritmo pagou ao acusar `48 de 81`
//! ficheiros. *Eles entram no dia em que alguém os pintar, e o gate irmão diz quando.*

use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "component.";
const TABLE: &str = "crates/ph2d-i18n/src/component_catalog.rs";

/// ⭐ As excepções, **com o mecanismo**.
const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn every_word_this_catalog_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte do catálogo e nunca chegam à \
         tabela de strings (HR-15):\n  {}\n\nA cura é a chave `{PREFIX}<componente>.name` em \
         `{TABLE}`, DERIVADA do `canonical_name` — ver o gate irmão.",
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

/// ⭐⭐⭐ **A OUTRA METADE: a chave existe dos DOIS lados** — e aqui ela vale mais do que num painel.
///
/// ⚠️ **Num painel, uma chave que falta pinta o identificador cru numa linha.** Aqui ela pinta-o
/// **na paleta inteira** — o *Add Component* lista as `86` de uma vez, e um descritor novo sem
/// chave entrega ao artista uma linha a dizer `component.foo.name`. ⛔ E é pior do que feio: o
/// `ph2d_i18n::tr` de uma chave desconhecida faz `leak_key` (`Box::leak` do identificador), e a
/// paleta é repintada por quadro.
///
/// ⚠️ **As duas direcções existem e a cura de cada uma é OPOSTA:** uma chave *usada e não
/// declarada* pede uma entrada na tabela; uma *declarada e sem uso* é um descritor que morreu e
/// deixou a palavra dele para trás.
#[test]
fn every_key_of_this_catalog_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, PREFIX, &[TABLE]);
    // ⛔ Controlo de vacuidade: a população MEDIDA na migração (291), menos folga para encolher.
    assert!(
        c.declaradas >= 280 && c.usadas >= 280,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas em `{TABLE}` — a paleta pinta o identificador cru e vaza uma \
         string por quadro:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "declaradas e ninguém as usa — um descritor morreu e deixou a palavra dele:\n  {:?}",
        c.orfas
    );
}
