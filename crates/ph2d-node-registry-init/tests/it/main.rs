//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 78 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

#[path = "../common/mod.rs"]
mod common;

mod a_node_that_reads_the_clock_is_temporal;
mod an_external_reader_never_reaches_the_device_blind;
mod arbitrary_selection;
mod boids_ceiling;
mod cel_animation_laws_the_graph_already_has;
mod collide_ceiling;
mod dead_knob_sweep;
mod death_replicates;
mod falloff_declaration;
mod from_wire_units;
mod generators_consume_accel;
mod heavy_reading_never_reaches_the_cook;
mod instance_ceiling_agrees;
mod integrator_ceilings;
mod kill_radius;
mod layout_affine_factorisation;
mod lsystem_is_a_real_skeleton;
mod measure_boids_and_wave_ceilings;
mod measure_boids_ceiling;
mod measure_clone_multisource;
mod measure_collide_ceiling;
mod measure_collider_shapes;
mod measure_combine_arity;
mod measure_envelope_and_kill_volume;
mod measure_identity_after_structure;
mod measure_instance_ceiling;
mod measure_integrate_substeps;
mod measure_layer_seed;
mod measure_named_coefficients;
mod measure_node_params;
mod measure_noise_space;
mod measure_path_controls;
mod measure_pick_instances;
mod measure_preroll;
mod measure_rope_ceiling;
mod measure_size_axes;
mod measure_spawn_probability;
mod measure_spin_authoring;
mod measure_spring_ceiling;
mod measure_stream_join_defects;
mod measure_substeps;
mod measure_substeps_name_collision;
mod measure_switch_arity;
mod measure_switch_laziness;
mod measure_value_reach;
mod measure_wave_edges;
mod measure_wave_producers;
mod measure_zone_life_cycle;
mod metric_vocabulary;
mod no_value_op_is_trigonometric;
mod normalized_age;
mod param_ceilings;
mod param_census;
mod param_gates_are_exact;
mod param_range_conventions;
mod param_widget_conventions;
mod path_controls;
mod population_cap;
mod probe_live_params_and_overlap;
mod proximity_reaches_its_readers;
mod pulse_edge_vocabulary;
mod pulse_level_chains;
mod rope_ceiling;
mod rope_thickness;
mod soft_body_radius;
mod spacing_scene;
mod spring_ceiling;
mod staleness;
mod substeps;
mod substeps_integrate;
mod the_corner_question_has_one_vocabulary;
mod the_falloff_reaches_the_soft_body;
mod the_pin_reaches_the_sims;
mod the_pivot_question_has_one_vocabulary;
mod time_port;
mod velocity_reaches_its_readers;
mod wave_producers;
mod wind_vocabulary;
