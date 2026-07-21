//! [`PaintMode`] — which operation the canvas pointer performs. Split from `paint.rs` for the workspace
//! file-LOC cap; a plain data enum with no `PaintState` coupling.

/// Which operation the canvas pointer performs — selected from the left rail's Painter tools and routed
/// in via `PanelEvent::SelectOption(PAINTER_PAINT_MODE, …)`. `Paint` = the normal dab-stamp (colour,
/// Shape, Grain, ramps); `Smear` drags canvas content along the stroke; `Blur` softens under each dab;
/// `Clone` copies from a sampled source at a fixed offset; `Mask` paints a TEMPORARY tool-side scratch
/// mask that hides/reveals the current layer live (no layer created — see [`super::mask`]); `Inpaint` is
/// a content-aware HEAL brush — paint over a defect to mark it (live red tint), and on pen-up the marked
/// region is reconstructed from the surrounding pixels by the `ph2d-inpaint` engine (multi-scale
/// PatchMatch), cropped to the defect neighbourhood so it stays interactive (see [`super::inpaint`]).
/// Eraser is a blend override on top of `Paint`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum PaintMode {
    #[default]
    Paint,
    Smear,
    Blur,
    Clone,
    Mask,
    Inpaint,
    /// **Fill** (Bucket) — Procreate ColorDrop: drag the colour onto the canvas to flood-fill the
    /// connected same-colour region, then drag to adjust the threshold live (see [`super::fill`]).
    Fill,
    /// **Selection** — Procreate-style selection (Automatic / Freehand / Rectangle / Ellipse) that builds
    /// a canvas-wide selection mask gating every other operation. The engine (marching-ants mask +
    /// Add/Remove/Invert/Feather, integrated into the painter undo queue) lands in a follow-up; for now
    /// the mode exists but paints nothing on canvas.
    Selection,
    /// **Deform** — brush-driven reshape / Liquify (Push / Twist / Pinch / Wrinkle / Fold / Reconstruct)
    /// over a single inverse-warp kernel (`out[dst] = sample(dst − D(dst))`, backward-gather). Only the
    /// displacement field `D` changes between sub-modes. Writes the active raster (structural undo), reuses
    /// the Selection mask for Freeze/Protect. See [`super::warp`] (Deform Wave 1).
    Deform,
    /// **Sculpt** — reshape the impasto RELIEF (Smooth / Sharpen; the spatula verbs land in Wave 2). It
    /// reads the layer's committed height and rewrites it, and it writes NOTHING else: no RGBA, no
    /// coverage, no material (`docs/Painter/18…` §5 — scraping the BODY of the paint does not take its
    /// PIGMENT, and a sculpt that erased colour would just be the eraser).
    ///
    /// Where Deform is a mode-exclusive tool with its OWN dab geometry, Sculpt deliberately has none: it
    /// consumes the SAME dab list the colour does, at the same choke point (`stamp_dabs_height`'s
    /// neighbour in `stamp_dabs_inner`), so Symmetry, Tiling, the shape editors, pressure, Jitter,
    /// falloff, Shape and Grain reach it for free — and keep reaching it when someone changes them. A
    /// spatula with Grain is a textured spatula, and it costs nothing. See [`super::sculpt`].
    Sculpt,
    /// **Knife** — the palette knife: a Smear that carries the impasto BODY, not just the colour.
    ///
    /// ⚠️ A mode of its own, and that is the whole point (Enio, 2026-07-19: *"o modo Smear do Impasto
    /// (knife) deve ser único e não compartilhado com o smear dos outros tipos de pintura já que ele
    /// afeta o Volume do impasto. Smear com botão no painel lateral é o smear dos outros modos de
    /// pintura."*). It ran as [`PaintMode::Smear`] until then, which meant the two shared one
    /// `BrushSpec` slot — so the Plow that makes a knife a knife was also on the ordinary smear, and
    /// dialling either one moved the other.
    ///
    /// Same engine path as the Smear (ask [`PaintMode::smears`], never a bare `== Smear`); different
    /// slot, so Plow / Size / Spacing are its own. It has **no left-rail button**: the rail's Smear is
    /// the plain one, and the Knife is picked from the Impasto TOOL list, which is where the tools that
    /// act on the paint's body live.
    Knife,
    /// **Wet Paint** — the fourth paint MEDIUM (ADR-0134): a real shallow-water / capillary fluid
    /// simulation over paper tooth (Rebelle-style), backed by the `ph2d-wet-paint` engine crate.
    /// Suspended pigment rides the flow, settles as it dries (darkened edges), re-wets, bleeds
    /// wet-on-wet and drips under tilt. The dab route hangs off `stamp_dabs_inner` like Sculpt
    /// does (Symmetry / Tiling / shape editors / pressure / Jitter for free); the live water keeps
    /// moving after pen-up through the `on_tick` heartbeat at a fixed 40 Hz accumulator.
    WetPaint,
}

/// Number of [`PaintMode`] variants — the length of the per-mode brush-settings array (see
/// [`PaintMode::slot`]). Keep in lock-step with the enum.
pub(crate) const PAINT_MODE_COUNT: usize = 12;

impl PaintMode {
    /// Whether this mode drags canvas content along the stroke — the **smear field**.
    ///
    /// The ordinary [`PaintMode::Smear`] and the [`PaintMode::Knife`] are the same operation with
    /// different settings: the knife's Plow carries the impasto body along with the pigment. Every site
    /// that dispatches the smear asks HERE rather than testing `== Smear`, because an enumeration of
    /// those sites is exactly what rots when a second member joins the family — the engine would run the
    /// warp for one and not the other, and the knife would simply do nothing.
    #[must_use]
    pub(crate) fn smears(self) -> bool {
        matches!(self, PaintMode::Smear | PaintMode::Knife)
    }

    /// This mode's index into the per-mode brush-settings array (`0..PAINT_MODE_COUNT`). Each tool keeps
    /// its own [`ph2d_painter_brush::BrushSpec`] here when settings are NOT linked across tools.
    pub(crate) fn slot(self) -> usize {
        match self {
            PaintMode::Paint => 0,
            PaintMode::Smear => 1,
            PaintMode::Blur => 2,
            PaintMode::Clone => 3,
            PaintMode::Mask => 4,
            PaintMode::Inpaint => 5,
            PaintMode::Fill => 6,
            PaintMode::Selection => 7,
            PaintMode::Deform => 8,
            PaintMode::Sculpt => 9,
            PaintMode::Knife => 10,
            PaintMode::WetPaint => 11,
        }
    }
}
