//! **Como esta shell responde ao [`ph2d_app_host::AppHost`]** — o único sítio onde a `App` diz o
//! que uma família de módulo pode saber dela.
//!
//! ⚠️ **Cinco métodos, e cada um devolve um VALOR.** Nenhum devolve a `App`, o `AppGfx` ou o
//! `HeroScreen`: um `fn gfx(&self) -> &AppGfx` desfaria a fronteira inteira, porque a família
//! voltaria a poder tudo — e voltaria a recompilar sempre que a shell mudasse, que é a metade do
//! problema que esta obra existe para resolver.

use ph2d_app_host::{AppHost, HostMods};
use ph2d_editor::zones::Rect;

use crate::app_state::App;

impl AppHost for App {
    fn pointer(&self) -> (f32, f32) {
        self.last_pointer
    }

    fn mods(&self) -> HostMods {
        HostMods {
            shift: self.modifiers.shift_key(),
            control: self.modifiers.control_key(),
            alt: self.modifiers.alt_key(),
            super_key: self.modifiers.super_key(),
        }
    }

    fn pointer_over_chrome(&self, x: f32, y: f32) -> bool {
        crate::chrome_hit::pointer_over_chrome(self.gfx.as_ref(), x, y)
    }

    fn modal_takes_the_pointer(&self) -> bool {
        self.command_palette_open()
    }

    fn note_authored_change(&mut self) {
        self.any_input_this_frame = true;
    }

    fn canvas_visible(&self, viewport: Rect) -> Rect {
        // ⚠️ Sem `HeroScreen` publicado ainda, a área é a janela — que é o que a porta já fazia.
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map_or(viewport, |h| {
                ph2d_app_host::canvas_area::visible(h, viewport)
            })
    }
}
