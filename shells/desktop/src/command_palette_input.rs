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

    /// Backspace in the palette's search box.
    pub(crate) fn command_palette_backspace(&mut self) {
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            h.store.command_palette_backspace();
        }
    }

    /// Enter: pick the TOP filtered match (the Motion bridge routes it to an `AddNode` next frame, exactly
    /// like a mouse pick). No-op when the search is empty or matches nothing — Enter never adds a node the
    /// artist did not narrow to, and `top_match` is the SAME predicate the widget paints the filter with.
    pub(crate) fn command_palette_confirm(&mut self) {
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            let store = &mut h.store;
            let picked = store.command_palette_model().and_then(|model| {
                ph2d_editor::widget::command_palette::top_match(model, store.command_palette_query())
            });
            if let Some(id) = picked {
                store.set_command_pick(id);
                store.close_command_palette();
            }
        }
    }
}
