//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 8 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_dropped_rule_says_why;
mod cada_palavra_deste_no_vem_da_tabela;
mod growth_is_two_laws;
mod measure_lsystem_ceiling;
mod newborn_law;
mod no_preset_shows_a_knob_its_grammar_cannot_read;
mod presets_frame_themselves;
mod the_alphabet_is_data_and_it_is_complete;
mod the_parser_fails_closed;
