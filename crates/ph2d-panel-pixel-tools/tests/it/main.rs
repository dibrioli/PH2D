//! Os gates de COSTURA do painel PIXEL — gesto REAL (`ph2d_ui_testkit`), nunca um `WidgetEvent`
//! sintético.
mod every_word_this_panel_shows_comes_from_the_string_table;
mod os_cartoes;
mod seam_pixel;
mod seam_pixel_cor;
mod seam_pixel_formas;
mod seam_pixel_gradiente;
mod seam_pixel_paleta_voo;
mod seam_pixel_pincel;
mod seam_pixel_selecao;
mod seam_pixel_texto;
mod seletor_arrasto;
mod seletor_leis;

/// O corpo de antes (`PH2D_CARDS=0`), que estes gates descrevem; os cartões provam-se em `os_cartoes`.
pub(crate) fn host_de_antes() -> ph2d_ui_testkit::MockPanelHost {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(false));
    ph2d_ui_testkit::MockPanelHost::with_panel::<ph2d_panel_pixel_tools::PixelToolsPanel>()
}
