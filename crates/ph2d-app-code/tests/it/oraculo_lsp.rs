//! ⭐ **E7 — o ORÁCULO de posição do símbolo culpado**: o `luau-lsp` fixado corre sobre o script
//! quebrado do smoke (`docs/CodeEditor/ferramentas/smoke/script_quebrado.luau`) e o `range` do
//! diagnóstico dele tem de ser EXACTAMENTE o culpado que a aba sublinha (vindo do tree-sitter). A
//! saída dele está gravada em `fixtures/lsp_script_quebrado.txt` (com cabeçalho); a corrida de hoje
//! tem de bater com ela — um `luau-lsp` que mude de opinião reprova aqui, não no ecrã.
//!
//! Regravar a fixtura (só depois de ler a diferença): `PH2D_GRAVA_ORACULO=1` neste teste.

use crate::Mesa;
use ph2d_code_lsp::{
    LspClient, ServerConfig, Solver, pinned_version, position_to_byte, resolve_binary,
};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("raiz do repo")
}

const FIXTURA: &str = "tests/it/fixtures/lsp_script_quebrado.txt";

#[test]
fn o_culpado_do_tree_sitter_e_o_range_do_luau_lsp() {
    let script = repo().join("docs/CodeEditor/ferramentas/smoke/script_quebrado.luau");
    let texto = std::fs::read_to_string(&script).expect("script quebrado do smoke");
    // O lado do PH2D: o problema que a aba mostra.
    let (mut m, ids) = Mesa::new(&[("script_quebrado.luau", &texto)]);
    m.pede(ids[0]);
    let problemas = m.docs().problems_of(ids[0]);
    let p = problemas.first().expect("a aba vê o erro");
    let culpado = p.culprit.clone().expect("com culpado");
    let inicio_da_linha: usize = texto.split_inclusive('\n').take(p.line).map(str::len).sum();
    assert_eq!(
        (
            p.line,
            culpado.start - inicio_da_linha,
            culpado.end - inicio_da_linha
        ),
        (2, 10, 11),
        "linha 3, o segundo `=`"
    );

    // O oráculo.
    let bin = match resolve_binary() {
        Ok((p, _)) => p,
        Err(e) => panic!(
            "⛔ {e}\n   Este gate precisa do luau-lsp fixado: `bash scripts/luau-lsp-fetch.sh`."
        ),
    };
    let mut cfg =
        ServerConfig::new(bin).with_definitions("@ph2d", repo().join("runtime/luau/ph2d.d.luau"));
    cfg.solver = Solver::Old;
    let mut c = LspClient::start(cfg).expect("luau-lsp arranca");
    let uri = "file:///ph2d-gate/script_quebrado.luau";
    c.pump();
    c.did_open(uri, &texto).expect("didOpen");
    let diags = c
        .wait_diagnostics(uri, Duration::from_secs(30))
        .expect("diagnósticos")
        .diagnostics;
    c.shutdown().expect("shutdown");
    // O luau-lsp 1.70.1 dá o erro de sintaxe com o código `1014` e a mensagem prefixada
    // («SyntaxError: …»); os outros dois são lints (aviso) sobre a mesma linha.
    let sintaxe: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == Some(1) && d.message.starts_with("SyntaxError"))
        .collect();
    assert_eq!(sintaxe.len(), 1, "um erro de sintaxe: {diags:?}");
    let d = sintaxe[0];
    let (a, b) = (
        position_to_byte(&texto, d.range.start),
        position_to_byte(&texto, d.range.end),
    );
    assert_eq!(
        a..b,
        culpado,
        "o range do luau-lsp ({:?}) ≠ o culpado do tree-sitter",
        d.range
    );

    // A fixtura gravada.
    let corrida: String = sintaxe
        .iter()
        .map(|d| {
            format!(
                "{}:{}-{}:{} [{}] {}\n",
                d.range.start.line,
                d.range.start.character,
                d.range.end.line,
                d.range.end.character,
                d.code.as_deref().unwrap_or("-"),
                d.message
            )
        })
        .collect();
    let caminho = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURA);
    if std::env::var_os("PH2D_GRAVA_ORACULO").is_some() {
        let cabecalho = format!(
            "# Oráculo do E7 — luau-lsp {} (fixado, ph2d-code-lsp), solver antigo, definições runtime/luau/ph2d.d.luau.\n\
             # Entrada: docs/CodeEditor/ferramentas/smoke/script_quebrado.luau. Uma linha por diagnóstico de ERRO de sintaxe:\n\
             # <linha>:<carácter UTF-16>-<linha>:<carácter> [código] mensagem (0-based, como o LSP os dá).\n\
             # Gravado por crates/ph2d-app-code/tests/it/oraculo_lsp.rs com PH2D_GRAVA_ORACULO=1.\n",
            pinned_version()
        );
        std::fs::write(&caminho, cabecalho + &corrida).expect("grava a fixtura");
    }
    let gravada = std::fs::read_to_string(&caminho).expect("fixtura do oráculo");
    let corpo: String = gravada
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_eq!(
        corrida, corpo,
        "o luau-lsp mudou de opinião sobre o script quebrado"
    );
}
