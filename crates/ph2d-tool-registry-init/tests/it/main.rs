//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 11 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod architecture_image_tool_kind_contract;
mod architecture_register_all_alphabetical;
mod chrome_manifest_coverage;
mod every_image_tool_is_reachable_without_the_legacy_bar;
mod every_image_tool_pill_dispatches;
mod registry_a11y_role;
mod registry_budget_aggregate;
mod registry_i18n_keys;
mod registry_no_tool_symbols_in_release;
mod staleness;
mod tool_manifest_design_sync;
