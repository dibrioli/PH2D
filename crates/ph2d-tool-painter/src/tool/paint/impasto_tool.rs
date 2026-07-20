//! **The Impasto TOOL — the ten things that act on the paint's body, as one list.**
//!
//! Enio, 2026-07-19: *"os tools de Impasto estão espalhados em 3 lugares: no painel brush, no smear e
//! Sculpt. Vamos unificar e organizar tudo num único lugar no painter."*
//!
//! They were three, and the split was not cosmetic — it was load-bearing on the mode:
//!
//! | where | reachable in | what you got |
//! |---|---|---|
//! | the Impasto section | `Paint` only ([`super::PainterTool::impasto_applies`]) | Body · Material · **Lighting** |
//! | the Knife | `Smear` only ([`super::PainterTool::impasto_plow_applies`]) | Plow, and nothing else |
//! | the Sculpt card | `Sculpt` only | the eight verbs, in a card at the top of the panel |
//!
//! Read the right-hand column down: **the Lighting card is reachable in `Paint` and nowhere else.** So
//! the artist switched into Sculpt — the mode whose entire purpose is to shape relief — and lost the
//! controls that make relief *visible*. Same in the Smear. That is not "three places for one subject",
//! it is two of the three places having no way to see what they are doing.
//!
//! ## The model
//!
//! There are ten operations on the body of the paint, and this enum is the list:
//!
//! - **Deposit** — the brush lays body down (`PaintMode::Paint`)
//! - **Knife** — the smear drags the body that is already there (`PaintMode::Smear`)
//! - the eight **sculpt verbs**, which reshape it (`PaintMode::Sculpt` + a `SculptMode`)
//!
//! Picking one *uses* it, so the selector drives the mode. It does that through the doors that already
//! exist — [`super::PainterTool::set_paint_tool_mode`] and [`super::PainterTool::set_sculpt_mode`] — and
//! never by writing `paint_mode` itself: those doors commit a live fill, swap the per-mode brush slot and
//! end the warp/deform sessions, and a second way in would be a second answer to *"what happens when I
//! change tool?"*.
//!
//! ⚠️ **This list and the left rail's chips are two VIEWS of one radio, not two radios.** The rail's
//! pressed state must therefore be derived from the published mode rather than written by whoever was
//! clicked last — see `rail_painter_tools::sync_from_mode`. Without that, picking Chisel here would leave
//! the rail highlighting "Brush" while you sculpt.

use super::{PaintMode, PainterTool};

/// How many tools the Impasto list carries: Deposit · Knife · the eight sculpt verbs.
pub const IMPASTO_TOOL_COUNT: u8 = 10;

/// The wire index of the **Deposit** tool (the brush laying body down).
pub const IMPASTO_TOOL_DEPOSIT: u8 = 0;
/// The wire index of the **Knife** (the smear dragging existing body).
pub const IMPASTO_TOOL_KNIFE: u8 = 1;
/// The wire index of the first **sculpt verb**; the eight run `2..IMPASTO_TOOL_COUNT`, in `SculptMode`
/// order (Smooth · Sharpen · Flatten · Scrape · Fill · Chisel · Layer · Inflate).
pub const IMPASTO_TOOL_SCULPT_BASE: u8 = 2;

impl PainterTool {
    /// Whether the **Impasto section** applies at all — the three modes that have something to say about
    /// the body of the paint.
    ///
    /// Deliberately wider than [`Self::impasto_applies`], which answers a different question: *does this
    /// brush DEPOSIT body?* (`Paint`, and only `Paint`). This one answers *is the artist working on the
    /// paint's body?*, and the Knife and the sculpt verbs are as much that work as the deposit is. The
    /// two live side by side because collapsing them would either hide the section from Sculpt (the bug
    /// this replaces) or offer a Depth slider to a knife that lays nothing down.
    ///
    /// Suppressed under the **watercolor** wash (a separate implementation — thin paint, no body) and the
    /// **eraser** (a blend override that removes paint rather than shaping it), exactly as its two
    /// siblings are. Modes absent from the list — Blur, Clone, Mask, Inpaint, Fill, Selection, Deform —
    /// have no verb on this list; ⚠️ Deform's **Affect Relief** toggle is relief-adjacent and stays in the
    /// Deform card, because Deform owns the whole panel body through an early return.
    #[must_use]
    pub fn impasto_section_applies(&self) -> bool {
        matches!(
            self.paint.paint_mode,
            PaintMode::Paint | PaintMode::Knife | PaintMode::Sculpt
        ) && !self.watercolor_render_active()
            && !self.paint.eraser
    }

    /// Which of the ten tools is in the artist's hand — a pure function of the modes, never a field.
    ///
    /// Storing it would make a second place for "which tool?" to be true, and the rail can already change
    /// the mode without going anywhere near this list.
    #[must_use]
    pub fn impasto_tool(&self) -> u8 {
        match self.paint.paint_mode {
            PaintMode::Knife => IMPASTO_TOOL_KNIFE,
            PaintMode::Sculpt => {
                IMPASTO_TOOL_SCULPT_BASE + self.paint.sculpt.mode.min(SCULPT_VERB_MAX)
            }
            _ => IMPASTO_TOOL_DEPOSIT,
        }
    }

    /// Pick one of the ten. Routed from the panel over the frozen `SelectOption` channel.
    ///
    /// Out-of-range indices fall back to **Deposit** rather than panicking: the value crosses a `String`
    /// wire, and the honest failure for a garbled one is the tool the artist started with.
    pub fn set_impasto_tool(&mut self, tool: u8) {
        match tool {
            IMPASTO_TOOL_KNIFE => self.enter_mode(PaintMode::Knife, "knife"),
            t if (IMPASTO_TOOL_SCULPT_BASE..IMPASTO_TOOL_COUNT).contains(&t) => {
                // Order matters: enter the mode first, so `set_sculpt_mode`'s re-stamp of any open shape
                // happens with the sculpt brush slot already loaded.
                self.enter_mode(PaintMode::Sculpt, "sculpt");
                self.set_sculpt_mode(t - IMPASTO_TOOL_SCULPT_BASE);
            }
            _ => self.enter_mode(PaintMode::Paint, "brush"),
        }
    }

    /// [`Self::set_paint_tool_mode`], skipped when the mode is already the one asked for.
    ///
    /// ⚠️ **Hygiene, not correctness — and the first draft of this comment claimed the opposite.** I wrote
    /// that re-entering the current mode would end the sculpt session a verb switch is about to re-stamp;
    /// then the mutation that removes this guard survived the whole workspace, so I went and read
    /// `set_paint_tool_mode`. Every session-ending branch in it is already gated on `old != new`
    /// (`stencil.rs`: leaving Smear, leaving Deform, leaving Sculpt), so a same-mode call ends nothing.
    /// What it does do is re-commit a fill that is not running and round-trip the brush slot through
    /// itself — dead work on every verb click, which is worth skipping but is not a bug.
    ///
    /// It is kept because it makes this call site robust against the shape of guard that function already
    /// carries three times: the next one written without an `old != new` test would fire on a verb switch,
    /// and *that* would be the bug this paragraph originally described.
    fn enter_mode(&mut self, mode: PaintMode, wire: &str) {
        if self.paint.paint_mode != mode {
            self.set_paint_tool_mode(wire);
        }
    }
}

/// The highest `SculptMode` wire value (eight verbs, `0..=7`). Local rather than imported so the clamp
/// above cannot silently widen if the verb list grows without this list growing with it.
const SCULPT_VERB_MAX: u8 = IMPASTO_TOOL_COUNT - IMPASTO_TOOL_SCULPT_BASE - 1;
