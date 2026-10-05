//! **A leitura da fonte da shell** para os arch-gates desta suíte — `source` e `function_body`.
//!
//! ⚠️ Eles viviam em `tests/sculpt_source/mod.rs`, o módulo partilhado que nasceu com os gates da
//! escultura e serviu também gates 2D (o menu Ficheiro, a timeline). A escultura saiu (ADR-0179),
//! e os dois helpers que os gates 2D usam ficaram aqui, sem mudança de comportamento.

use std::fs;

/// A fonte de `src/<name>` **sem comentários**.
///
/// ⚠️ Não é higiene: um arch-gate que varre o arquivo cru afirma coisas sobre a PROSA, e um gate
/// que dispara em documentação ensina a não documentar.
pub fn source(name: &str) -> String {
    let raw = fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("não consegui ler src/{name}: {e}"));
    raw.lines()
        .map(|l| match l.find("//") {
            Some(at) => &l[..at],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// O corpo de `fn <name>` até a chave que o fecha, contando profundidade.
pub fn function_body(src: &str, name: &str) -> String {
    // ⚠️ **O delimitador é load-bearing:** `find("fn render")` casa por PREFIXO com
    // `fn render_live`. O nome de uma função termina no `(` dos parâmetros (ou no `<` dos
    // genéricos). Quem passa a assinatura (`"radius_px(&self)"`) já traz o delimitador.
    let at = if name.contains('(') {
        src.find(&format!("fn {name}"))
    } else {
        ["(", "<"]
            .iter()
            .filter_map(|d| src.find(&format!("fn {name}{d}")))
            .min()
    }
    .unwrap_or_else(|| panic!("não achei `fn {name}`"));
    let open = src[at..].find('{').expect("corpo") + at;
    let mut depth = 0i32;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return src[open..open + i + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("`fn {name}` não fecha");
}
