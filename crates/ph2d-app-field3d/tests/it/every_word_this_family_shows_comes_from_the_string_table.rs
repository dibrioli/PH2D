//! ⭐⭐⭐ **NENHUMA PALAVRA DO MODELADOR É ESCRITA NO FONTE** — o HR-15 na crate de família do campo
//! implícito.
//!
//! Medido em 2026-09-16: **71** textos — os avisos de importar/exportar, os **diagnósticos da peça**
//! (`notice.rs`: *«Two parts of this piece point at each other in a loop»*), os verbos do desenho
//! que vira peça, os títulos dos grupos da paleta de formas e os **selos** da Hierarquia
//! (`BSE`/`UNI`/`SUB`/`INT`/`ISO`/`LNK`). Migrados para `ph2d-i18n/src/app_field3d.rs`.
//!
//! ⚠️⚠️ **Os selos são TEXTO, e eram `const`** — e `tr` não é `const fn`. Eles passam a guardar uma
//! [`ph2d_i18n::TextKey`] e quem pinta (ou compara) escreve `.tr()`; guardados como `&str`, uma
//! chave e um texto são o MESMO tipo e o esquecimento compila
//! (`feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting`).
//!
//! ⚠️ E `Family::title` dizia por escrito que um literal ali era legítimo *«porque quem pinta é o
//! widget genérico»* — é ao contrário: um widget genérico recebe o rótulo **já resolvido**, logo
//! quem o resolve é a FONTE do rótulo.

use ph2d_label_census::gate::{self, Excecao, Isento};

const TABLE: &str = "crates/ph2d-i18n/src/app_field3d.rs";

/// Nenhum ficheiro é isento inteiro nesta crate — a lista existe para o dia em que um seja.
const FORA: &[Isento] = &[];

/// Um literal isento, com o mecanismo.
const NOT_LANGUAGE: &[Excecao] = &[(
    "scene.rs",
    "Model",
    "o nome da peça na Hierarquia — CONTEÚDO que o artista renomeia (um `Name`, que neste repo é \
     identidade durável via `stable_name_id`), não chrome",
)];

#[test]
fn every_word_this_family_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte do modelador (HR-15):\n  {}\n\n\
         A cura é uma chave `app.field3d.<ficheiro>.<frase>` em `{TABLE}` e um `tr(\"…\")` no sítio \
         (uma frase com peças do código: `tr_with`, com marcadores NOMEADOS). ⚠️ Num `const` a cura \
         é uma `ph2d_i18n::TextKey` — ver os selos em `scene_acts.rs`.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A metade justa**, com o piso de população do roteador de cenas.
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
        em_cena >= 2,
        "o roteador de cenas abriga {em_cena} textos — em 2026-09-16 eram 2 (as razões que ele diz a \
         quem pede uma cena podada ou fora da faixa). Se chegou a zero, a régua por NOME deixou de \
         casar e este gate mede o nada"
    );
}

/// ⭐⭐ **As chaves existem dos dois lados.**
#[test]
fn every_key_of_this_family_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, "app.field3d.", &[TABLE]);
    assert!(
        c.declaradas >= 65 && c.usadas >= 65,
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
