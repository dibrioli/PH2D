//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 8 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_field_is_never_the_colour_of_what_it_sits_on;
mod an_alias_has_no_value_it_has_a_parent;
mod design_token_sync;
mod measure_override_layer;
mod mockup_tokens_exist;
mod the_ground_stands_under_every_panel;
mod the_leaf_stays_dep_free;
mod the_modern_family_derives_every_token;
mod the_oled_theme_separates_by_border;
