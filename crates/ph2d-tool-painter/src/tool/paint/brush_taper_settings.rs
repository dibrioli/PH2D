//! The **Taper** setters (Procreate *Touch Taper*; Enio 2026-08-08).
//!
//! ⚠️ **Every one of these fans out to all four media.** Each paint mode carries its own
//! [`ph2d_painter_brush::BrushSpec`] slot, so a value written only to the live one is a value the artist
//! loses the moment they pick Watercolor — the exact shape of the bug the Paint Mode dropdown documented
//! (*"ao entrar em Impasto e depois sair e selecionar Wet Paint, widgets sumiram"*). A taper is one
//! artistic decision about the SHAPE OF A MARK, and Digital / Watercolor / Impasto / Wet Paint are four
//! spellings of the same mark — so it follows the precedent of the paint **colour**
//! (`sync_brush_color_across_modes`) and of the impasto **material** (`set_material_field`), not the
//! precedent of the per-slot Size.
//!
//! One door per field, and the fan-out lives inside it: a caller that had to remember to spread the
//! value is a caller that eventually does not.

use super::PainterTool;
use ph2d_painter_brush::taper::MAX_TAPER_DIAMETERS;

impl PainterTool {
    /// Write one field of the taper into the live brush **and every mode slot**.
    ///
    /// The closure takes the slot's taper so the fan-out is expressed once. `set_material_field` is the
    /// same shape for the same reason.
    fn set_taper_field(&mut self, f: impl Fn(&mut ph2d_painter_brush::taper::Taper)) {
        f(&mut self.paint.brush.taper);
        for slot in &mut self.paint.brush_by_mode {
            f(&mut slot.taper);
        }
    }

    /// **Taper length at the stroke's start**, in brush diameters (`0` = off).
    pub fn set_brush_taper_start(&mut self, v: f32) {
        let v = v.clamp(0.0, MAX_TAPER_DIAMETERS);
        self.set_taper_field(|t| t.start = v);
    }

    /// **Taper length at the stroke's end**, in brush diameters (`0` = off).
    ///
    /// ⚠️ Non-zero is what arms the tail hold for the incremental methods, so this is the number that
    /// decides how far the wet end of a freehand stroke trails the cursor — see
    /// [`ph2d_painter_brush::stroke`].
    pub fn set_brush_taper_end(&mut self, v: f32) {
        let v = v.clamp(0.0, MAX_TAPER_DIAMETERS);
        self.set_taper_field(|t| t.end = v);
    }

    /// **Tip size** at the start, `0..1` (`0` = a sharp point, `1` = blunt). While the tips are linked
    /// this is the tip of BOTH ends — the panel only offers one row, and the engine reads it for both
    /// through `Taper::effective_tip_end`.
    pub fn set_brush_taper_tip_start(&mut self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.set_taper_field(|t| t.tip_start = v);
    }

    /// **Tip size** at the end, `0..1`. Ignored by the engine while the tips are linked.
    pub fn set_brush_taper_tip_end(&mut self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.set_taper_field(|t| t.tip_end = v);
    }

    /// Taper **Opacity**, `0..1`: how much the taper fades as well as narrows.
    pub fn set_brush_taper_opacity(&mut self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.set_taper_field(|t| t.opacity = v);
    }

    /// Toggle **Link tip sizes**.
    ///
    /// ⚠️ Unlinking SEEDS the end tip from the start's, so the two ends do not jump apart at the instant
    /// the artist unticks the box. Unlinking is a request to start editing them separately, not a request
    /// to change the mark — and a control that silently reshapes the stroke on the way to letting you
    /// reshape it is a control the artist learns to distrust.
    pub fn toggle_brush_taper_link(&mut self) {
        let on = !self.paint.brush.taper.link_tips;
        let seed = self.paint.brush.taper.tip_start;
        self.set_taper_field(|t| {
            if !on {
                t.tip_end = seed;
            }
            t.link_tips = on;
        });
    }
}
