//! `BrushSpec` — the brush parameters.
//!
//! Clean-room model of the relevant fields of Blender's `Brush` (`makesdna/DNA_brush_types.h`):
//! radius, strength, the per-dab build-up (`flow`/`alpha`), spacing, blend mode, the distance
//! falloff curve, jitter, and the paint colour. Fields the texture painter does not need yet
//! (texture slots, stencil masks, projection options) are deliberately omitted — see
//! `docs/Painter/02_plano_de_implementacao.md` for what each phase adds.

use crate::blend::BrushBlend;
use crate::falloff::Falloff;
use crate::falloff_curve::FalloffCurve;
use crate::height::{DepthSource, DrawTo};
use crate::stroke_method::{JitterUnit, StrokeMethod};
use crate::symmetry::SymmetrySettings;
use crate::texture::TextureSettings;

/// Largest brush radius the engine will allocate a dab for, in pixels. Derived from the editor
/// overlay budget (HR-4): a 4096-px-radius dab would be an 8k² bbox — far past interactive. This
/// caps the bbox, not the artist's intent (the value is clamped, not rejected).
pub const MAX_BRUSH_RADIUS_PX: f32 = 4096.0;

/// Airbrush **Rate** (timer period, seconds) soft-range floor — the fastest spray the UI slider
/// reaches (~100 Hz). Blender's `rate` `ui_range` is `0.01..1.0` s (default `0.1`); the hard RNA
/// range is wider, but `rna_brush.cc` is not in the vendored checkout, so this is the well-known
/// Blender soft range, not an in-tree-verified value. The default `0.1` IS verified
/// (`DNA_brush_types.h:232`). See [`crate::Stroke::tick`].
pub const AIRBRUSH_RATE_MIN_S: f32 = 0.01;
/// Airbrush **Rate** soft-range ceiling (slowest spray, ~1 Hz). See [`AIRBRUSH_RATE_MIN_S`].
pub const AIRBRUSH_RATE_MAX_S: f32 = 1.0;

/// Parameters of a single brush. Cheap to copy; the stroke engine reads it per dab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrushSpec {
    /// Dab radius in image pixels (UI label "Radius"). Clamped to `[0.5, MAX_BRUSH_RADIUS_PX]`.
    pub radius_px: f32,
    /// Plateau before the falloff begins, `0..1`. `0` = pure falloff curve; `1` = hard disk.
    /// Mirrors Blender's brush `hardness` as a distance remap applied before [`Self::falloff`].
    pub hardness: f32,
    /// Maximum opacity a single stroke can build to, `0..1` (Blender "Strength"). Enforced as a
    /// per-stroke accumulation cap by the stroke engine (later phase), not by a single dab.
    pub strength: f32,
    /// Per-dab build-up, `0..1` (Blender "Alpha"/flow). Scales every dab's coverage.
    pub flow: f32,
    /// Distance between dabs as a fraction of the diameter (Blender "Spacing"; default `0.10`).
    pub spacing: f32,
    /// How the dab colour combines with the layer.
    pub blend: BrushBlend,
    /// Radial intensity profile.
    pub falloff: Falloff,
    /// Random per-dab position offset when [`Self::jitter_unit`] is [`JitterUnit::Brush`]: a
    /// fraction of the diameter, `0..1` (Blender `Brush.jitter`, default `0.0`,
    /// `DNA_brush_types.h:221`). Max radial offset ≈ `jitter × diameter`.
    pub jitter: f32,
    /// Paint colour, straight RGB in `[0, 1]` in the layer's native space.
    pub color: [f32; 3],
    /// The editable profile used when [`Self::falloff`] is [`Falloff::Custom`]
    /// (ignored otherwise). Kept inline so the spec stays `Copy`/alloc-free.
    pub custom_falloff: FalloffCurve,

    // ── Stroke panel (Blender `paint_stroke.cc` / `DNA_brush_types.h`) ──────────────
    /// How the pointer path becomes dabs (Blender `stroke_method`, default `Space`,
    /// `DNA_brush_types.h:213`).
    pub stroke_method: StrokeMethod,
    /// "Adjust Strength for Spacing" — normalise total deposited opacity so a densely-spaced
    /// stroke doesn't pile up to full opacity (Blender `BRUSH_SPACE_ATTEN`, default ON,
    /// `DNA_brush_types.h:206`). Only applies for spacing < 100% (see [`Self::space_overlap_factor`]).
    pub space_attenuation: bool,
    /// **Accumulate** (Blender `BRUSH_ACCUMULATE`, bit 13, default OFF): when OFF, a single stroke is
    /// capped at [`Self::strength`] — overlapping dabs (incl. passing the brush back over itself) do
    /// NOT build past Strength (a per-stroke coverage mask, the classic "alpha mask"). When ON, every
    /// dab adds on top so opacity builds up without the per-stroke cap (airbrush-like buildup). At
    /// Strength `1.0` the two are identical (the cap is full coverage). See [`crate::dab`].
    pub accumulate: bool,
    /// Dash "on" fraction of each dash period, `0..1` (Blender `dash_ratio`, default `1.0` = solid,
    /// `DNA_brush_types.h:275`).
    pub dash_ratio: f32,
    /// Dash period length in dab-slots (Blender `dash_samples`, default `20`,
    /// `DNA_brush_types.h:276`). `0` is treated as "no dash" (solid).
    pub dash_samples: u32,
    /// Unit for [`Self::jitter`] / [`Self::jitter_absolute_px`] (Blender `BRUSH_ABSOLUTE_JITTER`).
    pub jitter_unit: JitterUnit,
    /// Random per-dab offset in pixels when [`Self::jitter_unit`] is [`JitterUnit::View`]
    /// (Blender `jitter_absolute`, default `0`, `DNA_brush_types.h:223`). Max radial offset
    /// ≈ `2 × jitter_absolute_px`, independent of brush size.
    pub jitter_absolute_px: f32,
    /// Number of most-recent raw input samples box-averaged before processing, `>= 1`
    /// (Blender `input_samples`, default `1`, `DNA_brush_types.h:216`).
    pub input_samples: u32,
    /// Stroke **stabilizer** intensity, `0..1` — a single "how regular" knob (substitute for
    /// Blender's smooth-stroke). `0` = the raw path (straight chords through every sample, exact,
    /// real-time); higher = a stronger lazy-mouse position filter (removes hand tremor) plus more
    /// inter-sample curvature, so a shaky hand draws a clean, regular line. Real-time (causal); the
    /// lag is proportional to the intensity the artist dials in. See [`Stroke`].
    pub stabilizer: f32,
    /// Airbrush emission period in seconds (Blender `rate`, default `0.1` = 10 Hz,
    /// `DNA_brush_types.h:232`). Used only by [`StrokeMethod::Airbrush`]; the tool's tick drives it.
    pub airbrush_rate_s: f32,
    /// "Edge to Edge" — Anchored only (Blender `BRUSH_EDGE_TO_EDGE`, `DNA_brush_enums.h:376`).
    /// OFF: the single stamp is centred on the anchor (press point) with radius = drag distance.
    /// ON: it's centred on the midpoint anchor→cursor with half that radius, so the dab spans
    /// edge-to-edge from the press point to the cursor. See the Anchored arm of [`Stroke::extend`].
    pub edge_to_edge: bool,

    // ── Grain slot (Blender `MTex` / `brush_painter_2d_tex_mapping`; = Procreate "Grain") ──────
    /// Brush **Grain**: a per-texel mask that modulates each dab's coverage *inside* the silhouette
    /// (Blender's brush `mtex`; the Procreate Grain). Default [`TextureSettings::kind`] is `None` (no
    /// modulation). The internal field name is `texture` for back-compat; the UI labels it "Grain".
    /// See [`crate::texture`] and `docs/Painter/05_design_dois_slots_textura.md`.
    pub texture: TextureSettings,
    /// **Grain Depth** (Procreate): how strongly the Grain bites, `0..1`. `1.0` (default) = the Grain
    /// fully modulates (`g_eff = g`, the historical behaviour); `0.0` = the Grain has no effect
    /// (`g_eff = 1`, silhouette only). `g_eff = 1 + (g − 1)·depth`. See [`crate::dab`].
    pub grain_depth: f32,

    // ── Shape slot (= Procreate "Shape": the silhouette / tip of the dab) ──────────────────────
    /// Brush **Shape**: the silhouette (alpha tip) of each dab, as an image. When inactive (default
    /// `kind == None`, or `Image` with no pixels loaded) the silhouette is the procedural
    /// [`Self::falloff`] — so a default brush is byte-identical to before. When a Shape **image** is
    /// assigned it **replaces** the falloff as the silhouette (a crisp imported tip; the falloff goes
    /// inactive in the UI). Pixels live in `PaintState` (heavy), like the Grain image. The `mapping` is
    /// always View-plane (dab-relative); `angle_deg`/`rake` rotate the tip frame (a random per-dab spin
    /// is the Stroke **Jitter Rotate**, which turns the whole stamp).
    pub shape: TextureSettings,

    /// **Something downstream reads [`crate::Dab::dir`]** — so the stroke must settle a heading before it
    /// emits, even though neither texture slot rakes.
    ///
    /// It is not a property of the brush, and it does not pretend to be: it is the **channel** by which a
    /// consumer of the dab list tells the stroke engine that the dabs are useless without a direction. The
    /// engine only ever sees the spec, so the spec is where the message has to travel.
    ///
    /// It exists because the heading **warm-up** — holding the opening dabs until enough travel has settled
    /// a direction ([`crate::heading`]) — was gated on `texture.rake || shape.rake`, i.e. on an enumeration
    /// of the readers of `dir` that was written when the two texture slots were the only ones. The Sculpt
    /// **Chisel** is a third reader: it carves a V about the stroke's axis. With the warm-up off its opening
    /// dabs carry `dir = [0, 0]`, the V collapses (`perp = [0,0]` ⇒ the tilt term is zero), and every chisel
    /// stroke **begins as a plain Scrape** until the heading catches up. Two doors to the same question, and
    /// they diverged: the artist unticks a checkbox about a *silhouette image* and the *groove* loses its
    /// start. So the enumeration gained its missing member instead of the Chisel borrowing someone else's.
    pub needs_heading: bool,

    // ── Dab flatten + rotate (Procreate "Shape" panel gizmo; Enio 2026-06-26) ───────────────────
    /// **Flatten** the dab footprint, `0..1`: `0` = round, → `1` = a thin ellipse (squished on the
    /// minor axis). Deforms the falloff envelope AND everything sampled in the footprint (the Shape
    /// silhouette + the View-mapped Grain) into a rotated ellipse — each slot keeps its own relative
    /// Size/Offset/Angle on top. Clamped to [`crate::footprint::DAB_FLATTEN_MAX`] so the minor axis
    /// never collapses. See [`Self::footprint_deform`].
    pub dab_flatten: f32,
    /// **Dab rotation** in whole degrees (`0..=360`) of the flatten/rotate frame — rotates the
    /// elliptical footprint and, even when round, the pattern sampled within it.
    pub dab_angle_deg: u16,

    // ── Per-dab randomize (Blender "Color → Randomize" + two PH2D extras; see [`crate::jitter`]) ──
    /// Master switch for **Randomize Color** (the subsection's enable checkbox). When off, every dab
    /// uses [`Self::color`] unchanged. Default `false` (clean-room model of Blender's brush
    /// `BRUSH_USE_RAND_HUE`/`SAT`/`VAL` group toggle).
    pub color_jitter_enabled: bool,
    /// **Hue** randomize amount, `0..1` — the per-dab hue offset is `±amount` on the colour wheel.
    pub color_jitter_hue: f32,
    /// **Saturation** randomize amount, `0..1` — per-dab offset `±amount`, clamped to `[0, 1]`.
    pub color_jitter_sat: f32,
    /// **Value** randomize amount, `0..1` — per-dab offset `±amount`, clamped to `[0, 1]`.
    pub color_jitter_val: f32,
    /// **Jitter Scale** (PH2D, not Blender): per-dab radius scatter, `0..1`. Each dab's radius is
    /// multiplied by `1 ± amount`, clamped so it never collapses. `0` = no scatter.
    pub jitter_scale: f32,
    /// **Jitter Rotate** (PH2D, not Blender): per-dab stamp-rotation scatter, `0..1`. Each dab's WHOLE
    /// stamp — the Shape silhouette AND the Grain frame together — is rotated by `±(amount · 180°)`.
    /// Visible with a Shape or a texture (a bare round falloff is isotropic); mappings that don't use
    /// per-dab rotation (Stencil) opt out. `0` = no scatter.
    pub jitter_rotate: f32,
    /// **Jitter Spacing** (PH2D, not Blender): per-gap scatter of the distance between dab centres,
    /// `0..1`. Each gap along the Space walk is multiplied by `1 ± amount` (clamped so it stays ≥ 1px),
    /// so dabs land irregularly instead of on a perfectly even grid. `0` = even spacing.
    pub jitter_spacing: f32,

    // ── Drawing symmetry (mirror / radial; see [`crate::symmetry`]) ──────────────────────────────
    /// **Symmetry**: replicate every dab across a mirror line (X / Y / a custom line) or as N radial
    /// copies about a centre. Default disabled, so a default brush paints byte-identically. The
    /// geometry (centre / custom-line direction) is in canvas pixels, resolved by the tool.
    pub symmetry: SymmetrySettings,

    // ── Watercolor (wet-media look; no fluid sim — see `docs/Painter/08_plano_aquarela_edge_grain_pigment.md`) ──
    /// Master switch for the **Watercolor** section. Default `false`, so a default brush is
    /// byte-identical: the edge/granulation/pigment paths below are all skipped unless this is on.
    pub watercolor: bool,
    /// **Edge darkening** gain, `0..` (the watercolor "fringe"). At stroke end a blur-difference of
    /// the stroke coverage pools pigment at the receding boundary (Curtis 1997 §4.3.3, image-space
    /// form). `0` = no rim. Only read when [`Self::watercolor`] is on.
    pub edge_gain: f32,
    /// **Edge spread**: blur radius (canvas px) of the coverage used by the edge-darkening pass — a
    /// wider spread makes a softer, thicker rim. Only read when [`Self::watercolor`] is on.
    pub edge_spread: f32,
    /// **Granulation**, `0..1`: how strongly the Grain sample gates deposition into the paper tooth's
    /// valleys (a non-linear gate, vs the linear [`Self::grain_depth`] multiply). `0` = the historical
    /// linear multiply (byte-identical). Pairs with a canvas-anchored Grain (`Tiled` + `Grain` kind).
    pub granulation: f32,
    /// **Pigment** build-up toggle: when on, dab colour composites subtractively (Kubelka–Munk) so
    /// wet-on-wet layers mix like real paint (blue+yellow → green) instead of the plain `Mix` blend.
    /// Default `false`. Only read when [`Self::watercolor`] is on.
    pub pigment: bool,
    /// **Pigment mix** amount, `0..1`: how much the subtractive path is applied vs the plain blend.
    /// Only read when [`Self::watercolor`] and [`Self::pigment`] are on.
    pub pigment_mix: f32,
    /// **Fill** density of the wash, `0..1` (wet_edges `fillDensity`): the optical density contributed by
    /// the flat interior of the stroke (before the edge term). Low = a translucent glaze, high = a
    /// saturated pool. Only read by the watercolor render-path (`ph2d-tool-painter`), when
    /// [`Self::watercolor`] is on. The whole appearance is reconstructed optically from this + the edge +
    /// granulation, not deposited per dab — see `docs/Painter/10_aquarela_render_path_preset_papers.md`.
    pub fill: f32,
    /// **Depth**, the Beer–Lambert optical-depth scale `> 0` (wet_edges `DEPTH`): transmittance per
    /// channel is `Tᵢ = pigmentᵢ^(D·depth)` in linear light, so a larger depth darkens the absorbed
    /// channels faster (the hue shifts more with thickness). Only read by the render-path.
    pub depth: f32,
    /// **Opacity** (pigment body / hiding power), `0..1`. Pure Beer–Lambert (`opacity = 0`) can only
    /// SUBTRACT light, so a light-valued pigment — yellow, light blue — leaves its bright channels at
    /// `Tᵢ ≈ 1` and barely deposits over white paper ("azul e amarelo quase não aparecem", Enio
    /// 2026-07-10). Body models the pigment's SCATTERING: it lays the pigment's own colour over the
    /// transmittance result with a coverage `opacity·(1 − e^{−k·od})` (od = deposit), so light pigments
    /// show at their hue regardless of value — transparent watercolor toward gouache. `0` (default-off
    /// for a plain brush; the render-path default is `0.4`) is byte-identical (the fold is skipped).
    /// Distinct from [`Self::flow`] (deposit RATE). Only read by the render-path when `watercolor` is on.
    pub opacity: f32,
    /// **Warp** amplitude in canvas px (wet_edges `warpAmp`): a fractal value-noise field displaces the
    /// coverage sampling so the wash boundary is organic (ragged), not a clean disc. `0` = a crisp edge.
    /// Only read by the render-path.
    pub warp: f32,
    /// **Smudge** amount `0..1`: the TRUE-SMEAR strength — each dab physically DRAGS the pre-stroke
    /// base's paint from the previous dab centre to its own (Krita Color Smudge "Smearing" / Blender
    /// `paint_2d_lift_smear`), and the wash composites over the smeared base. `0` (default) skips the
    /// path → byte-identical. Only read by the render-path when [`Self::watercolor`] is on.
    pub wet_smudge: f32,
    /// **Wet** amount `0..1` — wet-on-wet rewetting, per-pixel in the optical composite (no brush
    /// state, no physics). Two components: (a) the wash's OWN pigment redistributes — the interior
    /// thins while the receding front pools harder and raggeder (paper-tooth modulated), on any canvas;
    /// (b) where the wash covers already-painted paint, that paint LIFTS (the base lightens toward the
    /// paper), DISSOLVES (its colour bleeds up to `edge_spread` px through the wet region — a one-shot
    /// diffusion blur), and POOLS back into the wash's density at the front (the bloom). Wet also
    /// drives the subtractive (RYB) paint-mix over the base up to `max(Mix, wet)`, Pigment checked or
    /// not — a wet wash MIXES with what it rewets ("o segredo", Enio 2026-07-06). `0` (default) skips
    /// everything → byte-identical. Only read by the render-path when [`Self::watercolor`] is on.
    pub wet_rewet: f32,
    /// **Charge** `0..1` — the mixer-brush fresh-paint reserve (Procreate Wet Mix, `docs/Painter/07`
    /// §4). `1` (default) = the brush deposits pure fresh colour (the mixer is skipped → byte-
    /// identical); below `1` the brush picks up the surface it crosses (weighted by paint presence)
    /// and blends it into the deposit — `pickup = 1 − charge`. The pickup reads the FROZEN pre-stroke
    /// base (never the live canvas), so it can't self-feed. Only read by the render-path when
    /// [`Self::watercolor`] is on.
    pub wet_charge: f32,
    /// **Dilution** `0..1` — how much water thins the deposit (Procreate Wet Mix). `0` (default) =
    /// full-strength deposit (byte-identical); `1` = a near-transparent wash (the dab lays down less
    /// coverage: `flow = 1 − dilution`). Only read by the render-path when [`Self::watercolor`] is on.
    pub wet_dilution: f32,
    /// **Pull** `0..1` — the mixer's colour-carry / smudge length (Procreate Wet Mix). `0` (default) =
    /// the pickup resamples the surface every dab (no carry; the deposit tracks the local surface);
    /// toward `1` the picked-up colour LAGS and is dragged along the stroke (a red crossed early
    /// bleeds far downstream). Inert unless [`Self::wet_charge`] < 1. Only read by the render-path.
    pub wet_pull: f32,

    // ── Watercolor Paper slot + Granulation coupling (render-path only) ────────────────────────────
    /// The **Paper** slot: the substrate tooth (its own full texture section — a procedural `Paper*`/other
    /// kind or an `Image` tagged layer, `docs/Painter/10…` §5). Always canvas-anchored (Tiled), a fixed
    /// height-field over the image. `None` (default) → the render-path's built-in paper noise. The
    /// **Granulation** map is the existing [`Self::texture`] (Grain) slot — the section labelled "Grain".
    pub paper: TextureSettings,
    /// **Granulation "Same as Paper"**: when true (default), the granulation settles into the *paper's* own
    /// tooth (the common case — heavy pigment pools in the paper valleys), so the Grain slot texture is
    /// ignored and only the [`Self::granulation`] amount applies. False → the Grain slot IS the granulation
    /// map. (Shown in the Grain section only in watercolor mode.)
    pub granulation_use_paper: bool,
    /// **Paper Depth**, `0..1`: how strongly the [`Self::paper`] tooth textures the wash (the paper's
    /// substrate bite, independent of the granulation amount). `0` = a flat wash; `1` = full tooth.
    pub paper_depth: f32,
    /// **Shape "Automatic"** (watercolor mode, Enio 2026-07-07): `true` (default) = the watercolor
    /// stamp uses its own built-in silhouette (the two-segment feather disc — byte-identical to the
    /// historical behaviour, the Shape section is not consulted). `false` = the Shape section drives
    /// the watercolor coverage stamp: [`Self::falloff`] (incl. the [`crate::Falloff::Watercolor`]
    /// preset = the built-in curve) + [`Self::hardness`] + the Shape slot image/procedural +
    /// rotation. Only read by the watercolor render-path; inert for a plain brush.
    pub watercolor_shape_auto: bool,

    // ── Impasto (paint thickness + relief; see [`crate::height`] and `docs/Painter/16…`) ──────────
    /// Master switch for **Impasto**. Default `false`, and while it is off **none** of the fields
    /// below is read — a default brush deposits no height and the canvas is byte-identical to a
    /// build without impasto. (Every `impasto_*` field is prefixed because [`Self::depth`] and
    /// [`Self::opacity`] already belong to the *watercolor* optics — a bare `depth` here would have
    /// collided with the Beer–Lambert one and quietly crossed the two systems.)
    pub impasto: bool,
    /// **Depth**, `-1..1` — how much thickness one full-coverage dab lays down, in the height field's
    /// own units. Signed: positive **lifts** paint off the canvas, negative **carves** into it (the
    /// Painter "Negative Depth"). `0` = flat (no relief), so the pass is inert even with the master
    /// switch on.
    pub impasto_depth: f32,
    /// Which part of the dab mask sculpts the relief — see [`DepthSource`]. Default
    /// [`DepthSource::Uniform`] (a smooth plateau); [`DepthSource::Grain`] is the bristle look.
    pub impasto_source: DepthSource,
    /// Whether this brush writes colour, height, or both — see [`DrawTo`]. Default
    /// [`DrawTo::ColorAndDepth`] (an ordinary loaded brush).
    pub impasto_draw_to: DrawTo,
    /// **Smoothing**, `0..1`: how much the deposited height is softened before it is lit. `0` = the
    /// raw dab profile (crisp ridges, every grain striation lit); higher settles the paint like a
    /// thick medium relaxing under its own weight. Purely a property of the deposit, not of the light.
    pub impasto_smoothing: f32,
    /// **Body**, `0..1`: the relief's cross-section, between the two schools the state of the art
    /// ships (Photoshop's bevel *Technique*, Smooth ↔ Chisel Hard; Blender's Draw vs Layer brushes —
    /// `docs/Painter/17_impasto_deposito_pesquisa2.md` §3). `1` (default) = a **body of paint**: a
    /// level film with its wall a little inside the paint's edge ([`crate::height::body_profile`]).
    /// `0` = the relief **obeys the silhouette exactly** — falloff, Shape image and Shape-Tone ramp
    /// sculpt the cross-section, so a soft brush lays a perfectly rounded ridge (Enio's smoke,
    /// 2026-07-12: the roundness must stay reachable). Between the two, a mesa with a soft crown —
    /// the Chisel-Soft family.
    pub impasto_body: f32,
    /// **Plow**, `0..1`: how strongly the **Smear** brush drags EXISTING relief along with the colour —
    /// the palette knife (Corel's `Plow`; ArtRage's Flat blade). `0` (default) = the knife moves pigment
    /// only and the paint's body stays where it was; `1` = the body follows the smear in full.
    ///
    /// It displaces; it never deposits — so it is deliberately independent of [`Self::impasto_depth`],
    /// and it does not need the `impasto` master switch either: a stroke can plow relief that some other
    /// brush laid down.
    pub impasto_plow: f32,
    /// **Push**, `0..1`: how much of the paint ALREADY on the canvas this brush shoves aside as it passes
    /// — volume conservation, and the thing that separates paint from a bump map.
    ///
    /// `0` (the default) = the stroke piles onto whatever is under it, as if two bodies of paint could
    /// occupy the same space. `1` = the brush ploughs its footprint clean and every bit of the displaced
    /// material stands up as a **ridge along the edges of the stroke** — nothing is created, nothing is
    /// destroyed, it only moves.
    ///
    /// It acts on the GROUND (the relief that was there before this stroke), never on the stroke's own
    /// deposit: a brush pushes the paint it finds, not the paint it is laying. Derived from the stroke's
    /// footprint rather than its path, so it re-derives live with the rest of the Body card and survives
    /// the shape editors' re-stamp (see `impasto_settle::push_ground`).
    pub impasto_push: f32,

    // ── The paint's MATERIAL (what the light finds when it arrives — see [`crate::material`]) ────────
    //
    // These live on the BRUSH, not on the canvas, because they are properties of the PAINT: a preset
    // called "wet oil" and one called "dry gouache" differ here and nowhere else. They are baked into
    // the canvas with the stroke (like Depth and Body, and unlike the light rig, which is a property of
    // the ROOM and stays canvas-wide and live) — which is what lets one painting hold glossy oil beside
    // matte gouache, the thing a canvas-global Shine could never do.
    /// **Shine**, `0..1` — how much the specular highlight is worth. `0` = matte paint that only models;
    /// up = wet oil. (It USED to be canvas-global, `PaintState::impasto_shine`, while its own doc-comment
    /// called it "a property of the PAINT". Enio caught the smell from the other side, 2026-07-13:
    /// *"Shine parece ser a intensidade do brilho mas é global e não por lâmpada."* It is neither — it is
    /// the paint's, and paint is per pixel.)
    pub impasto_shine: f32,
    /// **Roughness**, `0..1` — how BROAD the highlight is, which is a different question from how strong.
    /// `0` = a tight glint riding the crest (varnish, fresh oil); `1` = a wide soft sheen (chalk, dry
    /// gouache). Default `0.5`, which maps to the exponent the pass had hard-coded as `SHININESS = 24`
    /// before this knob existed ([`crate::material::Material::NEUTRAL`]) — so a default brush is
    /// byte-identical to the pre-material build.
    pub impasto_roughness: f32,
    /// **Metallic**, `0..1` — whose colour the highlight takes. `0` (dielectric: oil, acrylic, gouache) =
    /// the LAMP's, so a white light glints white on red paint. `1` (metal: gold leaf, iridescent) = the
    /// PAINT's own, so gold glints gold.
    pub impasto_metallic: f32,
    /// **Wax**, `0..1` — the waxy, soft-terminator look of paint the light enters and leaves nearby.
    /// Wrap lighting, NOT a subsurface simulation; see [`crate::material::Material::wax`] for why the
    /// honest name is the one that promises the percept it delivers.
    pub impasto_wax: f32,
    /// **Wax Colour** — a FILTER on the light that scatters through the paint. Default WHITE, which IS
    /// the physics (the paint already scatters its own colour, for free). A warm filter over white paint
    /// is alabaster — the case the derived tint alone can never reach. See
    /// [`crate::material::Material::wax_color`].
    pub impasto_wax_color: [f32; 3],
}

impl BrushSpec {
    /// This brush's [`crate::material::Material`] — what its paint IS, as opposed to what shape it
    /// leaves. The one place the four fields are read together, so a knob added later cannot be
    /// forgotten by half the pipeline.
    #[must_use]
    pub fn material(&self) -> crate::material::Material {
        crate::material::Material {
            shine: self.impasto_shine,
            roughness: self.impasto_roughness,
            metallic: self.impasto_metallic,
            wax: self.impasto_wax,
            wax_color: self.impasto_wax_color,
        }
    }
}

impl BrushSpec {
    /// Effective dab radius after clamping to the allocation cap.
    #[must_use]
    pub fn clamped_radius(&self) -> f32 {
        self.radius_px.clamp(0.5, MAX_BRUSH_RADIUS_PX)
    }

    /// The baked dab flatten + rotate ([`crate::footprint::FootprintDeform`]) — applied to the
    /// footprint of the falloff, the Shape silhouette and the View-mapped Grain so they deform together.
    #[must_use]
    pub fn footprint_deform(&self) -> crate::footprint::FootprintDeform {
        crate::footprint::FootprintDeform::new(self.dab_flatten, self.dab_angle_deg)
    }

    /// Whether **Randomize Color** has any non-zero amount (so enabling it would actually change a
    /// dab). Used to gate the RNG draw in [`crate::jitter::per_dab`].
    #[must_use]
    pub fn has_colour_jitter_amount(&self) -> bool {
        self.color_jitter_hue > 0.0 || self.color_jitter_sat > 0.0 || self.color_jitter_val > 0.0
    }

    /// Whether this brush rotates each dab's texture frame independently (Jitter Rotate), so the
    /// constant-orientation stamp caches must be bypassed (each dab needs its own basis). Only
    /// meaningful with a texture that uses per-dab rotation (i.e. not Stencil); Drag Dot / Anchored
    /// opt out of all per-dab jitter, so they never trigger it.
    #[must_use]
    pub fn has_per_dab_rotation(&self) -> bool {
        // Jitter Rotate spins the WHOLE dab footprint (falloff + Shape + View-Grain) about the centre. It
        // only *shows* when the footprint is not rotation-invariant — i.e. an anisotropic footprint
        // (`dab_flatten > 0`) OR a texture that rides the dab frame. This used to require
        // `texture.is_active()`, so a FLATTENED, untextured dab with Jitter Rotate looked constant to the
        // guard and Smear/Blur/Clone served it the cached (constant-orientation) mask: every dab came out
        // with the SAME ellipse angle and the knob did nothing (sweep 2026-07-12). The paint path never had
        // the bug — with both slots off it has no cache to serve and resolves `d.rotation` per pixel.
        self.jitter_rotate > 0.0
            && self.stroke_method.allows_jitter()
            && (self.dab_flatten > 0.0
                || (self.texture.is_active() && self.texture.mapping.uses_dab_rotation()))
    }

    /// Whether the **Shape** slot supplies the silhouette (else the [`Self::falloff`] does). True when
    /// a shape kind is assigned AND — for the `Image` kind — the pixels are present: an Image shape with
    /// no image falls back to the falloff, so the brush never paints a blank silhouette. `has_shape_image`
    /// is whether the tool currently holds Shape pixels (the heavy buffer lives in `PaintState`).
    #[must_use]
    pub fn shape_silhouette_active(&self, has_shape_image: bool) -> bool {
        self.shape.is_active()
            && (self.shape.kind != crate::texture::TextureKind::Image || has_shape_image)
    }

    /// Grain Depth clamped to `[0, 1]`. The per-texel grain value `g` becomes `1 + (g − 1)·depth`, so
    /// `depth = 1` is the historical full-bite behaviour and `depth = 0` disables the grain.
    #[must_use]
    pub fn grain_depth(&self) -> f32 {
        self.grain_depth.clamp(0.0, 1.0)
    }

    /// Effective **Granulation** `[0, 1]` (the watercolor deposit gate on the Grain, [`crate::texture::grain_coverage`]).
    /// Zero unless the Watercolor section is on, so a non-watercolor brush keeps the plain Grain multiply
    /// (byte-identical). Pair with a canvas-anchored Grain (`Tiled` mapping + `Grain` kind) for paper granulation.
    #[must_use]
    pub fn effective_granulation(&self) -> f32 {
        if self.watercolor {
            self.granulation.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Effective **Pigment mix** `[0, 1]` (how much the dab composites subtractively — RYB, Gossett &
    /// Chen 2004 — vs the plain blend; [`crate::blend::blend_over_pigment`]). Zero unless BOTH the
    /// Watercolor section and the Pigment toggle are on, so a normal brush blends exactly as before
    /// (byte-identical). Non-zero ⇒ wet-on-wet mixes like real paint (blue + yellow → green).
    #[must_use]
    pub fn effective_pigment_mix(&self) -> f32 {
        if self.watercolor && self.pigment {
            self.pigment_mix.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Whether this brush's dabs deposit **height** (the impasto relief). The master switch must be on,
    /// the brush must be set to write depth, and the depth must be non-zero — a zero-depth dab is a
    /// no-op *by definition* (it neither lifts nor carves), so the height pass skips it entirely rather
    /// than writing a flat zero over relief that is already there.
    #[must_use]
    pub fn deposits_height(&self) -> bool {
        self.impasto && self.impasto_draw_to.writes_depth() && self.impasto_depth != 0.0
    }

    /// Whether this brush's dabs touch the height field **at all** — it lays body down, or it shoves
    /// existing body around, or both.
    ///
    /// Not the same question as [`Self::deposits_height`], and the difference is a real brush: at
    /// **Depth 0 with Push up** the brush carries no paint and still moves the paint it finds. That is a
    /// dry brush, and it is a palette knife — the tool that does nothing BUT displace. Gating the height
    /// pass on the deposit alone made that brush a no-op, and it is the most physical use of Push there is.
    #[must_use]
    pub fn touches_height(&self) -> bool {
        self.deposits_height() || (self.impasto && self.effective_impasto_push() > 0.0)
    }

    /// Whether this brush's dabs deposit **pigment**. Only [`DrawTo::Depth`] — *with the master switch
    /// on* — suppresses it. With impasto off, [`Self::impasto_draw_to`] is not read at all, so a brush
    /// left on "Depth" in a previous session paints normally again the moment impasto is unticked; the
    /// master switch is the single gate, which is what makes the off state byte-identical.
    #[must_use]
    pub fn deposits_color(&self) -> bool {
        !self.impasto || self.impasto_draw_to.writes_color()
    }

    /// Effective impasto **Depth**, clamped to `[-1, 1]`. Zero unless the master switch is on, so every
    /// caller can read this one number without re-checking the gate.
    #[must_use]
    pub fn effective_impasto_depth(&self) -> f32 {
        if self.impasto {
            self.impasto_depth.clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }

    /// Effective impasto **Smoothing** `[0, 1]`. Zero unless the master switch is on.
    #[must_use]
    pub fn effective_impasto_smoothing(&self) -> f32 {
        if self.impasto {
            self.impasto_smoothing.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Whether this brush's Smear should drag relief: a non-zero **Plow**. Independent of the `impasto`
    /// master switch — the knife moves paint that is already down, whoever laid it.
    #[must_use]
    pub fn impasto_plow_active(&self) -> bool {
        self.impasto_plow > 0.0
    }

    /// Effective impasto **Plow** `[0, 1]`.
    #[must_use]
    pub fn effective_impasto_plow(&self) -> f32 {
        self.impasto_plow.clamp(0.0, 1.0)
    }

    /// Effective impasto **Body** `[0, 1]` — how far the deposit is pushed through the body curve
    /// (`1` = plateau + wall; `0` = the silhouette's own cross-section). No master-switch gate: with
    /// impasto off the kernel is never reached, and the value has no other reader.
    #[must_use]
    pub fn effective_impasto_body(&self) -> f32 {
        self.impasto_body.clamp(0.0, 1.0)
    }

    /// [`Self::impasto_push`], clamped — how much of the ground this brush shoves aside.
    #[must_use]
    pub fn effective_impasto_push(&self) -> f32 {
        self.impasto_push.clamp(0.0, 1.0)
    }

    /// Compose the dab silhouette from a Shape sample `shape_val` and the round `falloff` envelope. The
    /// **Image** kind REPLACES the falloff (a crisp finite tip stays uneroded); any **procedural** kind is
    /// MASKED BY it (`falloff × pattern`, so the soft round envelope shapes the texture — Enio 2026-06-25).
    /// `None` never reaches here (the caller uses the bare falloff when the Shape is inactive). The single
    /// source for all three stamp paths (per-pixel, scale-invariant bake, canvas-cached) so they agree.
    #[must_use]
    pub fn compose_shape_silhouette(&self, shape_val: f32, falloff: f32) -> f32 {
        crate::texture::compose_shape_silhouette_kind(self.shape.kind, shape_val, falloff)
    }

    /// Whether the **Shape** slot rotates its silhouette frame per dab — Rake (follows the stroke) OR the
    /// Stroke **Jitter Rotate** (which spins the WHOLE dab stamp, Shape + Grain together) — so the
    /// constant-orientation caches can't apply (each dab needs its own Shape basis). Only meaningful when
    /// the Shape is the active silhouette. Mirrors [`Self::has_per_dab_rotation`].
    #[must_use]
    pub fn shape_has_per_dab_rotation(&self, has_shape_image: bool) -> bool {
        self.shape_silhouette_active(has_shape_image)
            && self.shape.mapping.uses_dab_rotation()
            && (self.shape.rake
                || self.shape.flow
                || (self.jitter_rotate > 0.0 && self.stroke_method.allows_jitter()))
    }

    /// Whether the **Grain** slot rotates its texture frame per dab (Rake follows the stroke) — each dab
    /// needs its own Grain basis. Mirrors [`Self::shape_has_per_dab_rotation`] for the Grain; complements
    /// [`Self::has_per_dab_rotation`] (which is the Grain **Jitter Rotate**).
    #[must_use]
    pub fn grain_has_per_dab_rotation(&self) -> bool {
        self.texture.is_active() && self.texture.rake && self.texture.mapping.uses_dab_rotation()
    }

    /// Whether the dab is eligible for the **scale-invariant cached stamp** ([`crate::stamp`]) given
    /// both slots: the silhouette must be dab-relative-constant (the falloff always is; a Shape image is
    /// when View-static) AND the Grain must be cacheable (None or static View). `has_shape_image` gates
    /// whether an `Image` shape counts as active. Mirrors [`crate::TextureSettings::is_cacheable`] for
    /// the Grain, extended with the Shape. (Per-dab rotation / Accumulate gating stays at the call site.)
    #[must_use]
    pub fn dab_mask_cacheable(&self, has_shape_image: bool) -> bool {
        let shape_static = !self.shape_silhouette_active(has_shape_image)
            || (matches!(
                self.shape.mapping,
                crate::texture::TextureMapping::ViewPlane
            ) && !self.shape.rake
                && !self.shape.flow);
        shape_static && self.texture.is_cacheable()
    }

    /// Distance between dab centres in pixels, derived from spacing × diameter.
    /// At least one pixel so a stroke always advances.
    #[must_use]
    pub fn dab_spacing_px(&self) -> f32 {
        (self.spacing.max(0.01) * 2.0 * self.clamped_radius()).max(1.0)
    }

    /// Whether the dab at dash-slot `slot` is painted, given the dash pattern.
    ///
    /// Behavioural reference: `paint_stroke.cc::add_step` (`dash = (slot % dash_samples) /
    /// dash_samples`; skip when `dash > dash_ratio`). `dash_samples == 0` ⇒ no dash (always on).
    /// With the default `dash_ratio = 1.0` every slot is on (solid).
    #[must_use]
    pub fn dash_on(&self, slot: u32) -> bool {
        if self.dash_samples == 0 {
            return true;
        }
        let dash = (slot % self.dash_samples) as f32 / self.dash_samples as f32;
        dash <= self.dash_ratio.clamp(0.0, 1.0)
    }

    /// "Adjust Strength for Spacing" multiplier applied to each dab's coverage, in `(0, 1]`.
    ///
    /// Behavioural reference (clean-room): `paint_stroke.cc::paint_stroke_integrate_overlap` +
    /// `paint_stroke_overlapped_curve`. Models how many neighbouring dab falloff kernels stack at a
    /// given phase and returns `1 / max_phase(Σ kernels)` so a densely-spaced stroke is normalised to
    /// unit opacity instead of piling up. Returns `1.0` (no attenuation) when the flag is off or
    /// spacing ≥ 100% (Blender's exact gate). Uses this brush's own falloff, like Blender.
    #[must_use]
    pub fn space_overlap_factor(&self) -> f32 {
        // Blender stores spacing as an integer percent; this engine stores a 0..1 fraction.
        let spacing_pct = (self.spacing * 100.0).max(0.0);
        if !(self.space_attenuation && spacing_pct < 100.0) {
            return 1.0;
        }
        // Sample the overlap sum at M phases across one period; the factor cancels the peak.
        const M: usize = 10;
        let g = 1.0 / M as f32;
        let mut max = 0.0_f32;
        for i in 0..M {
            let o = self.overlapped_curve(i as f32 * g, spacing_pct).abs();
            if o > max {
                max = o;
            }
        }
        if max == 0.0 { 1.0 } else { 1.0 / max }
    }

    /// Sum of overlapping dab falloff kernels at phase `x` for a given `spacing_pct`
    /// (`paint_stroke_overlapped_curve`): `n = floor(100 / spacing_pct)` kernels spaced
    /// `h = spacing_pct / 50` apart, each evaluated through this brush's falloff.
    fn overlapped_curve(&self, x: f32, spacing_pct: f32) -> f32 {
        let clamped = spacing_pct.max(0.1);
        let n = (100.0 / clamped) as i32;
        let h = clamped / 50.0;
        let x0 = x - 1.0;
        let mut sum = 0.0;
        for i in 0..n {
            let xx = (x0 + i as f32 * h).abs();
            if xx < 1.0 {
                sum += self.falloff_weight(xx);
            }
        }
        sum
    }

    /// Falloff weight remapped by [`Self::hardness`]. `t = distance / radius`.
    ///
    /// Hardness pushes the falloff outward: for `t < hardness` the weight is full; the curve then
    /// runs over `[hardness, 1]`. `hardness >= 1` yields a hard disk. [`Falloff::Custom`] reads the
    /// editable [`Self::custom_falloff`] profile; every other preset uses its formula.
    #[must_use]
    pub fn falloff_weight(&self, t: f32) -> f32 {
        let h = self.hardness.clamp(0.0, 1.0);
        if h >= 1.0 {
            return if t < 1.0 { 1.0 } else { 0.0 };
        }
        let remapped = ((t - h) / (1.0 - h)).clamp(0.0, 1.0);
        match self.falloff {
            Falloff::Custom => self.custom_falloff.weight(remapped),
            preset => preset.weight(remapped),
        }
    }
}
