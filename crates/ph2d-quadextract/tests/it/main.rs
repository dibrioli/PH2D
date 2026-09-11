//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 10 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

#[path = "../support/mod.rs"]
mod support;

mod gate_feature_sparse;
mod gate_lone_singularities;
mod gate_rounding_second_try;
mod gate_seam_closes;
mod gate_stiffening_inert;
mod gate_transition_fallback;
mod gates_exact;
mod gates_fixtures;
mod gates_precision;
mod measure_quad_shape;
