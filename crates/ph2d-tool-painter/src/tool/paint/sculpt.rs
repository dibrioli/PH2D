//! **Sculpt** — reshaping the relief the paint already laid (`docs/Painter/18_plano_sculpt_relevo.md`).
//!
//! Wave 1: **Smooth** (pull the relief toward its own local average) and **Sharpen** (the same kernel run
//! backwards). One engine, one sign.
//!
//! ## The two things that make this module unlike Deform
//!
//! **It has no geometry of its own.** Deform is mode-exclusive with its own dab lifecycle; Sculpt hangs
//! off the ONE choke point the colour already goes through (`stamp_dabs_inner`), so Symmetry, Tiling, the
//! shape editors, pressure, Jitter, falloff, Shape and Grain reach it without a line of code each — and
//! keep reaching it when someone changes them (§10.1). A pass with geometry of its own is how
//! *"Tiling doesn't work in Sculpt"* gets born six months from now. The consequence the reader should
//! expect: **the brush's own Size / Strength / Spacing / Falloff / Shape / Grain ARE the sculpt's
//! controls**, and the Sculpt card only adds what the brush cannot already say — which verb, and over
//! what scale.
//!
//! **It is applied once per stroke, not once per dab.** The dab does not blur `h` in place; it
//! accumulates an intensity ([`ph2d_painter_brush::sculpt::accumulate_dab_sculpt`]) and the relief is
//! re-rendered from a FROZEN copy taken at the start of the stroke. See that module for why the obvious
//! version is wrong twice (it is not idempotent under the shape editors' re-stamp, and composing N dab
//! blurs turns the smoothing scale into a function of the SPACING slider). The kernel + the memo that
//! makes it affordable live in the sibling [`super::sculpt_blur`].

use super::Region;
use crate::tool::PainterTool;
use std::sync::Arc;

/// The number of sub-modes — the segmented control's option count; [`SculptMode::from_u8`] clamps to it.
/// The order NEVER shifts (the discriminant IS the segmented control's index).
pub(crate) const SCULPT_MODE_COUNT: u8 = 8;

/// The kernel's radius at Radius = 1, in pixels.
///
/// Small on purpose, and the cap is a design statement rather than a performance excuse: smoothing at a
/// LARGE scale is not a bigger blur, it is **Flatten** — a least-squares plane fit over the footprint
/// (§7), which is a different kernel and lands in Wave 2. A 100-px "smooth" would be a slow, mushy,
/// worse Flatten. Sixteen pixels is the scale brush-marks and grain actually live at. // CLAMP-OK
pub(super) const SCULPT_RADIUS_MAX_PX: f32 = 16.0;

/// How far the plane Offset can travel, in **paint-loads** — the same unit `h` itself is in (`H_CEIL = 2.0`
/// is two full loads of paint).
///
/// One load either way: at `−1` the Scrape's plane sits a full stroke's thickness UNDER the surface it was
/// fitted to, which is a spatula gouging rather than skimming; at `+1` the Fill mounds a full load above it.
/// Wider than that and the plane leaves the paint entirely — Scrape would find nothing above it to remove,
/// and the control's whole outer half would silently do nothing. // CLAMP-OK
pub(super) const PLANE_OFFSET_MAX: f32 = 1.0;

/// How far the **Depth** knob can travel, in paint-loads (Layer / Inflate). One full load either way: Layer
/// lays at most one stroke's worth of paint however long you dwell, and negative carves the same. // CLAMP-OK
pub(super) const SCULPT_DEPTH_MAX: f32 = 1.0;

/// How far the **Inflate** Smoothness (Radius) knob can blur the ball's hard edge, in texels.
///
/// Sixteen, the same ceiling Smooth's own Radius uses — past it the dilation's shape is lost to the blur,
/// which is Smooth's job, not Inflate's. At `0` the edge is the raw ball. // CLAMP-OK
pub(super) const INFLATE_SMOOTH_MAX_PX: f32 = 16.0;

/// The **Chisel**'s widest angle, in degrees — how far the knife can be tipped onto its edge.
///
/// Sixty, not ninety: `tilt = tan(angle)` runs to infinity at 90°, and past ~60° the V is already steeper
/// than any paint stands up in. The far end of a slider must do something, and beyond this it would only do
/// arithmetic. // CLAMP-OK
pub(super) const CHISEL_ANGLE_MAX_DEG: f32 = 60.0;

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
pub(super) enum SculptMode {
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
    /// what they do to the *shape*. See [`super::sculpt_offset`], which also records the two bugs this line
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
///   in [`super::sculpt_blur`] legitimate.
/// * [`Plane`](SculptFamily::Plane) → `Σ w·plane(i)`: a function of the **dab list**, which no longer
///   exists once the batch has been consumed. It cannot be rebuilt without re-stamping the stroke.
/// * [`Height`](SculptFamily::Height) → a function of `pre` and a knob, evaluated per texel in the render.
///   **It has no buffer at all**, so a Layer stroke costs 8 B/px where the others cost 12.
///
/// That first asymmetry is why [`PainterTool::set_sculpt_mode`] re-stamps an open shape when the family
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
pub(super) enum SculptFamily {
    Memo,
    Plane,
    Height,
}

/// What the session's memo plane holds — and, because it is the invalidation key, what a change to it
/// throws away.
///
/// The two kernels are memoised by the SAME tile machinery (`super::sculpt_blur`) because they have the
/// same shape: each texel's target is a pure function of `pre` inside a bounded neighbourhood. That bound
/// is [`Self::reach`], and it is the width the read window is grown by — the single number the memo's
/// byte-identity argument rests on.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(super) enum MemoKey {
    /// No memo — the Plane and Height families own no kernel target.
    #[default]
    None,
    /// `blur(pre, r)` — Smooth / Sharpen. `r` in texels.
    Blur(u32),
}

impl MemoKey {
    /// How far, in texels, one output texel reads into `pre` — the tile memo's read-window growth.
    pub(super) fn reach(self) -> u32 {
        match self {
            MemoKey::None => 0,
            MemoKey::Blur(r) => r,
        }
    }
}

impl SculptMode {
    pub(super) fn from_u8(v: u8) -> Self {
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
    pub(super) fn family(self) -> SculptFamily {
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
    /// shows Angle — see [`PainterTool::is_sculpt_chisel`].) Not the same question as [`Self::family`].
    pub(super) fn knob_family(self) -> u8 {
        match self {
            SculptMode::Smooth | SculptMode::Sharpen => 0,
            SculptMode::Flatten | SculptMode::Scrape | SculptMode::Fill | SculptMode::Chisel => 1,
            SculptMode::Layer | SculptMode::Inflate => 2,
        }
    }
}

/// Sculpt settings + the per-stroke session (`docs/Painter/18…` §4).
///
/// The session is born at the first sculpt dab and **dies the moment that gesture is committed**: at
/// pen-up for the freehand methods, at **Apply** for the shape editors (which deliberately keep the stroke
/// open until then), and on Cancel, on a mode switch, and on a document rebind. On undo it is not dropped
/// but RESTORED, in lock-step with the relief ([`crate::undo::SculptSnap`]).
///
/// ## The session does NOT outlive the stroke — and that is a correction, not an omission
///
/// It used to. The session was *parked* at pen-up so that Radius and Smooth↔Sharpen would re-render the
/// stroke the artist had just made, riding the Body card's **"Adjust Last Stroke"** checkbox. Enio's smoke
/// killed it in one sentence: reaching for **Sharpen** — to sharpen somewhere *else* — silently converted
/// the Smooth he had just made into its opposite.
///
/// The mistake was reasoning by analogy from the deposit without asking whether the analogy holds:
///
/// * The **deposit is a substance**. Depth / Body / Source are properties *of the paint that stroke laid*,
///   so "let me keep tuning them" is a coherent offer, and the checkbox makes it.
/// * A **sculpt stroke is an operation**. It leaves nothing behind that has properties — only the relief,
///   as it now is. There is no "the smoothing" sitting there to re-parameterise. Operations are undone and
///   re-done; they are not re-dialled. (Photoshop's Blur tool has no radius-after-the-fact either, and for
///   this reason.)
/// * And the **Mode is not a parameter at all** — it is *which tool*. A verb that rewrites history when you
///   select it is not an adjustment, it is a destruction the artist did not ask for.
///
/// Once the verb cannot be retro-active, a retro-active Radius is *worse* than either rule applied
/// uniformly: two knobs side by side on one card, one rewriting the past and one not, and nothing on
/// screen to say which is which.
///
/// **What stays live is what is still being authored.** A shape editor's stroke is a PREVIEW — it has an
/// Apply button precisely because it is not committed — so while a shape is open the knobs do re-render it
/// ([`Self::refresh_live_sculpt`] gates on `open`). That is not an exception to the rule; it is the rule:
/// *the session lives exactly as long as the gesture is uncommitted.*
pub(crate) struct SculptState {
    /// Sub-mode (`0` Smooth · `1` Sharpen).
    pub(crate) mode: u8,
    /// Radius slider track (`0..1`; mapped to px by [`Self::radius_px`]).
    pub(crate) radius_norm: f32,

    // ── Session (per stroke; see the struct docs) ────────────────────────────────────────────────
    /// The layer's relief as this stroke found it — the frozen source every re-render reads.
    ///
    /// An `Arc` clone of the layer's own plane, so opening a session allocates **nothing**: the first
    /// write to `heights` copies-on-write and leaves this pointing at the original. A stroke that
    /// changes nothing (Strength 0) therefore never forks it, and costs literally zero bytes.
    pub(crate) pre: Arc<Vec<f32>>,
    /// Accumulated per-texel intensity (canvas-sized). Dabs SUM into it; the render clamps the total to 1.
    ///
    /// `Arc`-shared so the undo snapshot captures it with a refcount bump (the first write after a capture
    /// copies-on-write) — the same trick `deform.disp` uses, and for the same reason: every shape-editor
    /// gesture takes a snapshot, and a canvas-sized `Vec` cloned per gesture is a 64 MB copy at 4096².
    pub(crate) amount: Arc<Vec<f32>>,
    /// Plane Offset track (`0..1`; mapped to `∓PLANE_OFFSET_MAX` paint-loads by [`Self::plane_offset`]).
    pub(crate) offset_norm: f32,
    /// **Depth** track (`0..1`; mapped to `∓SCULPT_DEPTH_MAX` paint-loads) — the Height family's knob:
    /// how thick a coat Layer lays, how hard Inflate puffs. Signed: the lower half carves.
    pub(crate) depth_norm: f32,
    /// **Angle** track (`0..1`; mapped to `0..CHISEL_ANGLE_MAX_DEG`) — how far the Chisel's knife is tipped
    /// onto its edge. At `0` the Chisel is Scrape, to the byte.
    pub(crate) angle_norm: f32,
    /// **Smoothness** track (`0..1`; mapped to `0..INFLATE_SMOOTH_MAX_PX` texels) — Inflate's Radius knob.
    ///
    /// A ball dilation makes flat-topped plateaus with a hard edge; this blurs that edge off, the way
    /// Smooth's own Radius sets its kernel. Default **0** — the pure offset, so Inflate is byte-unchanged
    /// until the artist reaches for it.
    pub(crate) smooth_norm: f32,
    /// **Rake** — does the Chisel's V follow the stroke **as it turns**? (Enio 2026-07-14.) Default **on**.
    ///
    /// Both modes take their axis from the stroke; the question is *when they read it*.
    ///
    /// * **On** — every dab uses its **own** heading, so the angle updates in real time and the groove curves
    ///   with the hand. A knife dragged through paint.
    /// * **Off** — the knife enters at the angle the stroke **started** in and **holds it to the end**
    ///   ([`Self::locked_dir`]). Drag a curve and the crease keeps pointing the way you set off: a blade held
    ///   at a fixed attitude, which is a technique rather than a degenerate case.
    ///
    /// So the axis is never a *brush* setting. The first cut of this checkbox made "off" mean the brush's
    /// `dab_angle_deg` — a number the artist never related to the stroke at all — and Enio's correction is
    /// what it is now: *"o tool identifica a direção do traço e mantém o ângulo … até o final. Isso é bom …
    /// mas esse é o modo onde Rake NÃO está checado."*
    ///
    /// It is NOT the brush's `shape.rake`, and the separation is the point: that box turns a silhouette
    /// **image**, and the panel does not even draw it until one is loaded.
    pub(crate) rake: bool,
    /// The heading the knife **entered** at — the axis every dab folds its V about while Rake is OFF. `None`
    /// until the stroke has travelled far enough to have a direction at all.
    ///
    /// Session state, so it dies with the gesture. Cleared by [`PainterTool::restamp_reset_sculpt`] too,
    /// which is what lets a **shape editor** re-aim: drag a Line's endpoint round and the knife re-enters at
    /// the new start angle, instead of remembering one from a shape the artist has since redrawn.
    pub(crate) locked_dir: Option<[f32; 2]>,

    /// **Conserve** (W5, plan §6) — where does the scraped paint go? On: the volume the down-only verbs
    /// (Scrape / Chisel) take off piles up on the spatula's rim as a *bow wave*, instead of being deleted
    /// the way Blender deletes it. Off (default): today's non-conservative behaviour, byte for byte —
    /// the drawing of the pile is unproven until Enio's smoke, so it is opt-in.
    pub(crate) conserve: bool,
    /// The bow-wave field — the removed volume, banked onto the rim, accumulated **per dab** exactly the
    /// way `plane_sum` is (a function of the dab list; the shape editors rebuild it every re-stamp) —
    /// and only while the flag is ARMED, so an unarmed Scrape pays nothing. The render ADDS it after the
    /// verb's delta; on an open shape the checkbox stays live because its toggle re-stamps (the Rake's
    /// own pattern).
    ///
    /// `Arc` for the same reason the others are: the undo snapshot must be a refcount bump. Empty outside
    /// the plane family. It is NOT derived from `pre` (it is derived from the dabs), so — like
    /// `plane_sum`, and per §10.4 — it rides [`crate::undo::SculptSnap`].
    pub(crate) bank: Arc<Vec<f32>>,
    /// Per-dab rim-weight scratch for [`ph2d_painter_brush::height_push::bank_dab_push`] — caller-owned so
    /// a hot stroke allocates nothing. Never read across dabs; never snapshotted.
    pub(crate) bank_scratch: Vec<f32>,
    /// The previous dab's centre this gesture — gives the bank its direction of travel (the bow wave banks
    /// BESIDE the blade, not ahead of it: banking radially leaves half the pile inside the channel the
    /// next dab is about to cut — the deposit's own measured lesson). Session state; dies with the gesture
    /// and on every re-stamp.
    pub(crate) last_bank_center: Option<[f32; 2]>,

    /// `Σ w·plane_d(i)` over the stroke's dabs — the **plane family's** per-texel target, un-normalised.
    ///
    /// The render divides by `amount[i]` (the same `Σ w`) to get the coverage-weighted mean of every plane
    /// that touched the texel. Storing the mean directly would be wrong the moment a second dab arrives;
    /// storing the planes would need the dab list, which the shape editors rebuild from scratch every frame.
    ///
    /// `Arc` for the same reason `amount` is: the undo snapshot must be a refcount bump, not a 64 MB copy.
    /// Empty in the Smooth family — the two targets are mutually exclusive, which is what holds the session
    /// at **12 B/px** with five verbs instead of the sixteen a naive union would cost.
    pub(crate) plane_sum: Arc<Vec<f32>>,
    /// Per-dab `(index, weight)` scratch for the plane fit — caller-owned so a hot stroke allocates nothing
    /// (the silhouette is the expensive sample and it must be taken exactly once; see
    /// [`ph2d_painter_brush::sculpt::accumulate_dab_plane`]). Never read across dabs; never snapshotted.
    pub(crate) fit_scratch: Vec<(u32, f32)>,

    /// The **memo family's** per-texel target — `blur(pre, r)` (Smooth / Sharpen) or `offset(pre, r)`
    /// (Inflate) — memoised tile by tile (see [`super::sculpt_blur`]). Pure derived data: it dies with the
    /// session and is rebuilt lazily, on demand. Empty in the Plane and Height families.
    pub(crate) memo: Vec<f32>,
    /// One flag per tile of [`Self::memo`] — whether that tile has been computed this session.
    pub(crate) memo_done: Vec<bool>,
    /// Which kernel and radius [`Self::memo`] holds. A change to EITHER invalidates every tile — which is
    /// why the kernel is in the key and not merely the radius: Smooth → Inflate keeps the buffer's length
    /// and changes every value in it.
    pub(crate) memo_key: MemoKey,
    /// The layer's **coverage**, **material** and **pixels** as this stroke found them — frozen beside
    /// [`Self::pre`], by `Arc`, so opening a session still allocates nothing.
    ///
    /// ## Why a sculpt session freezes the paint, when §5 says it only writes `h`
    ///
    /// §5 is right about seven verbs and wrong about the eighth, and Enio's second smoke is what said so:
    /// *"inflate não engorda"*. **Inflate is not a reshaping — it is a MOVING.** Dilating the relief pushes
    /// the form's rim outward, onto texels that had no paint; and the light weighs its shading by the
    /// **coverage** (`impasto_light::paint_body(cover) = cover`), so relief standing over bare canvas
    /// renders exactly nothing. The buffer grew. The screen did not. The gate measured the buffer.
    ///
    /// So the matter travels with the height: what arrives at a texel came from the ball's **argmax** (the
    /// packed source [`super::sculpt_offset::blob_dilate`] returns, consumed in
    /// [`super::sculpt_blur::PainterTool::render_inflate`]), and it brings that texel's coverage, its material
    /// and its **colour** — because
    /// paint is a substance, and a substance that moves takes its colour with it. (The same sentence is
    /// already written for W4, the advective family. It arrived early, forced by the one verb that grows.)
    ///
    /// Frozen, like `pre`, so the re-render stays idempotent: every frame recomputes the whole gesture from
    /// the state it started in, never from the state it last left.
    pub(crate) pre_cover: Arc<Vec<u8>>,
    /// The material plane as the stroke found it — see [`Self::pre_cover`].
    pub(crate) pre_mats: Arc<Vec<ph2d_painter_brush::material::MaterialBytes>>,
    /// The active layer's pixels as the stroke found them — see [`Self::pre_cover`]. RGBA8, `4·n` bytes.
    pub(crate) pre_rgba: Arc<Vec<u8>>,
    /// Which layer the session belongs to — and, because the session dies at commit, **this is also what
    /// "a gesture is in flight" means**. `None` ⇒ no session; every guard in this module reads it as both.
    ///
    /// There used to be a separate `open: bool` beside it. Once the session stopped outliving the stroke
    /// the two became coextensive, and a redundant flag is one that eventually disagrees with the field it
    /// shadows — so the target IS the guard.
    pub(crate) layer: Option<crate::tool::RtLayerId>,
    /// The union of this stroke's dab footprints — the window the re-render owns, and the window a
    /// shape editor's re-stamp restores.
    pub(crate) bbox: Option<Region>,
}

impl Default for SculptState {
    fn default() -> Self {
        Self {
            mode: 0,          // Smooth — the one Enio asked for first, and the most valuable of the list
            radius_norm: 0.5, // 8 px: brush-mark scale
            offset_norm: 0.5, // dead centre = offset 0: the plane sits ON the surface it was fitted to
            depth_norm: 0.75, // +0.5 loads: half a stroke's thickness — a coat you can see, not a stunt
            angle_norm: 0.5,  // 30°: a knife on its edge, not a razor and not a flat blade
            smooth_norm: 0.0, // Inflate's edge is hard by default — the artist softens it deliberately
            rake: true, // the groove follows the stroke — what a knife dragged through paint does
            conserve: false, // opt-in until the bow wave's DRAWING survives a smoke (plan §6)
            bank: Arc::new(Vec::new()),
            bank_scratch: Vec::new(),
            last_bank_center: None,
            locked_dir: None,
            pre: Arc::new(Vec::new()),
            amount: Arc::new(Vec::new()),
            plane_sum: Arc::new(Vec::new()),
            fit_scratch: Vec::new(),
            memo: Vec::new(),
            memo_done: Vec::new(),
            memo_key: MemoKey::None,
            pre_cover: Arc::new(Vec::new()),
            pre_mats: Arc::new(Vec::new()),
            pre_rgba: Arc::new(Vec::new()),
            layer: None,
            bbox: None,
        }
    }
}

impl SculptMode {
    /// The verbs the **Conserve** flag applies to — the pure REMOVERS, whose taken volume has one honest
    /// destination (the rim). Flatten moves volume both ways and Fill only adds; conserving those needs a
    /// design for where the *added* volume comes FROM, which is a decision, not a flag (plan §6 names
    /// Scrape; the Chisel is Scrape with a folded blade).
    pub(super) fn conserves(self) -> bool {
        matches!(self, SculptMode::Scrape | SculptMode::Chisel)
    }
}

impl SculptState {
    /// **The one door for "is the bow wave on?"** — the flag AND a verb it governs. Both the dab
    /// accumulation (the bite + bank) and the render (the `+ bank` term) ask HERE: two copies of this
    /// predicate would diverge, and the half that drifted would either tax unarmed strokes or leak the
    /// pile past its flag.
    pub(super) fn conserve_active(&self) -> bool {
        self.conserve && self.mode_enum().conserves()
    }

    /// The kernel radius in px (`1..=SCULPT_RADIUS_MAX_PX`). Never 0 — a zero-radius blur is a no-op, and
    /// a control whose bottom end silently does nothing is a control that lies.
    pub(super) fn radius_px(&self) -> u32 {
        let t = self.radius_norm.clamp(0.0, 1.0);
        (1.0 + t * (SCULPT_RADIUS_MAX_PX - 1.0)).round() as u32
    }

    /// The plane's Offset in **paint-loads**, from the slider's `0..1` track.
    ///
    /// `−PLANE_OFFSET_MAX ..= +PLANE_OFFSET_MAX` for Flatten / Scrape / Fill — dead centre is exactly `0.0`,
    /// the plane sitting on the surface it was fitted to.
    ///
    /// ## The Chisel's track is NEGATIVE ONLY (Enio, smoke of 2026-07-14)
    ///
    /// A chisel **cuts in**. Its target is `plane + V + offset` and it only takes material *away*
    /// (`min(Δ, 0)`), so the offset is the one thing that decides whether the blade reaches below the
    /// surface at all. At `offset ≥ 0` the plane is the least-squares fit *through* the paint, so all the
    /// blade can reach is whatever already stands above that fit — the brush marks. It nicks the peaks. It
    /// cannot make a groove, which is the only thing anybody picks this verb up to do, and the upper half of
    /// the track is a slow fade into "Scrape, but weaker".
    ///
    /// So the Chisel's track runs `−PLANE_OFFSET_MAX ..= 0` and the whole of it carves. The default
    /// (`offset_norm = 0.5`) lands at half a load below the surface: a cut you can see.
    pub(super) fn plane_offset(&self) -> f32 {
        let t = self.offset_norm.clamp(0.0, 1.0);
        if matches!(self.mode_enum(), SculptMode::Chisel) {
            (t - 1.0) * PLANE_OFFSET_MAX
        } else {
            (t - 0.5) * 2.0 * PLANE_OFFSET_MAX
        }
    }

    /// Inflate's **Smoothness** (Radius) in texels — how far the ball's hard edge is blurred. `0` = the raw
    /// offset. Whole texels: a box blur's radius is a count of taps.
    pub(super) fn inflate_smooth_px(&self) -> u32 {
        (self.smooth_norm.clamp(0.0, 1.0) * INFLATE_SMOOTH_MAX_PX).round() as u32
    }

    /// Which kernel target the active verb needs memoised — [`MemoKey::None`] outside the memo family.
    ///
    /// Inflate's radius is its **Depth**, converted from paint-loads into pixels through the light's
    /// [`DEPTH_UNIT_PX`](super::impasto_light::DEPTH_UNIT_PX). The ball is a ball in the space the artist
    /// *sees*, and that constant is the only place the two axes of a height field are reconciled. The
    /// **Smoothness** rides the key too, so turning it rebuilds the memo like any other radius change.
    pub(super) fn memo_key(&self) -> MemoKey {
        match self.mode_enum() {
            SculptMode::Smooth | SculptMode::Sharpen => MemoKey::Blur(self.radius_px()),
            // Inflate is NOT memoised — its variable-radius ball depends on the live `amount` field. See
            // `super::sculpt_blur::render_inflate`; it reads `depth()` and `inflate_smooth_px()` straight.
            _ => MemoKey::None,
        }
    }

    /// The Height family's **Depth** in paint-loads (`−SCULPT_DEPTH_MAX ..= +SCULPT_DEPTH_MAX`).
    pub(super) fn depth(&self) -> f32 {
        (self.depth_norm.clamp(0.0, 1.0) - 0.5) * 2.0 * SCULPT_DEPTH_MAX
    }

    /// The Chisel's **Angle** in degrees (`0..=CHISEL_ANGLE_MAX_DEG`).
    pub(super) fn chisel_angle_deg(&self) -> f32 {
        self.angle_norm.clamp(0.0, 1.0) * CHISEL_ANGLE_MAX_DEG
    }

    /// The Chisel's tilt — the plane's rise **in paint-loads per texel** of sideways travel out of the
    /// stroke's axis.
    ///
    /// Zero in every verb but Chisel, and that is what lets the plane accumulator be ONE function: the
    /// `+ tilt·|lateral|` term vanishes and Flatten / Scrape / Fill are byte-for-byte the Wave 2 kernel.
    ///
    /// ## `tan(angle) ÷ DEPTH_UNIT_PX`, and the division is the whole correctness of the tool
    ///
    /// **A height field's two axes are not the same unit.** `x` is texels; `h` is paint-loads. An angle is a
    /// ratio of *lengths*, so it only means anything once the height has been converted to length — and the
    /// converter is the one the LIGHT uses, `DEPTH_UNIT_PX` (a load of paint stands 16 px tall).
    ///
    /// The first cut of this shipped `tan(angle)` raw. At 36° that tilts the plane by **0.73 loads per
    /// texel** — 8.7 loads across the brush's own footprint, four times `H_CEIL`. The "angle" was a number in
    /// a space with no geometry in it, and the V it cut was a cliff.
    ///
    /// It is the same mistake `inflate_nz` exists to avoid, one verb over, and I only avoided it there
    /// because I went looking. The rule is worth stating once for the whole family: **anything geometric —
    /// a normal, an angle, a slope the artist can see — crosses `DEPTH_UNIT_PX` on its way in.**
    ///
    /// (The `tan` is computed once per render, not per dab and not per texel. HR-5 governs code in a declared
    /// *deterministic* mode; this is an editor knob, and the crate already leans on `sqrt` and `powf`. What
    /// HR-5 would forbid is a transcendental in the per-texel loop, and there is not one.)
    pub(super) fn chisel_tilt(&self) -> f32 {
        if !matches!(self.mode_enum(), SculptMode::Chisel) {
            return 0.0;
        }
        let slope_px = self.chisel_angle_deg().to_radians().tan().max(0.0); // rise/run, both in PIXELS
        slope_px / super::impasto_light::DEPTH_UNIT_PX // …and back into paint-loads per texel
    }

    pub(super) fn mode_enum(&self) -> SculptMode {
        SculptMode::from_u8(self.mode)
    }
}

impl PainterTool {
    // ── UI-edit setters (the single clamp source; `handle_panel_event` forwards raw panel values here) ──

    /// Select the sub-mode from the segmented index (out of range clamps to the last mode).
    ///
    /// **A change of FAMILY re-stamps an open shape** rather than merely re-rendering it, and that is not
    /// belt-and-braces: the plane family's per-texel target (`plane_sum`) is a function of the DAB LIST,
    /// which the render no longer has. Switching Smooth → Scrape and only re-rendering would divide by an
    /// all-zero `plane_sum` and pull the whole footprint to height 0 — a flattening to the floor, dressed
    /// as a scrape. Re-stamping rebuilds the target from the dabs, and it is precisely what the house
    /// already does when Size / Spacing / Falloff change (`refill_if_appearance_changed`).
    ///
    /// A change WITHIN a family (Smooth ↔ Sharpen, Scrape ↔ Fill) reuses the target it already has, so the
    /// cheap re-render is the correct one.
    pub fn set_sculpt_mode(&mut self, m: u8) {
        let before = self.paint.sculpt.mode_enum().family();
        self.paint.sculpt.mode = m.min(SCULPT_MODE_COUNT - 1);
        self.sync_stroke_heading_need();
        if self.paint.sculpt.mode_enum().family() == before {
            self.refresh_live_sculpt();
        } else {
            self.drop_stale_family_target();
            self.refill_open_shape(); // rebuild the new family's target from the dabs (no-op if none open)
            self.refresh_live_sculpt();
        }
    }

    /// Set the kernel Radius from the slider's `0..1` track (Smooth family).
    pub fn set_sculpt_radius(&mut self, t: f32) {
        self.paint.sculpt.radius_norm = t.clamp(0.0, 1.0);
        self.refresh_live_sculpt();
    }

    /// Set the plane Offset from the slider's `0..1` track (Plane family).
    ///
    /// No re-stamp: the Offset is a **rigid shift** of the plane, and `Σ w·(plane + off) = plane_sum +
    /// off·amount`, so the render applies it with one add. That is the whole reason it is not folded into
    /// `plane_sum` at accumulation time — it keeps the slider live on an open shape for free.
    pub fn set_sculpt_offset(&mut self, t: f32) {
        self.paint.sculpt.offset_norm = t.clamp(0.0, 1.0);
        self.refresh_live_sculpt();
    }

    /// Set the **Depth** from the slider's `0..1` track (Layer / Inflate).
    ///
    /// No re-stamp — neither verb's target owes anything to the dab list. Layer's is `pre + Depth`,
    /// evaluated per texel in the render. Inflate's is `offset(pre, Depth)`, and Depth is that offset's
    /// *radius*, so turning this knob changes the memo's [`MemoKey`] and the render rebuilds it from `pre`
    /// — the same lazy, tile-by-tile rebuild that the Radius slider already triggers for Smooth.
    pub fn set_sculpt_depth(&mut self, t: f32) {
        self.paint.sculpt.depth_norm = t.clamp(0.0, 1.0);
        self.refresh_live_sculpt();
    }

    /// Set Inflate's **Smoothness** (Radius) from the slider's `0..1` track.
    ///
    /// No re-stamp: like the Smooth Radius, it changes the memo's [`MemoKey`], and the render rebuilds it
    /// from `pre` — a bigger blur on the offset, nothing the dab list owns.
    pub fn set_sculpt_smooth(&mut self, t: f32) {
        self.paint.sculpt.smooth_norm = t.clamp(0.0, 1.0);
        self.refresh_live_sculpt();
    }

    /// Set the Chisel's **Angle** from the slider's `0..1` track.
    ///
    /// This one DOES re-stamp: the tilt is folded into `plane_sum` as each dab lands (it is part of the
    /// plane the dab fitted), so the accumulated target has the old angle baked in. Same reason a family
    /// switch re-stamps, and the same remedy.
    pub fn set_sculpt_angle(&mut self, t: f32) {
        self.paint.sculpt.angle_norm = t.clamp(0.0, 1.0);
        self.refill_open_shape(); // the V is in `plane_sum`; only a re-stamp can change it
        self.refresh_live_sculpt();
    }

    /// Toggle the Chisel's **Rake**. Re-stamps: the V's axis is baked into `plane_sum` as each dab lands, so
    /// only a re-stamp can turn the knife (the same reason the Angle slider re-stamps).
    /// Toggle **Conserve** (W5): whether the down-only verbs bank the volume they remove onto the rim.
    /// Live on an open shape the way the Rake is: the bank is baked into the ACCUMULATION (only armed
    /// strokes pay the bite — unarmed Scrape costs what it always cost), so the toggle re-stamps the
    /// standing shape and the pile materialises — or vanishes — with the flag.
    pub fn toggle_sculpt_conserve(&mut self) {
        self.paint.sculpt.conserve = !self.paint.sculpt.conserve;
        self.refill_open_shape(); // the bank is a fact about the (armed) dab list; only a re-stamp changes it
        self.refresh_live_sculpt();
    }

    pub fn toggle_sculpt_rake(&mut self) {
        self.paint.sculpt.rake = !self.paint.sculpt.rake;
        self.paint.sculpt.locked_dir = None; // re-aim: the lock belongs to the stroke, not to the checkbox
        self.refill_open_shape(); // the V's axis is baked into `plane_sum`; only a re-stamp can turn it
        self.refresh_live_sculpt();
    }

    /// Whether the Chisel's V follows the stroke — what the card's checkbox shows.
    #[must_use]
    pub fn sculpt_rake(&self) -> bool {
        self.paint.sculpt.rake
    }

    /// The axis the Chisel's V folds about, for a dab whose own heading is `dir`.
    ///
    /// **Rake on** → the dab's own heading: the angle updates in real time and the groove curves with the
    /// hand. **Off** → [`SculptState::locked_dir`], the heading the stroke *entered* at, held to the end.
    ///
    /// One function, so the checkbox and the kernel cannot disagree about what *the direction of the stroke*
    /// means. Both modes read `Dab::dir` — which is why the engine settles a heading for the Chisel whether
    /// or not it rakes: a locked angle still has to be locked to *something the artist did*.
    ///
    /// A zero heading (a dab emitted before the stroke has travelled at all) collapses the V to a flat
    /// scrape. The warm-up makes that unreachable, and the fallback stands anyway: *cannot happen* is a
    /// claim, and this is one `[0, 0]` away from a silently blunt tool.
    pub(super) fn chisel_dir(&self, dir: [f32; 2]) -> [f32; 2] {
        if self.paint.sculpt.rake {
            return dir;
        }
        self.paint.sculpt.locked_dir.unwrap_or(dir)
    }

    /// Free the target the OTHER families own, on a family switch.
    ///
    /// Not an optimisation — it is what keeps the promise the cost gate makes. A session that carried every
    /// family's target would cost 16 B/px instead of 12, and the moment that is true "the session costs
    /// 12 B/px" becomes a sentence in a comment rather than a fact about the program. Both buffers are
    /// rebuildable: `memo` from `pre` (lazily, tile by tile) and `plane_sum` from the re-stamp that is
    /// about to run anyway. The **Height** family holds neither, so switching TO it frees both — a Layer
    /// stroke costs 8 B/px.
    fn drop_stale_family_target(&mut self) {
        let (keep_memo, keep_plane) = match self.paint.sculpt.mode_enum().family() {
            SculptFamily::Memo => (true, false),
            SculptFamily::Plane => (false, true),
            SculptFamily::Height => (false, false),
        };
        if !keep_plane {
            self.paint.sculpt.plane_sum = Arc::new(Vec::new());
        }
        if !keep_memo {
            self.paint.sculpt.memo = Vec::new();
            self.paint.sculpt.memo_done = Vec::new();
            self.paint.sculpt.memo_key = MemoKey::None;
        }
    }
}
