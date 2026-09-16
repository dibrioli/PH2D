//! ⭐⭐⭐ **NENHUMA PALAVRA DO PAINEL DE VECTOR É ESCRITA NO FONTE** — o HR-15, com a régua que vê o
//! painel INTEIRO.
//!
//! > `CLAUDE.md` §0.3: *«UI canônica: zero hex, zero `f32` literal de UI, **zero string
//! > hardcoded** — tudo via tokens / i18n (HR-15).»*
//!
//! Até 2026-09-16 este painel escrevia **225** textos no fonte (medido pela régua da
//! `ph2d-label-census`), o segundo maior bloco de texto fora da tabela entre os painéis — e o censo
//! em Python via **4**, porque ele segue o texto só até um pintor e este painel passa o texto por
//! métodos (`self.slider_row(…)`, `self.segmented(…)`) e por tabelas. Migrado nesse dia por
//! `scripts/migrar-texto-pintado.py` (mapa `docs/UI_New_and_Simple/ferramentas/seccoes_vector.tsv`).
//!
//! # A régua é a da `ph2d-label-census`, e este gate é POR CRATE
//!
//! Irmão do gate do painel Painter, pelas mesmas razões: uma catraca global com a dívida das outras
//! crates dentro poria a próxima linha vermelha por causa de um gate desta (`CLAUDE.md` §0.2).
//!
//! ⚠️ **O que ele NÃO vê, nomeado:** os nomes que o MOTOR publica (efeitos e os parâmetros deles,
//! filtros e os controlos e modos deles, leis de mistura, presets da gaiola) moram noutra crate. Desde
//! 2026-09-16 essa metade tem gate próprio: `every_name_the_engine_publishes_has_a_key`, sobre a
//! correspondência `nomes_do_motor` (a forma do `adjust_nomes` do Painter).

use std::path::{Path, PathBuf};

use ph2d_label_census::{keys, language_literals};

const PREFIX: &str = "panel.vector.";
const TABLE: &str = "crates/ph2d-i18n/src/vector.rs";
/// ⚠️ **Duas tabelas, um prefixo** (2026-09-16): os nomes que o motor publica moram na irmã
/// `vector_engine.rs` (tecto de LOC e responsabilidade). Com uma só aqui, as 114 chaves
/// `panel.vector.engine.*` liam-se «sem tradução».
const TABLES: &[&str] = &[TABLE, "crates/ph2d-i18n/src/vector_engine.rs"];

/// ⭐ As excepções, **com o mecanismo** — `(ficheiro relativo a src/, texto exacto, porquê)`.
const NOT_LANGUAGE: &[(&str, &str, &str)] = &[(
    "lib.rs",
    "Vector",
    "o `Panel::TITLE` e' um `const &'static str` que o registo le para a ABA, e o `tr` nao e' \
     `const fn` -- a mesma excepcao do painel Painter, e a mesma obra propria para a curar.",
)];

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

/// ⭐⭐⭐ **O painel não escreve palavras no fonte.**
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
        "estes textos com cara de língua estão escritos no fonte do painel de Vector e nunca chegam \
         à tabela de strings (HR-15):\n  {}\n\nA cura é uma chave `{PREFIX}<secção>.<nome>` em \
         `{TABLE}` e um `tr(\"…\")` no sítio (uma frase com peças do código: `tr_with`). ⚠️ Se o \
         texto NÃO é língua, a cura é uma linha em `NOT_LANGUAGE` **com o mecanismo** — nunca sem \
         ele.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A METADE JUSTA: cada excepção ainda descreve alguma coisa?** — e é ela o controlo de
/// vacuidade: uma régua partida devolve zero literais e lê-se como aprovada, mas não acha as
/// excepções.
#[test]
fn every_named_exception_still_shelters_a_real_literal() {
    let hits = language_literals(&src_root());
    for (file, text, why) in NOT_LANGUAGE {
        assert!(
            why.len() > 40,
            "a excepção `{file}` · {text:?} não diz o mecanismo — uma lista sem mecanismo é uma \
             licença"
        );
        assert!(
            hits.iter().any(|l| l.rel == *file && l.text == *text),
            "a excepção `{file}` · {text:?} já não abriga literal nenhum (ou a régua ficou cega) — \
             apague a linha, senão ela fica aberta para o próximo texto que caia ali"
        );
    }
}

/// ⭐⭐ **Uma chave usada e não declarada pinta o identificador cru** — em qualquer crate que use o
/// espaço `panel.vector.*`.
#[test]
fn every_vector_key_that_is_used_is_declared() {
    let repo = repo_root();
    let used = keys::keys_used(&repo, PREFIX, TABLES);
    let declared = keys::keys_declared(&repo, TABLES, PREFIX);
    // ⛔ Controlo de vacuidade: o vocabulário medido na migração de 2026-09-16 (≈510 chaves).
    assert!(
        declared.len() >= 450 && used.len() >= 400,
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
        "estas chaves são usadas e NÃO existem em `{TABLE}` — o `tr` pinta o identificador cru:\n  \
         {}",
        sem_traducao.join("\n  ")
    );
}
