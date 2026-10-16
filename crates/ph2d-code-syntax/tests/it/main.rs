//! Um binário de teste para a crate (convenção do repo: teste novo = ficheiro aqui + linha `mod`).

mod autoedit;
mod cronometro;
mod dobras;
mod erros;
mod incremental;
mod nao_ascii;
mod indentacao;
mod pares;
mod queries;
mod realce;

/// A amostra canónica (cópia com cabeçalho de origem, ver o próprio ficheiro).
pub const BOB: &str = include_str!("fixtures/bob.luau");

/// Linha (base 0) da primeira linha do `BOB` que contém `needle`.
pub fn line_of(src: &str, needle: &str) -> usize {
    src.lines()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("`{needle}` não está na amostra"))
}
