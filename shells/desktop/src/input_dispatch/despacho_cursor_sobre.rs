//! **O cursor está sobre que painel?** ([`super`]) — as duas perguntas que roteiam o meio-arrasto e o
//! teclado pela área sob o cursor; mudadas VERBATIM do irmão dos picks e arrastos pelo tecto de 600.

impl crate::App {
    /// Motion Nodes M1: is the cursor over the docked graph panel? Drives the
    /// cursor-gated graph keyboard focus + middle-pan routing (Blender-style F
    /// acts on the hovered area, graph vs scene).
    pub(crate) fn cursor_over_motion_graph(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.panel_rect_fisico(ph2d_editor_core::ids::MOTION_GRAPH_PANEL))
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }

    /// W2.E6: is the cursor over the general timeline dock? Mirrors
    /// [`Self::cursor_over_motion_graph`] — a middle-drag there pans the
    /// dope-sheet (via its `TimelineSurface` gesture), not the camera behind it.
    /// Blender-style: the hovered component owns the zoom/pan.
    pub(crate) fn cursor_over_timeline(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.panel_rect_fisico(ph2d_editor_core::ids::TIMELINE_PANEL))
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }
}
