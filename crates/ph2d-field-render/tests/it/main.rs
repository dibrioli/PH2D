//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 24 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod after_the_settle;
mod cohort_dispersion;
mod edge_pass_budget;
mod gradient_is_lazy;
mod how_many_frames_to_keep;
mod how_many_slabs_now;
mod hull_cache_posed;
mod march_budget;
mod stale_costs_as_an_oracle;
mod tape_budget;
mod tape_cache_alternation;
mod tape_cache_budget;
mod the_bend_does_not_starve_the_march;
mod the_eviction_storm;
mod the_picture_matches_an_honest_march;
mod the_piece_does_not_vanish_as_shapes_are_added;
mod the_price_of_four_views;
mod the_price_of_freeing_a_tape;
mod the_price_of_the_curves;
mod the_price_of_the_superformula;
mod the_price_of_the_thread;
mod the_price_of_the_torus_knot;
mod the_twist_does_not_starve_the_march;
mod what_a_stack_of_deformers_costs_the_march;
mod where_the_frame_goes_now;
