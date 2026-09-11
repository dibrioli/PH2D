//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 15 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.
//! ⛔ Excepção: um ficheiro com `#[global_allocator]` tem de ser binário PRÓPRIO (dois alocadores
//! globais não cabem num binário, e um contador global veria as alocações dos vizinhos): fica em `tests/`.

mod closed_mesh_never_leaks_a_ray;
mod measure_cotangent_smoothing;
mod measure_curvature;
mod measure_curvature_estimators;
mod measure_curvature_units;
mod measure_dyntopo;
mod measure_extract;
mod measure_multires;
mod measure_normals;
mod measure_subdivide;
mod measure_transfer_probe;
mod measure_where_a_pick_misses;
mod probe_pick_after_topology;
