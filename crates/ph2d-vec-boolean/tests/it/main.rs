//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 22 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod an_offset_past_the_shapes_death_leaves_no_phantom;
mod arrangement_product_shapes;
mod measure_aligned_stroke;
mod measure_face_hit;
mod measure_live_boolean_chain;
mod measure_power_stroke;
mod measure_power_stroke_slivers;
mod measure_width_presets;
mod offset_live_cost;
mod probe_contour_cost;
mod probe_contour_demo_star;
mod probe_offset_as_effect;
mod probe_offset_extreme_d;
mod probe_offset_fine_sweep;
mod probe_offset_ring_sign;
mod probe_ring_loops;
mod silhouette_cost;
mod silhouette_of_a_stroked_shape;
mod the_chain_folds_with_a_verb_per_step;
mod the_dilate_floor_matches_the_engine;
mod the_offset_never_takes_the_app_down;
mod the_ring_matches_the_booleana;
