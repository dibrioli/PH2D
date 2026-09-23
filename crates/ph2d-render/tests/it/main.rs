//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 43 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

#[path = "../fx_stack_common/mod.rs"]
mod fx_stack_common;

mod a_mistura_do_device_chega_ao_pixel;
mod architecture_sprite_inspector_surface;
mod blend_mode_regression;
mod bloom_wgsl_valid;
mod camera_screen_to_world;
mod clip_children_regression;
mod fx_look_probe;
mod fx_scene_scale_cost;
mod fx_stack_adjust_gpu;
mod fx_stack_atlas_gpu;
mod fx_stack_bevel_gpu;
mod fx_stack_blend_gpu;
mod fx_stack_duotone_gpu;
mod fx_stack_feather_gpu;
mod fx_stack_gpu;
mod fx_stack_gradient_map_gpu;
mod fx_stack_kinds_gpu;
mod fx_stack_linear_gpu;
mod fx_stack_modes_gpu;
mod fx_stack_morphology_gpu;
mod fx_stack_segment_cost;
mod fx_stack_turbulence_gpu;
mod generate_v3_fixtures;
mod gpu_gates_are_not_vacuous;
mod impasto_light_gpu;
mod individual_readback;
mod individual_texture_honours_its_sampling;
mod ktx2_format_exhaustive_mapping;
mod layer_compositor_gpu;
mod layers_no_alloc;
mod mask_interaction_regression;
mod measure_first_stroke_pipelines;
mod migrate_sprite_v3_to_v4;
mod o_compositor_e_meia_cobertura;
mod precision_parity_gpu;
mod preview_premul_gpu;
mod render_instance_pod_size_v4;
mod skin_mesh_gpu_ceiling;
mod skin_pieces_gpu_cost;
mod smoke_fixture_renderable;
mod spatial_weights_parity;
mod sprite_mesh_gpu;
mod sprite_premul;
mod sprite_versioned_postcard;
mod sprite_wgsl_valid;
mod the_gpu_extra_draw_binds_per_texture_run;
mod the_pass_aa_is_never_chosen_by_a_text_preference;
mod tonemap_descent_gpu;
