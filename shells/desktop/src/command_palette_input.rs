//! Shell-side glue for the full-screen "Add Node" command palette (Motion's picker, an
//! `ph2d-editor-core` chrome widget). The palette is a MODAL: while it is open the keyboard handler
//! ([`crate::input_dispatch::keyboard`]) routes every key here — printable characters build its search
//! text, Enter picks the top filtered match, Backspace deletes, Escape closes. These are thin doors onto
//! the store ops; the widget filters + highlights, and the shell's Motion bridge drains the pick and
//! turns it into an `AddNode` (mirroring a mouse pick, so Enter and click take the exact same path).

use crate::App;

impl App {
    /// Is the full-screen Add-Node palette open? (Its keys are captured before every other shortcut.)
    pub(crate) fn command_palette_open(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.command_palette_open())
    }

    /// Close the open palette (Escape / done).
    pub(crate) fn command_palette_close(&mut self) {
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            h.store.close_command_palette();
        }
    }

    /// Feed one typed character to the palette's search box (control chars are dropped by the store).
    pub(crate) fn command_palette_type(&mut self, c: char) {
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            h.store.command_palette_push_char(c);
        }
    }

    /// ↑ ↓ move the highlight (palettes that highlight before picking).
    pub(crate) fn command_palette_step(&mut self, delta: isize) {
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            h.store.step_command_palette_highlight(delta);
        }
    }

    /// Backspace in the palette's search box.
    pub(crate) fn command_palette_backspace(&mut self) {
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            h.store.command_palette_backspace();
        }
    }

    /// Enter: pick the highlighted item (Add Object) or the TOP filtered match — routed next frame exactly
    /// like a mouse pick; no-op when nothing is highlighted or matched.
    pub(crate) fn command_palette_confirm(&mut self) {
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            let store = &mut h.store;
            if let Some(id) = store.command_palette_enter_target() {
                store.set_command_pick(id);
                store.close_command_palette();
            }
        }
    }
}

impl App {
    /// `Alt+Enter` na busca global: em vez de executar, mostra onde o comando mora.
    pub(crate) fn command_palette_show_where(&mut self) {
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut())
            && h.store.command_palette_model().is_some_and(|m| m.is_list())
            && let Some(id) = h.store.command_palette_enter_target()
        {
            ph2d_editor_core::screens::hero::menu_reveal::show(h, id);
        }
    }

    /// ⭐ **A roda pertence à PALETA enquanto ela estiver aberta** — ver
    /// [`ph2d_app_components::component_attach::palette_wheel`], onde mora o porquê.
    pub(crate) fn command_palette_wheel(&mut self, dy: f32) -> bool {
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        ph2d_app_components::component_attach::palette_wheel(hero, dy)
    }
}
