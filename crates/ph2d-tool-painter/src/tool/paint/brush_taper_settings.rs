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
use ph2d_painter_brush::taper::{MAX_FALL_OFF_DIAMETERS, MAX_TAPER_DIAMETERS};

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

    /// ⭐ **L5 — a CAUDA**, em diâmetros (`0` = desligada): re-pintada ao levantar, no plano do traço do Digital.
    pub fn set_brush_taper_end(&mut self, v: f32) {
        let v = v.clamp(0.0, MAX_TAPER_DIAMETERS);
        self.set_taper_field(|t| t.end = v);
    }

    /// A largura na ponta do FIM, `0..1`.
    pub fn set_brush_taper_tip_end(&mut self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.set_taper_field(|t| t.tip_end = v);
    }

    /// ⭐ **L5 — o FALL-OFF**: a carga que se esgota ao longo deste comprimento, em diâmetros (`0` = desligado).
    pub fn set_brush_fall_off(&mut self, v: f32) {
        let v = v.clamp(0.0, MAX_FALL_OFF_DIAMETERS);
        self.set_taper_field(|t| t.fall_off = v);
    }

    /// O patamar antes de a carga começar a acabar, em diâmetros.
    pub fn set_brush_fall_off_start(&mut self, v: f32) {
        let v = v.clamp(0.0, MAX_FALL_OFF_DIAMETERS);
        self.set_taper_field(|t| t.fall_off_start = v);
    }
}

// ⭐ A cauda VOLTOU (L5, 20/10) — só no Digital, re-pintada no plano do traço ao levantar (`traco_plano`): sem
// restaurar a tela nem re-carimbar o traço, numa transacção. O porquê de ter saído (Enio 2026-08-10) e de voltar
// assim mora em `ph2d_painter_brush::taper`. O *Link tip sizes* não voltou: duas linhas, dois números.
