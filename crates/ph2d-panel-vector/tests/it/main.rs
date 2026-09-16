//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 44 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod architecture_sections_read_the_document;
mod bool_registration_parity;
mod connector_section;
mod every_chip_is_linked_to_its_slider;
mod every_word_this_panel_shows_comes_from_the_string_table;
mod fill_chip_labels_fit;
mod marker_rows;
mod popover_band;
mod probe_gutters;
mod seam;
mod seam_anchors;
mod seam_arrange_z;
mod seam_bool;
mod seam_bucket;
mod seam_components;
mod seam_contour;
mod seam_filters;
mod seam_frame;
mod seam_layout;
mod seam_layout_grid_item;
mod seam_marquee;
mod seam_morph_states;
mod seam_node_xy;
mod seam_paint_stack;
mod seam_pencil;
mod seam_pencil_knobs;
mod seam_resize_box;
mod seam_signals;
mod seam_snap;
mod seam_spring;
mod seam_stroke_paint;
mod seam_stroke_present;
mod seam_symmetry_segments;
mod seam_text_axes;
mod seam_text_wrap;
mod seam_text_wrap_on_path;
mod seam_texture_pattern;
mod seam_tokens;
mod seam_ui_states;
mod seam_vertex;
mod seam_weld;
mod seam_widget;
mod section_headers_are_collapsible;
mod shape_params_visibility;
mod the_panel_shows_what_the_tool_is_for;
