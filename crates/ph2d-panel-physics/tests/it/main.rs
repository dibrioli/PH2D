//! Um binário de teste por crate: cada ficheiro em `tests/it/` é um MÓDULO deste binário, nunca um
//! `tests/*.rs` solto (auditoria de velocidade de 2026-09-10, onda W1 — 3× menos compilação).
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo.

mod as_seccoes_arrastam_e_tem_tema; // o tema e o arrasto das secções do painel
mod every_word_this_panel_shows_comes_from_the_string_table;
