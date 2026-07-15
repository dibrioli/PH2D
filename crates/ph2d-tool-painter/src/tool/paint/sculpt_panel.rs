//! The Sculpt card's **seam with the panel** — the read-only accessors it paints from, and the router that
//! turns its events back into tool calls.
//!
//! Split from [`super::sculpt`] (the model) for the workspace file-LOC cap, and it splits cleanly: nothing
//! here decides anything. Every accessor is the SAME function the kernel calls, so the number on the chip
//! cannot drift from the number in the arithmetic — that is the whole reason they are functions and not
//! fields ([[feedback_two_doors_to_the_same_question_diverge]]).

use super::sculpt::SculptMode;
use crate::tool::PainterTool;

impl PainterTool {
    /// Whether the active operation is **Sculpt** — the panel shows the Sculpt card (and hides the colour
    /// controls, which a tool that lays no pigment has no use for).
    #[must_use]
    pub fn is_sculpt_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Sculpt)
    }

    /// Which knob row the card must paint: `0` Radius (Smooth family) · `1` Offset (Plane) · `2` Depth
    /// (Height). The Chisel additionally shows Angle — see [`Self::is_sculpt_chisel`].
    ///
    /// The card shows the knobs the active verb USES and no others. A knob that does nothing to the tool in
    /// your hand is a knob that lies about what the tool can do, and this card has already cost a smoke over
    /// exactly that class of mistake.
    #[must_use]
    pub fn sculpt_knob_family(&self) -> u8 {
        self.paint.sculpt.mode_enum().knob_family()
    }

    /// Whether the active verb is the **Chisel** — the one verb with two knobs (Offset *and* Angle).
    #[must_use]
    pub fn is_sculpt_chisel(&self) -> bool {
        matches!(self.paint.sculpt.mode_enum(), SculptMode::Chisel)
    }

    /// The kernel radius in px — what the Radius chip shows the artist, and what the kernel actually uses.
    /// One function, so the number on screen cannot drift from the number in the blur.
    #[must_use]
    pub fn sculpt_radius_px(&self) -> u32 {
        self.paint.sculpt.radius_px()
    }

    /// The plane Offset in paint-loads — what the Offset chip shows, and what the kernel adds. One function,
    /// same reason.
    #[must_use]
    pub fn sculpt_plane_offset(&self) -> f32 {
        self.paint.sculpt.plane_offset()
    }

    /// The Height family's Depth in paint-loads — the chip's number and the kernel's, one function.
    #[must_use]
    pub fn sculpt_depth(&self) -> f32 {
        self.paint.sculpt.depth()
    }

    /// The Chisel's Angle in degrees — the chip's number and the kernel's, one function.
    #[must_use]
    pub fn sculpt_chisel_angle_deg(&self) -> f32 {
        self.paint.sculpt.chisel_angle_deg()
    }

    /// Tell the stroke engine whether this stroke's dabs are useless without a **heading**.
    ///
    /// The Chisel carves a V about the stroke's axis, so it reads `Dab::dir`. The engine holds the opening
    /// dabs of such a stroke until travel has settled a direction (the rake **warm-up**) — but it only knew
    /// to do that for the two texture slots. Without this, a chisel's first dabs arrive with `dir = [0, 0]`
    /// and carve **nothing** (the V degenerates to Scrape): the groove starts blunt, every single stroke.
    ///
    /// Called from the two places that can change the answer — this card's verb, and the paint-mode switch
    /// (which swaps the whole brush slot underneath us). One function, so the two cannot disagree.
    pub(super) fn sync_stroke_heading_need(&mut self) {
        // BOTH Rake modes read `Dab::dir` — one live, one LOCKED at the heading the stroke entered at — so
        // the engine has to settle one either way. Gating this on `rake` would make the un-raked knife lock
        // onto `[0, 0]` (the pen-down dab, before any travel) and cut a flat scrape for the whole stroke:
        // the very bug this flag exists to kill, wearing the other checkbox state.
        let need = self.is_sculpt_mode() && self.is_sculpt_chisel();
        self.paint.brush.needs_heading = need;
        let slot = super::PaintMode::Sculpt.slot();
        self.paint.brush_by_mode[slot].needs_heading = need;
    }

    /// Route a Sculpt-panel event to its setter. Segmented mode = `Click` on an option id (its array
    /// position is the mode); sliders = `SetValue`. Returns `true` iff consumed. Mirrors
    /// `route_deform_event`; hung off the `handle_panel_event` chain.
    /// Inflate's Smoothness track (`0..1`) — the card paints it, the chip shows the mapped texel radius.
    #[must_use]
    pub fn sculpt_smooth(&self) -> f32 {
        self.paint.sculpt.smooth_norm
    }

    /// Inflate's Smoothness in texels (`0..16`), as the chip reads it.
    #[must_use]
    pub fn sculpt_smooth_px(&self) -> u32 {
        self.paint.sculpt.inflate_smooth_px()
    }

    /// Whether the active verb is **Inflate** — the one Memo verb that shows Depth *and* Smoothness.
    #[must_use]
    pub fn is_sculpt_inflate(&self) -> bool {
        matches!(self.paint.sculpt.mode_enum(), SculptMode::Inflate)
    }

    pub(crate) fn route_sculpt_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::Click(id) if core_ids::PAINTER_SCULPT_MODE_IDS.contains(id) => {
                let idx = core_ids::PAINTER_SCULPT_MODE_IDS
                    .iter()
                    .position(|x| x == id)
                    .unwrap_or(0) as u8;
                self.set_sculpt_mode(idx);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SCULPT_RADIUS_SLIDER => {
                self.set_sculpt_radius(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SCULPT_OFFSET_SLIDER => {
                self.set_sculpt_offset(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SCULPT_DEPTH_SLIDER => {
                self.set_sculpt_depth(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SCULPT_ANGLE_SLIDER => {
                self.set_sculpt_angle(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SCULPT_SMOOTH_SLIDER => {
                self.set_sculpt_smooth(*v as f32);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SCULPT_RAKE => {
                self.toggle_sculpt_rake();
                true
            }
            _ => false,
        }
    }
}
