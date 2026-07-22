//! [`PaintMedia`] — **what the paint is made of**, and the one door that switches it.
//!
//! The Painter has four media and they are mutually exclusive: the plain **Digital** brush (the
//! Blender-style deposit), the optical **Watercolor** wash, the **Impasto** body, and the **Wet Paint**
//! fluid simulation. Until 2026-07-22 they were three independent *checkboxes* — which made "which
//! medium am I in?" a question with eight answers, only four of which mean anything, and left the
//! artist to keep them exclusive by hand (Enio: *"no lugar dos checkbox coloque um dropdown para o modo
//! de pintura com os 4 modos. O padrão é o Digital normal"*).
//!
//! ⚠️ **The medium is DERIVED, never stored.** The three flags stay where they were (`BrushSpec`'s
//! `watercolor` / `impasto`, `WetPaintState::armed`) and this enum is a *view* of them. A fourth field
//! holding "the current medium" would be a second place for the same fact to be true, and the two would
//! drift the first time anything else touched a flag — the failure this file exists to end, not repeat.
//!
//! ⚠️ **A medium OWNS paint modes, and that ownership is the bug fix.** `Knife` and `Sculpt` exist only
//! because Impasto is on; `WetPaint` exists only because the fluid is armed. Leaving a medium therefore
//! has to bring the artist back out of a mode that belongs to it — see [`PaintMedia::cannot_outlive`] and
//! `PainterTool::set_brush_impasto`. Measured before the fix: enter Impasto, pick the Knife, untick
//! Impasto, tick Wet Paint — and you are *still holding the knife*, with the Colour and Blend rows gone
//! (`paints_no_color()` is true for the knife) while the panel says Wet Paint.

use super::PaintMode;

/// The paint's medium — the four mutually exclusive answers to *what is this brush made of?*
///
/// The wire `u8` is the dropdown's option value and is part of the panel seam; append only.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PaintMedia {
    /// The plain brush — no wash, no body, no water. The default, and the one every other medium
    /// returns to when it is switched off.
    #[default]
    Digital,
    /// The optical watercolor wash (Beer–Lambert deposit + edge darkening + granulation).
    Watercolor,
    /// The paint has THICKNESS — the relief planes, the ten tools that shape it, and its light.
    Impasto,
    /// The shallow-water fluid simulation (ADR-0134).
    WetPaint,
}

impl PaintMedia {
    /// Number of media — the dropdown's option count.
    pub const COUNT: u8 = 4;

    /// The wire value the panel's dropdown carries.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            PaintMedia::Digital => 0,
            PaintMedia::Watercolor => 1,
            PaintMedia::Impasto => 2,
            PaintMedia::WetPaint => 3,
        }
    }

    /// Decode a wire value; anything unknown falls back to [`PaintMedia::Digital`] — the medium that
    /// is always safe, because it is the absence of the other three.
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => PaintMedia::Watercolor,
            2 => PaintMedia::Impasto,
            3 => PaintMedia::WetPaint,
            _ => PaintMedia::Digital,
        }
    }

    /// The dropdown's display name. English UI (HR-15), and these are the words the artist already
    /// reads on the section headers below the chip.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            PaintMedia::Digital => "Digital",
            PaintMedia::Watercolor => "Watercolor",
            PaintMedia::Impasto => "Impasto",
            PaintMedia::WetPaint => "Wet Paint",
        }
    }

    /// Can this medium work in `mode` — is the tool in the artist's hand one this medium paints with?
    ///
    /// `Digital` answers `false` everywhere on purpose: it is the *absence* of a medium, so it never
    /// has an opinion about which tool you hold (see [`Self::cannot_outlive`], which reads this).
    ///
    /// * **Impasto** — `Paint` (the Deposit), `Knife`, `Sculpt`: the ten tools of its TOOL list.
    /// * **Watercolor** — `Paint`: the wash reinterprets the ordinary deposit; it has no mode of its own.
    /// * **Wet Paint** — `WetPaint`: the fluid IS a mode.
    #[must_use]
    pub(crate) fn works_in(self, mode: PaintMode) -> bool {
        match self {
            PaintMedia::Digital => false,
            PaintMedia::Watercolor => mode == PaintMode::Paint,
            PaintMedia::Impasto => {
                matches!(
                    mode,
                    PaintMode::Paint | PaintMode::Knife | PaintMode::Sculpt
                )
            }
            PaintMedia::WetPaint => mode == PaintMode::WetPaint,
        }
    }

    /// Would `mode` be **orphaned** by switching this medium off — does it exist only because the
    /// medium is on?
    ///
    /// Derived from [`Self::works_in`] rather than listed a second time, because two hand-written
    /// lists of "which modes are the Impasto's" are two chances to forget the eleventh tool. The
    /// subtraction is `Paint`: the plain brush uses it too, so the wash and the body are only ever
    /// *reinterpreting* it — leaving them there is honest, and yanking the artist to a mode they are
    /// already in would be a jolt for nothing.
    ///
    /// This is the whole of the 2026-07-22 defect: `Knife` and `Sculpt` survived the Impasto being
    /// switched off, `paints_no_color()` stayed true for them, and the Colour + Blend rows were gone
    /// from a panel that said Wet Paint at the top.
    #[must_use]
    pub(crate) fn cannot_outlive(self, mode: PaintMode) -> bool {
        self.works_in(mode) && mode != PaintMode::Paint
    }
}

impl crate::tool::PainterTool {
    /// The medium the brush is in **right now** — read off the three flags, never stored.
    ///
    /// The precedence only matters for a state the door below cannot produce (two flags at once); it
    /// mirrors the behavioural precedence, so the answer always names the medium that is actually
    /// painting: the wet arm reroutes the `"brush"` wire outright, and the wash's render path
    /// short-circuits the impasto (`impasto_section_applies` already asks `!watercolor_render_active`).
    #[must_use]
    pub fn paint_media(&self) -> PaintMedia {
        if self.paint.wetpaint.armed {
            PaintMedia::WetPaint
        } else if self.paint.brush.watercolor {
            PaintMedia::Watercolor
        } else if self.paint.brush.impasto {
            PaintMedia::Impasto
        } else {
            PaintMedia::Digital
        }
    }

    /// **The one door that switches medium** — the Paint Mode dropdown's setter, and the only place
    /// that knows the four are exclusive.
    ///
    /// Three steps, and each is load-bearing — **in this order**:
    ///
    /// 1. **Leave** every medium that is not the pick. Each `set_*` is responsible for bringing the
    ///    artist out of a mode that [`PaintMedia::cannot_outlive`], so no step here can strand one.
    /// 2. **Choosing a medium USES it** — the TOOL list's own law (Enio, 2026-07-19: *"escolher uma
    ///    ferramenta USA ela"*). If the mode in hand is not one the new medium works in, take the
    ///    brush. Without this, picking Impasto while holding the rail Smear selects a medium whose
    ///    section is not even painted (`impasto_section_applies`), and picking Wet Paint from there
    ///    arms a fluid that the `"brush"` wire never reaches — a dropdown naming a medium that is not
    ///    running. `Digital` is exempt because it owns no mode: the rail tools ARE digital, so
    ///    picking it must never yank you out of the Smear.
    /// 3. **Enter** the pick.
    ///
    /// ⚠️ **Steps 2 and 3 were the other way round, and that made Watercolor a DEAD control from any
    /// rail tool** (measured 2026-07-22): each paint mode keeps its own [`ph2d_painter_brush::BrushSpec`],
    /// so entering first wrote `watercolor = true` into the **Smear's** slot — and step 2 then switched
    /// to the brush, whose slot still read `false`. The chip snapped straight back to *Digital*. Impasto
    /// survived by luck (`set_brush_impasto` mirrors its flag into all three relief slots) and Wet Paint
    /// by design (`armed` lives in `WetPaintState`, not in a slot). Take the tool FIRST, then arm the
    /// medium on the slot the artist actually lands in.
    pub fn set_paint_media(&mut self, media: PaintMedia) {
        if media != PaintMedia::WetPaint {
            self.set_wetpaint_armed(false);
        }
        if media != PaintMedia::Watercolor {
            self.set_brush_watercolor(false);
        }
        if media != PaintMedia::Impasto {
            self.set_brush_impasto(false);
        }
        if media != PaintMedia::Digital && !media.works_in(self.paint.paint_mode) {
            self.set_paint_tool_mode("brush");
        }
        match media {
            PaintMedia::Digital => {}
            PaintMedia::Watercolor => self.set_brush_watercolor(true),
            PaintMedia::Impasto => self.set_brush_impasto(true),
            PaintMedia::WetPaint => self.set_wetpaint_armed(true),
        }
    }
}

#[cfg(test)]
mod tests;
