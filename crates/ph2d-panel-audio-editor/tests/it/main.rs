//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 3 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod as_seccoes_arrastam_e_tem_tema; // o tema e o arrasto das secções do corpo
mod every_word_this_panel_shows_comes_from_the_string_table;
mod nenhuma_fileira_fora_da_porta;
mod no_dead_buttons;
mod seam;
mod the_audio_editor_asks_the_store_how_its_buttons_look;
mod os_cartoes;

/// O painel de `PH2D_CARDS=0`: os testes de antes descrevem esse corpo.
pub(crate) fn host_de_antes() -> ph2d_ui_testkit::MockPanelHost {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(false));
    ph2d_ui_testkit::MockPanelHost::with_panel::<ph2d_panel_audio_editor::AudioEditorPanel>()
}
