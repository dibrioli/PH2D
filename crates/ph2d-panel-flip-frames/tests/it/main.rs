//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 2 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod close_button_seam;
mod every_word_this_panel_shows_comes_from_the_string_table;
mod seam;
mod os_cartoes;

/// O corpo de antes (`PH2D_CARDS=0`), que estes gates descrevem; o cromo prova-se em `os_cartoes`.
pub(crate) fn host_de_antes() -> ph2d_ui_testkit::MockPanelHost {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(false));
    ph2d_ui_testkit::MockPanelHost::with_panel::<ph2d_panel_flip_frames::FlipFramesPanel>()
}
