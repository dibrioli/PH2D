//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 14 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod architecture_toll;
mod composite_blend;
mod gpu_colorize_look;
mod gpu_fill_fit;
mod gpu_render;
mod hardness_law;
mod integral_law;
mod pack_perf;
mod painter_look;
mod probe_bucket_vs_draw_filled;
mod probe_halo_under_soft_line;
mod sampling_invariance;
mod walk_gpu_parity;
mod walk_perf;
