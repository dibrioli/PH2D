//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio. ⚠️ Esta crate
//! ficou de fora da onda W1 com UM `tests/seam.rs` solto; em 2026-09-16, quando o gate por crate
//! do HR-15 chegou, os dois passaram a ser módulos daqui.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod every_word_this_panel_shows_comes_from_the_string_table;
mod seam;
