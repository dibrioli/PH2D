//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 9 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

#[path = "../look/mod.rs"]
mod look;

mod fx_look;
mod hex_probe;
mod measure_corner_on_arcs;
mod measure_fill_rule_matters;
mod measure_node_reach;
mod measure_scene_clone;
mod measure_trim_on_shapes;
mod node_reach;
mod which_shapes_close;
