//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 19 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.
//! ⛔ Excepção: um ficheiro com `#[global_allocator]` tem de ser binário PRÓPRIO (dois alocadores
//! globais não cabem num binário, e um contador global veria as alocações dos vizinhos): fica em `tests/`.

mod anchor_mount_hierarchy;
mod end_to_end_m14;
mod generate_transform_v1_fixtures;
mod measure_incremental_capture;
mod measure_restore;
mod measure_spawn_and_death;
mod measure_tag_scan;
mod mede_o_que_a_composicao_ja_da_ao_cerebro;
mod mede_o_que_a_composicao_ja_da_ao_golpe;
mod named_anchor_caps;
mod nesting_sorts_as_a_block;
mod no_untracked_writes_in_the_sim_crates;
mod only_the_door_reads_tags;
mod rewind_runtime;
mod sim_present_flow;
mod sorting_pipeline_determinism;
mod the_frame_ritm_is_per_cell;
mod the_hierarchy_is_the_one_blender_measures;
mod the_morph_graph_survives_the_snapshot;
mod the_present_world_has_no_resources;
mod the_run_never_enters_the_document;
mod transform_determinism;
mod transform_hierarchy;
mod transform_inverse;
mod transform_versioned_postcard;
mod ysort_direction_and_root_repro;
