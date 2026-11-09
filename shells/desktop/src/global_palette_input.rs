//! **A paleta de comandos GLOBAL, do lado da shell** — o `Ctrl+K` e o dreno do pick.
//!
//! O modelo e a execução vivem em `ph2d_editor_core::screens::hero::global_palette`; as linhas do
//! Pixel (Run Command, P12 R) em `ph2d_app_pixel::comando`. Aqui ficam só a TECLA e o DRENO.
//!
//! ⚠️ **O dreno é CONDICIONAL**: o canal do pick tem vários consumidores (este, o do Pixel e o
//! `route_palette_pick` do Motion); cada um reconhece os próprios ids (`take_command_pick_if`) e
//! deixa o resto, para que a ORDEM dos drenos no frame não decida quem recebe o pick.

use crate::App;
use ph2d_app_pixel::{PixelApp, comando, input::KeyAnswer};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::global_palette;

/// `Ctrl+K` — abre a paleta sobre o que o app oferece AGORA (construída na abertura, nunca por
/// frame). Toma o `HeroScreen` e o Pixel, e não o `App`: o `match` de teclas já tem o `gfx`
/// emprestado.
pub(crate) fn open_global_palette(hero: &mut HeroScreen, pixel: &mut PixelApp) {
    global_palette::open_with(hero, comando::group(pixel));
}

impl App {
    /// Drena o pick DESTA paleta e executa-o. Chamado uma vez por frame; o modelo reconstrói-se
    /// para reconhecer o pick (ele chega com a paleta já fechada).
    pub(crate) fn global_palette_drain(&mut self) {
        let Some(g) = self.gfx.as_mut() else { return };
        let Some(h) = g.hero_screen.as_mut() else {
            return;
        };
        if let Some(id) = h
            .store
            .take_command_pick_if(|id| comando::action_of(id).is_some())
            && let Some(KeyAnswer::Toast(t)) = comando::run(&mut self.pixel, id)
        {
            g.toasts.push(ph2d_editor_core::Toast::info(t));
        }
        global_palette::drain(h);
    }
}
