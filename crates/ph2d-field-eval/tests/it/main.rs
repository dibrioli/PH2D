//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 68 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

#[path = "../common/mod.rs"]
mod common;

mod a_modifier_acts_on_the_axis_it_is_told;
mod a_modifier_either_moves_the_field_or_offers_a_number;
mod a_row_of_holes_is_already_expressible;
mod audit_the_new_junctions;
mod every_trio_of_modifiers_keeps_the_field_marchable;
mod measure_cloud_lobes;
mod measure_gear_teeth;
mod measure_polygon_vertices;
mod measure_prism_sides;
mod measure_sharp_edges;
mod measure_star_points;
mod measure_the_band_shoulder;
mod measure_the_chain_of_fillets;
mod measure_the_wall_after_a_warp;
mod measure_twist_cost;
mod probe_bezier;
mod probe_cloud_report;
mod probe_exp_ln_accuracy;
mod probe_gielis;
mod probe_is_a_polygon_already_reachable;
mod probe_knot;
mod probe_mirror_pairs;
mod probe_superquadric;
mod probe_tag_at_the_fence;
mod probe_the_analytic_gradient_blocker;
mod probe_the_infinite_piece;
mod probe_the_other_junctions;
mod probe_the_star_chamfer;
mod probe_thread;
mod probe_triangle_thin;
mod probe_w119_lote;
mod probe_w122_flow;
mod probe_w123_curves;
mod spike_chamfer_then_fillet;
mod spike_formula_vs_profile;
mod spike_join_between_copies;
mod spike_per_edge_radius;
mod the_bend_is_a_minorant;
mod the_box_of_a_bound_contains_the_piece;
mod the_census_of_every_primitive;
mod the_coil_and_the_lattice;
mod the_composed_jacobian_is_not_the_product;
mod the_divisor_is_global_but_the_march_is_local;
mod the_fillet_is_a_true_arc_at_every_angle;
mod the_four_characters;
mod the_four_shapes_of_the_flowchart_lote;
mod the_joint_between_copies;
mod the_knot_that_two_counts_make;
mod the_march_clip_does_not_touch_the_piece;
mod the_march_reads_the_verb_of_each_shape;
mod the_nine_shapes_of_the_symbol_lote;
mod the_point_probe_is_the_same_answer;
mod the_seam_characters;
mod the_shape_that_bulges;
mod the_shape_that_is_a_whole_catalogue;
mod the_shape_that_morphs_a_family;
mod the_shape_the_prism_cannot_make;
mod the_shape_with_as_many_corners_as_you_want;
mod the_six_shapes_of_the_arrow_lote;
mod the_stack_composes_without_tearing;
mod the_star_the_frame_and_the_ellipsoid;
mod the_thread_that_winds_a_cylinder;
mod the_three_new_solids;
mod the_turns_slider_has_a_dead_top;
mod the_twist_is_a_minorant;
mod the_two_curves_with_thickness;
mod the_two_shapes_that_left_the_drawing;
mod the_two_solids_of_revolution;
