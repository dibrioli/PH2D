//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 29 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod boundary_arity;
mod generated_wgsl_validates;
mod gpu_boids;
mod gpu_boids_scale;
mod gpu_collide;
mod gpu_cpu_parity;
mod gpu_cpu_parity_arith;
mod gpu_cpu_parity_curve;
mod gpu_cpu_parity_deform;
mod gpu_cpu_parity_driven;
mod gpu_cpu_parity_holds;
mod gpu_cpu_parity_sim;
mod gpu_cpu_parity_stats;
mod gpu_cpu_parity_table_seed;
mod gpu_cpu_parity_time;
mod gpu_cpu_parity_xy;
mod gpu_grid;
mod gpu_neighbor;
mod gpu_proximity;
mod gpu_reduce;
mod gpu_reduce_perf;
mod gpu_scan;
mod gpu_stream_ops;
mod gpu_texture_id;
mod gpu_voronoi;
mod measure_static_orbit;
mod node_key_uniform;
mod plan_analysis;
mod plan_simulation;
mod sim_invalidation;
