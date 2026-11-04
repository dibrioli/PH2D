//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 3 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod catalogue;
mod every_word_this_panel_shows_comes_from_the_string_table;
mod geometry;
mod seam;
mod os_cartoes;

/// O corpo de antes (`PH2D_CARDS=0`): é ele que as varreduras antigas medem.
fn corpo_de_antes() -> ph2d_ui_testkit::MockPanelHost {
    ph2d_panel_widget_lab::cards::force_cards_on_this_thread(Some(false));
    ph2d_ui_testkit::MockPanelHost::with_panel::<ph2d_panel_widget_lab::WidgetLabPanel>()
}
