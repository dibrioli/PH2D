//! "Sync with other tools" — the per-tool brush-settings model. A submodule of `paint`, split from
//! `paint.rs`/`stencil.rs` for the workspace LOC cap.
//!
//! By DEFAULT every paint tool (Brush / Smear / Blur / Clone / Mask / Inpaint / Fill) is INDEPENDENT:
//! each [`super::PaintMode`] keeps its own [`ph2d_painter_brush::BrushSpec`] in `PaintState::brush_by_mode`,
//! swapped into the live `brush` on a mode change ([`PainterTool::switch_brush_slot`], driven by
//! `set_paint_tool_mode`). So editing the panel while in Mask never bleeds into the Brush panel.
//!
//! The **"Sync with other tools"** checkbox flips `link_shared_settings`. While ON, a mode change no
//! longer swaps slots, so all tools share the live `brush` — the panel where it was checked immediately
//! configures the others (they show `brush` on switch). Turning it OFF snapshots the current (shared)
//! settings into every mode's slot, so each tool keeps that as its starting point and then diverges.

use crate::tool::PainterTool;

impl PainterTool {
    /// Swap the active brush settings to `new_mode`'s slot (the independent-tools model). No-op when the
    /// settings are LINKED (all modes share the live `brush`) or the mode is unchanged. Saves the current
    /// mode's live edits back to its slot first, then loads the target mode's saved settings.
    pub(crate) fn switch_brush_slot(&mut self, new_mode: super::PaintMode) {
        if self.paint.link_shared_settings || new_mode == self.paint.paint_mode {
            return;
        }
        let old = self.paint.paint_mode.slot();
        let new = new_mode.slot();
        self.paint.brush_by_mode[old] = self.paint.brush; // save this tool's edits
        self.paint.brush = self.paint.brush_by_mode[new]; // restore the target tool's settings
        // Falloff provenance for the freshly loaded brush: the factory `Smooth` is presumed armable
        // (the pre-existing ambiguity — an explicit artist Smooth is indistinguishable from factory);
        // anything else in a slot is presumed the artist's until an arm installs over it.
        self.paint.falloff_armed = self.paint.brush.falloff == ph2d_painter_brush::Falloff::Smooth;
    }

    /// Whether brush settings are currently linked across tools (the "Sync with other tools" checkbox).
    #[must_use]
    pub fn link_shared_settings(&self) -> bool {
        self.paint.link_shared_settings
    }

    /// Toggle "Sync with other tools". Turning it ON shares the live `brush` across every tool (this
    /// panel's current settings, since a mode switch no longer swaps). Turning it OFF snapshots the
    /// current shared settings into every mode's slot, so each tool keeps them as its own starting point
    /// and then diverges. (Simple field writes — no derived cache to settle.)
    pub fn toggle_link_shared_settings(&mut self) {
        self.paint.link_shared_settings = !self.paint.link_shared_settings;
        if !self.paint.link_shared_settings {
            // Unlinking: every mode keeps what's currently on screen (all were showing `brush`).
            let current = self.paint.brush;
            for slot in &mut self.paint.brush_by_mode {
                *slot = current;
            }
        }
        // Linking needs no seeding: while linked, a mode switch keeps the live `brush`, so every tool
        // immediately shows this (the checked) panel's settings.
    }
}
