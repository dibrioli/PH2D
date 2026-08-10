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

    /// **Tip size**, `0..1` (`0` = a sharp point, `1` = blunt). Procreate's "Tip".
    pub fn set_brush_taper_tip_start(&mut self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.set_taper_field(|t| t.tip_start = v);
    }

    /// Taper **Opacity**, `0..1`: how much the taper fades as well as narrows.
    pub fn set_brush_taper_opacity(&mut self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.set_taper_field(|t| t.opacity = v);
    }
}

// ⛔ `set_brush_taper_end`, `set_brush_taper_tip_end` and `toggle_brush_taper_link` lived here and went
// with the far end (Enio 2026-08-10: *"deixe o ajuste apenas para o início do traço"*). They are named
// here rather than silently absent because each was a LIVE control with a row on screen: a length, a
// second tip, and the toggle that linked the two. `ph2d_painter_brush::taper` carries why, and what the
// removal cost.
