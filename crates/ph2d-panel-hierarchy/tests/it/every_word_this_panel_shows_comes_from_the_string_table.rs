//! ⭐⭐⭐ **NENHUMA PALAVRA DA HIERARQUIA É ESCRITA NO FONTE** — o HR-15, com a régua que vê o
//! painel INTEIRO.
//!
//! O mesmo gate da `ph2d-panel-painter-layers` (a régua é a da `ph2d-label-census`, partilhada; só a
//! lista e o prefixo são deste painel). ⭐ A Hierarquia é o painel que o artista tem aberto o dia
//! inteiro, e tinha **zero** chaves `panel.hierarchy.*` — logo nenhum vocabulário para o degrau `G`
//! classificar (`docs/UI_New_and_Simple/medicoes/07` §3-bis).

use std::path::{Path, PathBuf};

use ph2d_label_census::{keys, language_literals};

const PREFIX: &str = "panel.hierarchy.";
const TABLE: &str = "crates/ph2d-i18n/src/lib.rs";

/// ⭐ As excepções, **com o mecanismo** — `(ficheiro relativo a src/, texto exacto, porquê)`.
const NOT_LANGUAGE: &[(&str, &str, &str)] = &[
    (
        "lib.rs",
        "Hierarchy",
        "o `Panel::TITLE` e' um `const &'static str` que o registo le para a ABA, e o `tr` nao e' \
         `const fn`. Fazer as abas falarem pela tabela e' mudar o contrato do painel nos 26 que o \
         implementam -- obra propria, nomeada no handoff.",
    ),
    (
        "ids/menus.rs",
        "Scene Root",
        "e' o NOME de uma entidade da cena de amostra (o `hierarchy_label_for_id` devolve-o para a \
         etiqueta da seleccao): conteudo do documento, como o nome de uma camada. Traduzir o nome \
         de um objecto do artista seria errado.",
    ),
];

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<crate>/ tem dois pais")
        .to_path_buf()
}

#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let intrusos: Vec<String> = language_literals(&src_root())
        .into_iter()
        .filter(|l| {
            !NOT_LANGUAGE
                .iter()
                .any(|(f, t, _)| l.rel == *f && l.text == *t)
        })
        .map(|l| format!("{}:{} · {:?}", l.rel, l.line, l.text))
        .collect();
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte da Hierarquia e nunca chegam à \
         tabela de strings (HR-15):\n  {}\n\nA cura é uma chave `{PREFIX}<nome>` em `{TABLE}` e \
         um `tr(\"…\")` no sítio (uma frase com peças do código: `tr_with`). ⚠️ Se o texto NÃO é \
         língua, a cura é uma linha em `NOT_LANGUAGE` **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}

/// ⭐ A metade justa — e o controlo de vacuidade: uma régua cega não acha as excepções.
#[test]
fn every_named_exception_still_shelters_a_real_literal() {
    let hits = language_literals(&src_root());
    for (file, text, why) in NOT_LANGUAGE {
        assert!(
            why.len() > 40,
            "a excepção `{file}` · {text:?} não diz o mecanismo"
        );
        assert!(
            hits.iter().any(|l| l.rel == *file && l.text == *text),
            "a excepção `{file}` · {text:?} já não abriga literal nenhum (ou a régua ficou cega) — \
             apague a linha"
        );
    }
}

#[test]
fn every_hierarchy_key_exists_on_both_sides() {
    let repo = repo_root();
    let used = keys::keys_used(&repo, PREFIX, &[TABLE]);
    let declared = keys::keys_declared(&repo, &[TABLE], PREFIX);
    // ⛔ Controlo de vacuidade: dois conjuntos vazios concordam.
    assert!(
        declared.len() >= 3 && used.len() >= 3,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        declared.len(),
        used.len()
    );
    let sem_traducao: Vec<String> = used
        .iter()
        .filter(|(k, _)| !declared.contains(*k))
        .map(|(k, f)| format!("{k}  (usada em {f})"))
        .collect();
    assert!(
        sem_traducao.is_empty(),
        "estas chaves são usadas e NÃO existem em `{TABLE}`:\n  {}",
        sem_traducao.join("\n  ")
    );
    let orfas: Vec<&String> = declared.iter().filter(|k| !used.contains_key(*k)).collect();
    assert!(
        orfas.is_empty(),
        "estas chaves estão na tabela e ninguém as usa — apague-as:\n  {orfas:?}"
    );
}
