//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 69 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.
//! ⛔ Excepção: um ficheiro com `#[global_allocator]` tem de ser binário PRÓPRIO (dois alocadores
//! globais não cabem num binário, e um contador global veria as alocações dos vizinhos): fica em `tests/`.

mod air_drag;
mod area_falloff;
mod area_frame;
mod body_defaults;
mod buoyancy;
mod buoyed_query;
mod capsule;
mod ccd;
mod checkpoint;
mod collider_offset;
mod combine_rules;
mod compound;
mod contacts;
mod damping;
mod dominance;
mod effector;
mod ellipse_collider;
mod grab;
mod gravity_scale;
mod initial_velocity;
mod joint_break;
mod joint_custom;
mod joint_motor;
mod joint_pair;
mod joint_retune;
mod joint_rod;
mod joint_soft_weld;
mod joint_wheel;
mod joints;
mod kinematic_substeps;
mod lock_rotation;
mod lock_translation;
mod mass_override;
mod measure_impact;
mod measure_joint_break;
mod measure_joint_pair;
mod measure_pulley;
mod measure_pulley_bias_radius;
mod measure_pulley_budget;
mod measure_pulley_composition;
mod measure_pulley_differential;
mod measure_pulley_kinematic_sheave;
mod measure_pulley_tackle;
mod measure_rod;
mod measure_rope_route;
mod measure_rope_stop;
mod measure_settings;
mod measure_soft_weld;
mod measure_stop_sideways;
mod measure_the_compound_push;
mod measure_the_swim_threshold;
mod measure_weston;
mod measure_wheel;
mod one_way;
mod oraculo_da_pilha_do_motion;
mod penetration;
mod pulley;
mod pulley_break;
mod pulley_composition;
mod pulley_differential;
mod pulley_kinematic_sheave;
mod pulley_tackle;
mod pulley_weston;
mod pulley_winch;
mod rope_stop;
mod sensors;
mod snap_points;
mod the_old_math_vocabulary_is_gone;
mod zone_push;
