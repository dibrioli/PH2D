//! The sculpt **mode taxonomy** (child of [`super`], split for the workspace
//! file-LOC cap): which verb, which ENGINE family it runs on, and which memo
//! key the session carries. The state, the knobs and the tool-side routing
//! stay in the parent — this file is the one place a verb's identity lives.

/// Which verb the sculpt brush performs. Discriminants are the segmented control's indices.
///
/// **All eight are one kernel** — `h = pre + k·Δ`, where `Δ` is the distance to a per-texel *target* and the
/// verb decides two things: where the target comes from, and which SIGN of `Δ` is allowed through.
///
/// | verb | target | Δ |
/// |---|---|---|
/// | Smooth | `blur(pre)` | both ways |
/// | Sharpen | `pre + (pre − blur(pre))` | both ways |
/// | Flatten | the fitted **plane** | both ways |
/// | Scrape | the fitted plane | **down only** (`min(Δ, 0)`) |
/// | Fill | the fitted plane | **up only** (`max(Δ, 0)`) |
/// | Chisel | the plane **+ a V about the stroke's axis** | down only |
/// | Layer | `pre + Depth` | both ways |
/// | Inflate | the **Blob**: a ball whose radius follows the falloff (`render_inflate`) | both ways |
///
/// So Scrape and Fill are not two more engines — they are Flatten with one half of the number thrown away,
/// and they cost one `min` each. Chisel is Scrape with one `abs`. Layer is the kernel with a *constant*
/// target, which is what makes it bounded: `k ≤ 1`, so `h` never passes `pre + Depth` however long you dwell.
///
/// ## Three of Blender's brushes are missing, and each absence is a finding
///
/// * **Clay** is `Flatten` with a positive Offset. The plane sits above the surface, so the hollows rise to
///   it and the ridges fall to it: material added, surface flattened — that IS clay. Both knobs are already
///   on screen. A chip for it would be a preset of another chip, and a card cannot tell you which of two
///   identical tools you are holding.
/// * **Clay Strips** is Clay with a square dab. The dab's shape belongs to the **brush** (ten falloffs, a
///   Shape image slot, flatten and angle) — a square falloff is a gap in the brush, not a verb in the sculpt.
/// * **Draw Sharp** collapses into **Layer**. Blender needs it as a separate brush because its ordinary Draw
///   reads the *deformed* mesh and so rounds its own crest off; our per-stroke engine reads the FROZEN `pre`
///   and cannot do otherwise (§4). Every additive verb here is "sharp" by construction — there is nothing
///   left for a second one to be.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(in crate::tool::paint) enum SculptMode {
    /// Pull the relief toward its own local average — `lerp(pre, blur(pre), k)`.
    Smooth,
    /// Push it away from that average — `pre + k·(pre − blur(pre))`, an unsharp mask on a height field.
    /// Bounded by construction (`k ≤ 1` ⇒ at most twice the local detail), so it cannot ring away.
    Sharpen,
    /// Pull the relief toward the **tilted plane** fitted to the footprint (§7). Tilted, not horizontal:
    /// a horizontal fit cuts a crater into a hillside instead of flattening it.
    Flatten,
    /// The same plane, **down only** — the spatula taking the high ground off and leaving the valleys.
    Scrape,
    /// The same plane, **up only** — paint pushed into the valleys, the ridges left standing.
    Fill,
    /// The knife **tipped onto its edge**: the same plane plus a V rising out of the stroke's own axis, and
    /// scraped down to. What it leaves is a groove with a crease at the bottom — the sharpest mark a palette
    /// knife makes. At Angle 0 it IS Scrape, to the byte ([`ph2d_painter_brush::sculpt::Chisel`]).
    Chisel,
    /// **Bounded build-up**: `pre + Depth`, so the relief rises toward exactly that and stops. Dwell all day
    /// and the coat is still one Depth thick — which is the whole point, and the one thing neither the
    /// deposit (which accumulates) nor the plane verbs (which level) can do.
    Layer,
    /// **Puff**: the relief **offset along its own normal** by Depth — which, done exactly, is the
    /// morphological dilation (or, for a negative Depth, erosion) of the height field by a **ball** of that
    /// radius. The form gets *fatter*, not merely taller: its rim is pushed outward, creases fill in, and a
    /// negative Depth eats thin ridges away.
    ///
    /// On a **flat** it raises by exactly Depth — the same as [`Layer`](SculptMode::Layer) — and that is
    /// geometry, not a bug: offsetting a plane along its normal is a translation. The two verbs differ in
    /// what they do to the *shape*. See [`super::super::sculpt_offset`], which also records the two bugs this line
    /// shipped here (the normal was applied *upside down*, and a per-texel formula cannot inflate anything).
    Inflate,
}

/// Which *engine* a verb runs on — and therefore which per-texel target the session must carry.
///
/// It is not cosmetic bookkeeping: the three targets are derived from different things, and they are not
/// equally recoverable.
///
/// * [`Memo`](SculptFamily::Memo) → a **pure function of the frozen relief and one radius**, computed by a
///   kernel over a neighbourhood: `blur(pre, r)` for Smooth / Sharpen, `offset(pre, r)` for Inflate. Throw
///   it away and it rebuilds itself, tile by tile, from `pre` — which is exactly what makes the tile memo
///   in [`super::super::sculpt_blur`] legitimate.
/// * [`Plane`](SculptFamily::Plane) → `Σ w·plane(i)`: a function of the **dab list**, which no longer
///   exists once the batch has been consumed. It cannot be rebuilt without re-stamping the stroke.
/// * [`Height`](SculptFamily::Height) → a function of `pre` and a knob, evaluated per texel in the render.
///   **It has no buffer at all**, so a Layer stroke costs 8 B/px where the others cost 12.
///
/// That first asymmetry is why `PainterTool::set_sculpt_mode` re-stamps an open shape when the family
/// changes, and why it does not need to when it does not.
///
/// ## The engine family is NOT the knob family, and it used to be by coincidence
///
/// Inflate takes the **Depth** knob (like Layer) but runs on the **memo** engine (like Smooth) — because
/// its target is `offset(pre, Depth)`, a neighbourhood kernel, not a per-texel formula. While Inflate was
/// (wrongly) a per-texel formula the two groupings happened to coincide, and one enum served both. It does
/// not any more, so [`SculptMode::knob_family`] answers the panel's question and this one answers the
/// session's. Two questions that merely agreed for a while are two questions.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(in crate::tool::paint) enum SculptFamily {
    Memo,
    Plane,
    Height,
}

/// What the session's memo plane holds — and, because it is the invalidation key, what a change to it
/// throws away.
///
/// The two kernels are memoised by the SAME tile machinery (`super::super::sculpt_blur`) because they have the
/// same shape: each texel's target is a pure function of `pre` inside a bounded neighbourhood. That bound
/// is [`Self::reach`], and it is the width the read window is grown by — the single number the memo's
/// byte-identity argument rests on.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(in crate::tool::paint) enum MemoKey {
    /// No memo — the Plane and Height families own no kernel target.
    #[default]
    None,
    /// `blur(pre, r)` — Smooth / Sharpen. `r` in texels.
    Blur(u32),
}

impl MemoKey {
    /// How far, in texels, one output texel reads into `pre` — the tile memo's read-window growth.
    pub(in crate::tool::paint) fn reach(self) -> u32 {
        match self {
            MemoKey::None => 0,
            MemoKey::Blur(r) => r,
        }
    }
}

impl SculptMode {
    pub(in crate::tool::paint) fn from_u8(v: u8) -> Self {
        match v {
            1 => SculptMode::Sharpen,
            2 => SculptMode::Flatten,
            3 => SculptMode::Scrape,
            4 => SculptMode::Fill,
            5 => SculptMode::Chisel,
            6 => SculptMode::Layer,
            7 => SculptMode::Inflate,
            _ => SculptMode::Smooth,
        }
    }

    /// Which **engine** the verb runs on — i.e. which per-texel target the session must carry.
    pub(in crate::tool::paint) fn family(self) -> SculptFamily {
        match self {
            SculptMode::Smooth | SculptMode::Sharpen => SculptFamily::Memo,
            SculptMode::Flatten | SculptMode::Scrape | SculptMode::Fill | SculptMode::Chisel => {
                SculptFamily::Plane
            }
            // Inflate has NO persistent target buffer: its ball radius follows the falloff (`amount`), which
            // grows across the stroke, so the offset is not a stroke-constant and cannot be memoised (that is
            // the whole reason the constant-ball memo was retired — Enio 2026-07-14, the *"parece mistura de
            // inflate com layer"* smoke). It recomputes per render in `render_inflate`, like Layer, and holds
            // no buffer either.
            SculptMode::Layer | SculptMode::Inflate => SculptFamily::Height,
        }
    }

    /// Which **knob** the card must paint: `0` Radius · `1` Offset · `2` Depth. (The Chisel additionally
    /// shows Angle — see `PainterTool::is_sculpt_chisel`.) Not the same question as [`Self::family`].
    pub(in crate::tool::paint) fn knob_family(self) -> u8 {
        match self {
            SculptMode::Smooth | SculptMode::Sharpen => 0,
            SculptMode::Flatten | SculptMode::Scrape | SculptMode::Fill | SculptMode::Chisel => 1,
            SculptMode::Layer | SculptMode::Inflate => 2,
        }
    }
}
