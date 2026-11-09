//! **O cursor está sobre que painel?** ([`super`]) — as duas perguntas que roteiam o meio-arrasto e o
//! teclado pela área sob o cursor; mudadas VERBATIM do irmão dos picks e arrastos pelo tecto de 600.

impl crate::App {
    /// Motion Nodes M1: cursor over the graph panel (its keys and middle-pan, Blender-style).
    pub(crate) fn cursor_over_motion_graph(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.panel_rect_fisico(ph2d_editor_core::ids::MOTION_GRAPH_PANEL))
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }

    /// W2.E6: cursor over the timeline dock (dope-sheet pan, and the Pixel cedes its keys).
    pub(crate) fn cursor_over_timeline(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.panel_rect_fisico(ph2d_editor_core::ids::TIMELINE_PANEL))
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }
}
