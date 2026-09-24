//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 28 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

#[path = "../util/mod.rs"]
mod util;

mod acceptance;
mod acceptance_budget;
mod deposit_rows;
mod fingerprint;
mod flow_grid;
mod flow_symmetry;
mod live_span;
mod measure_dab_halves;
mod measure_density;
mod measure_deposit_rows;
mod measure_experimental;
mod measure_flow_ratio;
mod measure_flow_reduction;
mod measure_long_session;
mod measure_parallel_rows;
mod measure_pass_cost;
mod measure_transfer_candidates;
mod measure_transport_range;
mod parallel_rows;
mod perf;
mod perf_experimental;
mod product_doors;
mod product_rewet;
mod product_tool_doors;
mod resumable_step;
mod solver_symmetry;
mod spans;
mod the_accumulate_declares_what_it_wrote;
mod the_window_follows_the_brush;
mod transfer_accuracy;
