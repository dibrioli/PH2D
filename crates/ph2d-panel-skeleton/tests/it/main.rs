//! Um binário de teste por crate: cada ficheiro em `tests/it/` é um MÓDULO deste binário, nunca um
//! `tests/*.rs` solto (auditoria de velocidade de 2026-09-10, onda W1 — 3× menos compilação).
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo.

mod every_word_this_panel_shows_comes_from_the_string_table;
