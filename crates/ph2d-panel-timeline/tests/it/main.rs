//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 14 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_coluna_do_toggle_mede_a_lista;
mod buffer_curves_seam;
mod close_button_seam;
mod containers_tab_seam;
mod duration_chip_gesture;
mod every_word_this_panel_shows_comes_from_the_string_table;
mod extrapolation_seam;
mod marker_menu_seam;
mod nesting_seam;
mod one_door_names_the_channel;
mod seam;
mod strip_ease_grip_seam;
mod transport_motion_path_seam;
mod transport_onion_seam;
mod transport_physics_seam;
mod view_tabs_seam;
