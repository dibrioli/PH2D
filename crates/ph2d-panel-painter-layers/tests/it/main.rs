//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 19 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod cada_nome_de_marcar_cabe_na_coluna_da_seccao;
mod curve_handle_menu_e2e;
mod every_word_this_panel_shows_comes_from_the_string_table;
mod falloff_drain_repro;
mod falloff_handle_menu_e2e;
mod seam;
mod seam_curve_drag_ownership;
mod seam_deform;
mod seam_dock_modes;
mod seam_grid_stamp;
mod seam_impasto_rig;
mod seam_impasto_tool;
mod seam_line_card;
mod seam_paint_media;
mod seam_sculpt;
mod seam_shape_deposit;
mod seam_spray;
mod seam_substrate;
mod seam_taper;
mod seam_texture_colors;
mod seam_wetpaint;
