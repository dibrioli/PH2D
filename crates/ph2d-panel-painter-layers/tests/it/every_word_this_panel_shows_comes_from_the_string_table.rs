//! ⭐⭐⭐ **NENHUMA PALAVRA DO PAINEL PAINTER É ESCRITA NO FONTE** — o HR-15, com a régua que vê o
//! painel INTEIRO.
//!
//! > `CLAUDE.md` §0.3: *«UI canônica: zero hex, zero `f32` literal de UI, **zero string
//! > hardcoded** — tudo via tokens / i18n (HR-15).»*
//!
//! # Por que este painel, e por que agora
//!
//! Era o maior bloco de texto fora da tabela no app: **376** literais com cara de língua, e **zero**
//! chaves `panel.painter_layers.*`. ⚠️ O censo anterior contava **166** — ele seguia o texto só até
//! um pintor, e os construtores de widget (`Button::new(id, "Apply")`), as tabelas
//! (`["Paint", "Erase"]`) e o `format!` ficavam de fora. Um gate sobre aquela régua ficaria verde
//! com mais de metade do painel ainda escrito à mão.
//!
//! ⭐ E não é só canon: sem vocabulário declarado, o degrau `G` do redesenho (esvaziar os painéis,
//! classificando cada entrada por âmbito) não tinha lista para classificar
//! (`docs/UI_New_and_Simple/medicoes/07` §3-bis).
//!
//! # A régua é a da `ph2d-label-census`, e este gate é POR CRATE
//!
//! Uma catraca global com a dívida das outras crates dentro poria a próxima linha vermelha por
//! causa de um gate desta (`CLAUDE.md` §0.2). Este fala só deste painel, a ZERO, com as excepções
//! nomeadas e o mecanismo de cada uma.

use std::path::{Path, PathBuf};

use ph2d_label_census::{keys, language_literals};

const PREFIX: &str = "panel.painter_layers.";
const TABLE: &str = "crates/ph2d-i18n/src/painter_layers.rs";

/// ⭐ As excepções, **com o mecanismo** — `(ficheiro relativo a src/, texto exacto, porquê)`.
///
/// ⚠️ Uma lista de dívida tolerada que não diz *porquê* é uma licença (`CLAUDE.md` §5.0), e o
/// teste irmão exige que cada entrada ainda abrigue um literal real.
const NOT_LANGUAGE: &[(&str, &str, &str)] = &[
    (
        "lib.rs",
        "Painter",
        "o `Panel::TITLE` e' um `const &'static str` que o registo le para a ABA, e o `tr` nao e' \
         `const fn`. Fazer as abas falarem pela tabela e' mudar o contrato do painel nos 26 que o \
         implementam -- obra propria, nomeada no handoff, e nao uma excepcao deste painel.",
    ),
    (
        "paint_adjust.rs",
        "RGB",
        "o separador MESTRE das curvas, irmao de `R`, `G` e `B` na mesma tabela: e' o NOME de um \
         modelo de cor, o mesmo simbolo em toda lingua (como `HSV` e `OKLCH` no seletor de cor). \
         Traduzir so' este dos quatro deixaria a fileira meio traduzida.",
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
        "estes textos com cara de língua estão escritos no fonte do painel Painter e nunca chegam \
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

/// ⭐⭐⭐ **Uma chave com erro de escrita pinta o identificador cru** (e vaza, por quadro); uma
/// declarada e não usada é uma órfã. Os dois lados.
#[test]
fn every_painter_key_exists_on_both_sides() {
    let repo = repo_root();
    let used = keys::keys_used(&repo, PREFIX, &[TABLE]);
    let declared = keys::keys_declared(&repo, &[TABLE], PREFIX);
    // ⛔ Controlo de vacuidade: um caminho errado dá dois conjuntos vazios, que concordam. O piso
    //    é o vocabulário medido na migração de 2026-09-13 menos folga para o painel encolher.
    assert!(
        declared.len() >= 200 && used.len() >= 200,
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
    let orfas: Vec<&String> = declared.iter().filter(|k| !used.contains_key(*k)).collect();
    assert!(
        orfas.is_empty(),
        "estas chaves estão na tabela e ninguém as usa — apague-as:\n  {orfas:?}"
    );
}
