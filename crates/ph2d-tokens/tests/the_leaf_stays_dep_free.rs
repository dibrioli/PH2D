//! **A folha de que 44 widgets dependem não ganha dependências de runtime** — arch-gate sobre o
//! `Cargo.toml` desta crate.
//!
//! # Por que um GATE, e não a disciplina de quem edita
//!
//! O `Cargo.toml` desta crate declara, por extenso, *"design-data puro — zero runtime deps"*. Essa
//! frase é a razão de existirem **duas** crates satélite em vez de mais dois módulos aqui:
//!
//! - a **`ph2d-token-math`**, porque `ph2d-expr-parse` arrasta `ph2d-expr` → `ph2d-nodegraph`, e um
//!   botão de ícone passaria a compilar o motor de cozimento para saber de que cor é;
//! - a **`ph2d-tokens-dtcg`** (plano UI/UX W9), porque `serde_json` é um parser de JSON no caminho
//!   de compilação de cada um desses widgets para uma feature que corre **duas vezes na vida de um
//!   projeto**.
//!
//! Um comentário não impede a linha seguinte. Este gate impede — e o modo de falha que ele fecha é
//! silencioso: acrescentar uma dep aqui **compila**, passa em todos os outros testes, e só aparece
//! como tempo de build de toda a árvore.
//!
//! ⚠️ **`[dev-dependencies]` é OUTRA coisa e fica de fora de propósito:** ela não entra no grafo de
//! quem depende desta crate. O `design_token_sync` usa `serde_json` ali há muito, e é ele que
//! re-parseia o `tokens.json` com um parser INDEPENDENTE do `build.rs`.

use std::fs;

#[test]
fn ph2d_tokens_declares_no_runtime_dependency() {
    let toml = fs::read_to_string("Cargo.toml").expect("Cargo.toml desta crate");

    // A secção `[dependencies]` até à próxima secção.
    let start = toml
        .find("\n[dependencies]")
        .expect("o Cargo.toml tem de ter a seccao [dependencies], mesmo vazia")
        + 1;
    let body = &toml[start..];
    let end = body[1..].find("\n[").map_or(body.len(), |i| i + 1);
    let section = &body[..end];

    let deps: Vec<&str> = section
        .lines()
        .skip(1) // o `[dependencies]`
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    assert!(
        deps.is_empty(),
        "a `ph2d-tokens` ganhou dependencia(s) de runtime: {deps:?}\n\
         Ela e' a folha de que 44 widgets e todo painel dependem, e o `Cargo.toml` dela declara \
         zero deps de propósito. Se a capacidade nova precisa de uma dep, ela vive numa crate \
         satelite — o precedente e' a `ph2d-token-math` (o parser de math) e a `ph2d-tokens-dtcg` \
         (o codec DTCG)."
    );

    // ⚠️ **CONTROLE POSITIVO**: sem ele, um `Cargo.toml` que o gate não achasse (renomeado,
    // movido) passaria com a lista vazia — verde por não medir nada.
    assert!(
        toml.contains("name = \"ph2d-tokens\""),
        "o gate leu um Cargo.toml que nao e' o desta crate"
    );
    assert!(
        toml.contains("[dev-dependencies]"),
        "o gate tem de conseguir distinguir as duas seccoes — sem a de dev ele nao esta' a \
         separar nada"
    );
}
