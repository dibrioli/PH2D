//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 7 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod every_word_this_panel_shows_comes_from_the_string_table;
mod hierarchy_apply_event;
mod hierarchy_context_menu;
mod hierarchy_selection_by_identity;
mod hierarchy_sync_round_trip;
mod o_selo_de_uma_linha_cabe_no_selo;
mod seam;
mod the_hierarchy_row_inset_is_the_owners_two;
mod the_rows_of_the_list_touch;
