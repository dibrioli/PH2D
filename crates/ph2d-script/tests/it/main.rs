//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 7 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_api_e_a_tabela;
mod a_recarga_e_o_console;
mod corpus_de_erros;
mod diagnostico_do_luau_medido;
mod hot_reload_per_entity;
mod lateral_storage_determinism;
mod lateral_storage_rejects_non_pod;
mod luau_script_component;
mod m7_gc_pause;
mod m7_host;
mod mede_o_que_a_composicao_ja_da_aos_tipos;
mod messaging_proptest;
