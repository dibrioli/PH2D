//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 55 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.
//! ⛔ Excepção: um ficheiro com `#[global_allocator]` tem de ser binário PRÓPRIO (dois alocadores
//! globais não cabem num binário, e um contador global veria as alocações dos vizinhos): fica em `tests/`.

mod a_vector_path_fades;
mod apply;
mod apply_from_doc;
mod apply_perf;
mod arrange_is_independent;
mod auto_orient;
mod clearing_a_formula_hands_the_pose_back;
mod clip_clock;
mod clip_stack;
mod clip_stack_autokey;
mod clip_stack_eval;
mod container_transport;
mod containers_are_assets;
mod detached_bindings;
mod doc_clips;
mod doc_roundtrip;
mod explicit_duration;
mod expr_in_blend;
mod expressions;
mod fade_curves;
mod fade_fingerprint;
mod fade_fingerprint_channels;
mod follow_an_unbound_object;
mod gap_fade_out;
mod intents;
mod lead_in;
mod lone_fade;
mod loop_pingpong_gap;
mod loop_wrap;
mod loop_wrap_both_ends;
mod loop_wrap_out;
mod measure_motion_path;
mod morph_is_authorable;
mod motion_path;
mod motion_path_perf;
mod nesting_additive;
mod nesting_clock;
mod nesting_data;
mod nesting_leads;
mod nesting_map;
mod nesting_view;
mod pingpong_scrub_exit;
mod pingpong_tail_fade;
mod prop_readback;
mod pure_expression_window;
mod seam_determinism;
mod signals_crossed;
mod solo_apply;
mod stack_demo_probe;
mod stack_trailing_release;
mod strip_marks;
mod the_noise_seed_is_stable;
mod track_extrapolation;
