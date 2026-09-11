//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 28 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_long_popover_scrolls;
mod action_verb_is_a_dropdown;
mod every_form_row_reserves_the_animation_column;
mod every_painted_id_is_reachable;
mod inspector_regression;
mod inspector_regression_anchors;
mod inspector_regression_sections;
mod inspector_regression_slice;
mod seam;
mod seam_anim;
mod seam_apply_ladder;
mod seam_joint;
mod seam_open_prefab;
mod seam_orphan_list;
mod seam_physics;
mod seam_player;
mod seam_precision;
mod seam_properties;
mod seam_render_source;
mod seam_texture_slot;
mod seam_wheel;
mod the_add_component_button_follows_the_selection;
mod the_audio_section_is_alive;
mod the_camera_section_is_alive;
mod the_ordering_labels_come_from_the_descriptor;
mod the_sheet_grid_switch_is_offered_only_where_there_is_a_grid;
mod the_slice_hints_take_the_room_they_actually_use;
mod two_sections_never_stack;
