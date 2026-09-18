//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 59 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_resposta_do_filtro_ao_arrasto;
mod cada_rotulo_deste_motor_vem_da_tabela;
mod cotangent_operator;
mod measure_alpha;
mod measure_alpha_over_a_whole_mesh;
mod measure_anchor_law;
mod measure_brush_kernel;
mod measure_clay_thumb;
mod measure_curvature_normal;
mod measure_directional_wash;
mod measure_draw_sharp;
mod measure_dyntopo_spikes;
mod measure_field_cost;
mod measure_field_verbs;
mod measure_grab_modes;
mod measure_hardness_and_falloff;
mod measure_inflate_normal_drift;
mod measure_kelvinlet;
mod measure_layer_comb;
mod measure_layer_front_face;
mod measure_layer_law;
mod measure_layer_zoom_and_flank;
mod measure_mls_plane;
mod measure_multiplane_scrape;
mod measure_offset_at_the_operating_point;
mod measure_path_invariance;
mod measure_pinch_family_modes;
mod measure_pull_profile;
mod measure_raycast_feedback;
mod measure_reach_by_mode;
mod measure_reference_divergence;
mod measure_saturation;
mod measure_scale_filter;
mod measure_sharpen_filter;
mod measure_sharpen_intensify;
mod measure_sharpen_law;
mod measure_sharpen_valence;
mod measure_slide_relax;
mod measure_smooth_shrinkage;
mod measure_smoothing_power;
mod measure_stroke_ripple;
mod measure_surface_smooth;
mod measure_taubin_lambda;
mod measure_the_walk_loses_dabs;
mod measure_transform;
mod measure_valley;
mod measure_where_the_curve_knobs_reach;
mod mede_o_esfregao;
mod mede_o_filtro_de_tecido;
mod mede_o_projectar;
mod mede_o_raio_do_pincel;
mod mede_o_tecido_que_atravessa_os_gestos;
mod mede_o_tecido_que_estica;
mod o_produto_corre_a_lei;
mod oraculo_do_esfregao;
mod oraculo_do_pincel_afiado;
mod oraculo_do_pincel_afiado_produto;
mod oraculo_do_pincel_afiado_silhueta;
mod oraculo_do_pincel_de_plano;
mod oraculo_do_pincel_de_plano_fabrica;
mod oraculo_do_projectar;
mod oraculo_dos_gestos_tangenciais;
mod probe_cloth_front;
mod probe_layer_product;
mod sculptgl_parity;
mod sonda_da_onda_da_prega;
mod sonda_do_expand_que_nao_para;
mod sonda_do_falloff_pela_superficie;
mod taubin_pair;
mod the_frame_is_hoisted_out_of_the_vertex_loop;
mod the_stamp_is_pinned_to_the_view;
