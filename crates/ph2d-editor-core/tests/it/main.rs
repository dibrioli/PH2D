//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 108 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.
//! ⛔ Excepção: um ficheiro com `#[global_allocator]` tem de ser binário PRÓPRIO (dois alocadores
//! globais não cabem num binário, e um contador global veria as alocações dos vizinhos): fica em `tests/`.

#[path = "../common/cfg_test_modules.rs"]
mod cfg_test_modules;
#[path = "../common/hero_sources.rs"]
mod hero_sources;

mod a_button_row_is_laid_out_by_the_door;
mod a_column_closes_by_the_gesture_that_resizes_it;
mod a_label_that_does_not_fit_is_elided_not_wrapped;
mod a_list_is_not_a_form;
mod a_panel_scrolls_by_dragging_its_body;
mod a_ring_painted_as_a_fill_is_still_a_frame;
mod arch_color_space_typed;
mod arch_mode_has_reconcile;
mod arch_no_absolute_drag_pattern;
mod arch_no_char_count_widths;
mod arch_safe_clamp_only;
mod arch_shape_slot_uses_the_shape_door;
mod architecture_adr_numbers_are_unique;
mod architecture_chrome_dispatch_in_sync;
mod architecture_curve_drag_asks_whose_gesture;
mod architecture_cycle_prevention;
mod architecture_docs_paths_and_smokes_resolve;
mod architecture_docs_reference_live_gates;
mod architecture_every_panel_is_painted;
mod architecture_interactive_crate_has_behavioral_test;
mod architecture_motion_chrome_never_wraps_a_row_label;
mod architecture_msrv_is_the_pinned_toolchain;
mod architecture_no_chip_without_steppers;
mod architecture_no_orphan_source_file;
mod architecture_no_restricted_source_citations;
mod architecture_panel_host_surface;
mod architecture_panel_loc_cap;
mod architecture_panel_wiring_parity;
mod architecture_stack_versions_doc_matches_the_lockfile;
mod architecture_the_shell_only_shrinks;
mod architecture_tool_contract_surface;
mod architecture_topbar_registration_parity;
mod architecture_widget_loc_cap;
mod architecture_widget_mod_in_sync;
mod architecture_widget_showcase_coverage;
mod architecture_workspace_file_loc_cap;
mod canonical_icon_button;
mod docs_bugs_have_gates;
mod every_button_wears_the_live_hover;
mod every_frame_goes_through_the_theme_door;
mod every_menu_row_is_registered;
mod every_menu_row_reaches_a_handler;
mod every_new_image_choice_is_alive_under_the_mouse;
mod every_skeleton_control_declares_whose_subject_it_is;
mod every_stack_of_rows_asks_the_rhythm;
mod every_widget_event_variant_has_a_producer_and_a_reader;
mod grid_toggle;
mod hr12_widgets_a11y;
mod hr15_no_hardcoded_ui_strings;
mod make_square_algorithm;
mod measure_ui_motion;
mod no_effect_inside_debug_assert;
mod no_label_of_this_crate_is_written_in_the_painter;
mod no_literal_color;
mod no_magic_numeric;
mod no_tofu_glyphs;
mod node_id_collisions;
mod nothing_inside_a_section_wears_the_section_tone;
mod number_input_focus_replaces;
mod number_input_mapped_link;
mod the_animation_column_has_one_x;
mod the_app_default_slider_style_is_the_one_the_owner_chose;
mod the_bar_relocated_every_row_of_the_menus_it_replaced;
mod the_boundary_of_a_section_is_a_card;
mod the_chip_axis_has_one_door;
mod the_chrome_never_eats_more_of_a_tablet_than_this;
mod the_chrome_reads_the_ui_clock;
mod the_corner_of_a_control_comes_from_the_theme;
mod the_dock_border_resizes_the_column;
mod the_drag_band_never_reaches_the_close_button;
mod the_face_of_a_tab_reaches_the_scene;
mod the_family_hover_map_agrees_with_the_button_kinds;
mod the_file_menu_items_are_not_mute;
mod the_fill_lands_under_the_cursor;
mod the_flat_surface_reads_the_clock;
mod the_form_has_one_right_margin;
mod the_gap_between_an_icon_and_its_label;
mod the_gap_between_two_rows_is_one_answer;
mod the_ground_paints_around_the_area_not_under_it;
mod the_hero_paint_docks_the_timeline_into_motion;
mod the_hover_axis_never_flashes_the_far_end;
mod the_indent_of_a_child_is_one_number;
mod the_input_map_window_binds_a_key;
mod the_input_map_window_is_painted_where_it_says;
mod the_inspector_is_open_when_the_app_opens;
mod the_intent_drain_reaches_every_variant;
mod the_look_is_a_widget_skin_never_an_area_model;
mod the_menu_bar_relocates_the_verbs_it_shows;
mod the_modern_family_paints_fewer_frames;
mod the_painted_control_reaches_a_consumer;
mod the_panel_row_reads_the_sliders_state;
mod the_physics_pill_opens_the_physics_panel;
mod the_pointer_and_the_clock_agree_on_who_lights_up;
mod the_rail_label_band_is_one_number;
mod the_rail_names_a_consumer_for_every_chip;
mod the_redesign_wears_the_godot_family;
mod the_row_height_is_one_number;
mod the_ruler_prints_the_projects_unit;
mod the_rulers_never_share_a_pixel_with_docked_chrome;
mod the_sheet_resolution_modal_is_alive;
mod the_side_columns_are_anchored;
mod the_tab_row_recedes_and_the_chosen_tab_rises;
mod the_tail_of_a_block_is_one_answer;
mod the_tokens_pill_opens_the_tokens_panel;
mod the_tool_bar_is_a_region_of_the_area;
mod the_top_column_wears_the_flat_skin;
mod the_two_looks_are_one_switch_apart;
mod the_ui_pill_opens_the_authored_panel;
