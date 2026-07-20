//! Brush **texture** — a per-texel mask that modulates the dab's coverage.
//!
//! Clean-room model of Blender's brush texture (`editors/sculpt_paint/mesh/paint_image_2d.cc`,
//! `brush_painter_2d_tex_mapping` + `BKE_brush_sample_tex_3d`), **adapted to 2D**: the `3D` mapping
//! mode and the `Z` axis are dropped (degenerate for raster paint). The texture multiplies the
//! falloff mask per pixel, exactly as the falloff weight does — see [`crate::dab::stamp_dab`].
//!
//! **Where the rotations live.** A slot frame ([`dab_basis`]) carries the slot's **Angle** and nothing
//! else. "Follow the stroke" (Rake / Flow) and the per-dab **Jitter Rotate** orient the DAB — one rotor,
//! built once in [`crate::BrushSpec::dab_rotor`] and applied to the footprint every sampler reads. That is
//! Blender's shape too (a single `brush_rotation`, applied once); ours additionally has an elliptical
//! footprint, which Blender's texture paint does not, so the rotor lands on the frame instead of the
//! lookup. Splitting it per slot is what let a flattened tip and the pattern inside it disagree on a curve.
//!
//! **Determinism (HR-5).** Transcendental-free: rotation is carried as a unit *vector*, never an
//! angle (as in `stroke/ellipse.rs` / `stroke/polygon.rs`). **Angle** rotates `(1,0)` by repeated
//! application of the baked 1° step [`DEG_STEP`]. The sampler uses only `floor`/`*`/`+`/`sqrt`.

mod kind;
pub(crate) mod patterns;
mod shape;
mod stencil;
mod tiled;
pub use kind::TextureKind;
pub use shape::{
    ImageRgb, compose_shape_silhouette_kind, remap_shape_value, render_shape_preview,
    sample_shape_rgb_unit, sample_shape_silhouette, sample_shape_silhouette_unit,
};
pub use stencil::{render_stencil_preview, stencil_frame, stencil_gate};
pub use tiled::{angle_basis, sample_tiled, sample_tiled_rot, sample_tiled_rot_wrapped};

/// Largest **Angle** the slider reaches, in whole degrees (one full turn).
pub const TEX_ANGLE_MAX_DEG: u16 = 360;
/// **Offset** range, in tile fractions (one unit = one full tile shift). Symmetric about 0.
pub const TEX_OFFSET_MIN: f32 = -1.0;
/// See [`TEX_OFFSET_MIN`].
pub const TEX_OFFSET_MAX: f32 = 1.0;
/// **Size** (scale) range. `1.0` = one tile per footprint (View) / per [`TEX_TILE_BASE_PX`] (Tiled).
pub const TEX_SIZE_MIN: f32 = 0.1;
/// See [`TEX_SIZE_MIN`]. Wide (`100×`) so a paper/grain can tile very fine or very coarse (Enio 2026-07-05).
pub const TEX_SIZE_MAX: f32 = 100.0;
/// Canvas pixels spanned by one texture tile at Size `1.0` under the **Tiled** mapping.
pub const TEX_TILE_BASE_PX: f32 = 256.0;

/// Baked unit-vector step for a **1° rotation**, `(cos 1°, sin 1°)`. Rotating by `n°` applies this
/// `n` times — only `*`/`+` at runtime, bit-identical on every platform (mirrors
/// `stroke/polygon.rs::POLY_STEP`); drift over ≤360 steps is deterministic + sub-`5e-5`.
pub const DEG_STEP: [f32; 2] = [0.999_847_7, 0.017_452_406];

/// How texture coordinates are derived from the dab. `3D` is dropped (2D adaptation); `Stencil` is a
/// later phase (needs a screen-space overlay), so P1 exposes View/Tiled/Random.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextureMapping {
    /// Coordinates relative to the dab footprint, centred on the cursor — the texture *follows the brush* (the 2D-paint default).
    #[default]
    ViewPlane,
    /// Coordinates from the canvas position — the texture is *fixed to the image* while you paint over it.
    Tiled,
    /// Like [`Self::ViewPlane`] but with a random per-dab offset.
    Random,
    /// A positioned/rotated/scaled rectangular **stencil** you paint *through*: the texture fills the rect
    /// once and masks outside it. 2D-adapted to image space, driven by Offset/Size/Angle. See [`dab_basis`].
    Stencil,
}

impl TextureMapping {
    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::ViewPlane => 0,
            Self::Tiled => 1,
            Self::Random => 2,
            Self::Stencil => 3,
        }
    }

    /// Inverse of [`Self::to_u8`]; unknown values fall back to [`Self::ViewPlane`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Tiled,
            2 => Self::Random,
            3 => Self::Stencil,
            _ => Self::ViewPlane,
        }
    }

    /// Number of selectable mappings (drives the dropdown decode range).
    pub const COUNT: u8 = 4;

    /// English label for the dropdown.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::ViewPlane => "View Plane",
            Self::Tiled => "Tiled",
            Self::Random => "Random Offset",
            Self::Stencil => "Stencil",
        }
    }

    /// Whether this mapping randomises the per-dab translation (only [`Self::Random`]).
    #[must_use]
    pub fn randomises_offset(self) -> bool {
        matches!(self, Self::Random)
    }

    /// Whether the per-dab Rake rotation applies (Stencil has its own fixed frame → ignores it).
    #[must_use]
    pub fn uses_dab_rotation(self) -> bool {
        !matches!(self, Self::Stencil)
    }

    /// Whether this is the [`Self::Stencil`] mapping (image-space positioned mask).
    #[must_use]
    pub fn is_stencil(self) -> bool {
        matches!(self, Self::Stencil)
    }

    /// Canvas-fixed mappings (Tiled / Stencil): sample at the canvas pixel ([`sample`], which masks the
    /// Stencil rect), never the dab-local [`sample_unit`] (no rect → leaks past it).
    #[must_use]
    pub fn is_canvas_fixed(self) -> bool {
        matches!(self, Self::Tiled | Self::Stencil)
    }
}

/// Brush-texture parameters. Cheap to copy (no pixels) so [`crate::BrushSpec`] stays `Copy`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextureSettings {
    /// Which procedural pattern (or `None`).
    pub kind: TextureKind,
    /// How texture coordinates are derived.
    pub mapping: TextureMapping,
    /// Base rotation in whole degrees, `0..=`[`TEX_ANGLE_MAX_DEG`].
    pub angle_deg: u16,
    /// **Rake**: the DAB FRAME follows the stroke direction, with [`Self::angle_deg`] composed as an
    /// offset within it. The flag is a request — the rotation itself is applied once, for the whole dab,
    /// by [`crate::BrushSpec::dab_rotor`]; this slot's own frame never carries the tangent.
    pub rake: bool,
    /// **Flow** (Shape slot): lay the pattern in the STROKE's own frame — the *along* coordinate is the
    /// dab's [`crate::Dab::arc_len`] plus the pixel's projection on the tangent, the *across* coordinate is
    /// the perpendicular. This keeps the pattern's phase continuous from dab to dab, so the silhouette's
    /// lines stay parallel and follow the curve (calligraphy / textured-stroke), instead of the per-stamp
    /// Rake that resets phase each dab and interferes on curves. Like [`Self::rake`] it asks the DAB FRAME
    /// to follow the stroke (same single rotor); what it adds is the arc-length, so the phase is continuous
    /// across dabs. The two are mutually exclusive through one door (the Follow selector). Only the Shape
    /// slot exposes it; Grain / Paper leave it `false`. See [`crate::texture::shape`].
    pub flow: bool,
    /// Translation in tile fractions, each component in `[`[`TEX_OFFSET_MIN`]`, `[`TEX_OFFSET_MAX`]`]`.
    pub offset: [f32; 2],
    /// Per-axis scale, each in `[`[`TEX_SIZE_MIN`]`, `[`TEX_SIZE_MAX`]`]` (`1.0` = one tile).
    pub size: [f32; 2],
    /// **Stencil** rect centre, per axis in `[`[`TEX_OFFSET_MIN`]`, `[`TEX_OFFSET_MAX`]`]` (`0` =
    /// canvas centre). Independent of [`Self::offset`] so the gizmo placement and the texture tiling
    /// don't fight over one field — used only by the [`TextureMapping::Stencil`] mapping.
    pub stencil_offset: [f32; 2],
    /// **Stencil** rect half-extent as a canvas fraction, per axis in `[`[`TEX_SIZE_MIN`]`,
    /// `[`TEX_SIZE_MAX`]`]` (default `0.5` = the rect is 50 % of the sprite). Independent of
    /// [`Self::size`]; used only by the [`TextureMapping::Stencil`] mapping.
    pub stencil_size: [f32; 2],
    /// **Stencil** rect rotation in whole degrees, `0..=`[`TEX_ANGLE_MAX_DEG`]. Independent of
    /// [`Self::angle_deg`]; used only by the [`TextureMapping::Stencil`] mapping.
    pub stencil_angle_deg: u16,
    /// Per-pattern shape knobs, each **normalized** `[0, 1]`; meaning is per-[`TextureKind`] (see
    /// [`param_specs`]). Slots `0`/`1` are the universal **Contrast** / **Brightness** (`0.5` =
    /// neutral); slot `2` is the kind's shape param (Detail / Turbulence / Radius / …). `0.5`
    /// throughout is the neutral default, reset to each kind's [`param_specs`] defaults on a change.
    pub params: [f32; MAX_TEX_PARAMS],
}

/// Per-pattern parameter slots in [`TextureSettings::params`] (`0`/`1` = Contrast / Brightness; `2..` = shape knobs).
pub const MAX_TEX_PARAMS: usize = 6;

impl Default for TextureSettings {
    fn default() -> Self {
        Self {
            kind: TextureKind::None,
            mapping: TextureMapping::ViewPlane,
            angle_deg: 0,
            rake: false,
            flow: false,
            offset: [0.0, 0.0],
            size: [1.0, 1.0],
            stencil_offset: [0.0, 0.0],
            stencil_size: [0.5, 0.5],
            stencil_angle_deg: 0,
            params: [0.5; MAX_TEX_PARAMS],
        }
    }
}

pub use patterns::{
    ParamSpec, analytic_needs_hash_wrap, analytic_tile_period, lattice_tileable, param_specs,
};

impl TextureSettings {
    /// Whether the texture actually modulates anything (a kind is assigned).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.kind != TextureKind::None
    }

    /// Stamp is dab-relative + constant across a stroke → render once into a mask + scale-blit
    /// (Blender's brush-image cache; see [`crate::stamp`]). True with no texture or a static **View**
    /// texture; Rake (per-dab rotation) and Tiled / Stencil (canvas-relative) stay per-pixel.
    #[must_use]
    pub fn is_cacheable(&self) -> bool {
        !self.is_active()
            || (matches!(self.mapping, TextureMapping::ViewPlane) && !self.rake && !self.flow)
    }

    /// Texture is canvas-fixed + dab-independent → cache each canvas pixel once per stroke
    /// ([`crate::stamp::blit_canvas_cached`]). Static **Tiled** / **Stencil**; Rake stays per-pixel.
    #[must_use]
    pub fn is_canvas_cacheable(&self) -> bool {
        self.is_active()
            && matches!(
                self.mapping,
                TextureMapping::Tiled | TextureMapping::Stencil
            )
            && !self.rake
            && !self.flow
    }
}

/// A borrowed grayscale (luminance) image supplied to [`sample`] for [`TextureKind::Image`]. One byte per
/// texel, row-major; sampled bilinearly (centre-coord) + tiled (`fract`) so it composes with every mapping.
/// Kept borrowed so the heavy pixels stay owned by the caller (the `Copy` settings can't hold them).
#[derive(Clone, Copy, Debug)]
pub struct ImageMask<'a> {
    /// Luminance bytes, row-major, at least `width * height` long.
    pub lum: &'a [u8],
    /// Image width in texels.
    pub width: u32,
    /// Image height in texels.
    pub height: u32,
}

/// Per-dab resolved texture frame: the pattern's rotated unit basis (`u`, `v`), the per-dab random
/// translation, and — for Stencil — the rect (centre / half-extent / own rotation `stencil_u`,`v`; `u`/`v`
/// then carry the texture angle within it). Computed once per dab by [`dab_basis`] so [`sample`] is cheap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TexDabBasis {
    u: [f32; 2],
    v: [f32; 2],
    jitter: [f32; 2],
    /// Stencil rect centre in canvas px (unused unless the mapping is Stencil).
    stencil_center: [f32; 2],
    /// Stencil rect half-extent in canvas px, per axis (unused unless Stencil).
    stencil_half: [f32; 2],
    /// Stencil rect rotation basis (from `stencil_angle_deg`) for the mask; unused unless Stencil.
    stencil_u: [f32; 2],
    stencil_v: [f32; 2],
    /// Brush-dab flatten + rotate, applied to the footprint coord BEFORE this texture's own Size /
    /// rotation / Offset (so the Shape + View-Grain deform with the falloff). Identity for Tiled / Stencil.
    footprint: crate::footprint::FootprintDeform,
    /// The stroke frame the **Flow** mapping lays its pattern in — see [`ShapeFrame`]. Only the Shape
    /// door ([`shape_basis`]) can supply it, so it can never be silently omitted; ignored unless `flow`.
    frame: ShapeFrame,
}

/// The stroke facts the Shape **Flow** mapping needs to lay a pattern in the STROKE's frame rather than
/// the dab's. There is no `Default` and no builder **on purpose**: Flow's whole promise is that the phase
/// is continuous from dab to dab, and a caller that forgets to supply the arc-length gets a pattern that
/// resets at every stamp — the exact artefact Flow exists to remove, failing silently with the dropdown
/// still reading "Flow". So [`shape_basis`] takes it by value and every Shape route must answer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeFrame {
    /// A live dab on a stroke: its [`crate::Dab::arc_len`] and the stroke's nominal dab radius
    /// ([`crate::Dab::stroke_radius_px`]).
    ///
    /// ⚠️ `unit_px` **must be constant for the whole stroke.** The along-coordinate is
    /// `(arc_len + projection) / unit_px`: the numerator is the pixel's absolute position along the path
    /// and telescopes exactly between neighbours, so a *per-dab* divisor (the live, pressure-scaled
    /// radius) re-phases the whole accumulated history by `arc_len · Δ(1/r)` — an error that grows the
    /// further into the stroke you are. Measured with the shipped `size_pressure` default: 0.42 tile
    /// units of jump between adjacent dabs, ~21 % of a Stripes period. Blender takes the same care for
    /// its Tiled mapping, normalising by `start_pixel_radius` "so the tiling doesn't breathe with pressure".
    Stroke {
        /// Cumulative path length at the dab (px).
        arc_len: f32,
        /// The stroke-constant length one pattern tile spans (px) — the brush's nominal dab radius.
        unit_px: f32,
    },
    /// No stroke: the panel preview and the scale-invariant cached stamp bake. Flow is inert here (it is
    /// never cached — [`TextureSettings::is_cacheable`] refuses it — and a preview has no path).
    Static,
}

impl ShapeFrame {
    /// `(arc_len, unit_px)`, with a safe unit for the `Static` frame (Flow is inert there).
    #[must_use]
    fn parts(self) -> (f32, f32) {
        match self {
            Self::Stroke { arc_len, unit_px } => (arc_len, unit_px.max(1e-3)),
            Self::Static => (0.0, 1.0),
        }
    }
}

impl TexDabBasis {
    /// The identity frame (no rotation, no jitter) — used when there is no texture.
    #[must_use]
    pub fn identity() -> Self {
        Self {
            u: [1.0, 0.0],
            v: [0.0, 1.0],
            jitter: [0.0, 0.0],
            stencil_center: [0.0, 0.0],
            stencil_half: [1.0, 1.0],
            stencil_u: [1.0, 0.0],
            stencil_v: [0.0, 1.0],
            footprint: crate::footprint::FootprintDeform::identity(),
            frame: ShapeFrame::Static,
        }
    }
}

/// Resolve the per-dab texture frame: the slot's own **Angle** rotation, a splitmix64 `rng` (Random
/// **Offset** mapping), the canvas (Stencil rect) and the dab `footprint` (Tiled / Stencil ignore it).
///
/// ⚠️ **The stroke tangent does not enter here.** "Follow the stroke" (Rake / Flow) orients the DAB FRAME
/// once, in [`crate::BrushSpec::dab_footprint`], and reaches every sampler through the `footprint` — one
/// rotation, one place, which is also Blender's model (a single `brush_rotation`, applied to the lookup,
/// never twice). Applying it here as well would rotate a following tip by twice the tangent.
#[must_use]
pub fn dab_basis(
    s: &TextureSettings,
    rng: &mut u64,
    canvas: [f32; 2],
    footprint: crate::footprint::FootprintDeform,
) -> TexDabBasis {
    // Stencil = a fixed image-space rect (canvas-fixed, so no Rake/Random/Jitter and no dab flatten).
    if s.mapping.is_stencil() {
        return stencil::stencil_basis(s, canvas);
    }
    let u = rotate_by_degrees(s.angle_deg);
    let v = perp(u);
    let jitter = if s.mapping.randomises_offset() {
        // A full-tile random shift per dab, in tile fractions.
        [crate::jitter::next_f32(rng), crate::jitter::next_f32(rng)]
    } else {
        [0.0, 0.0]
    };
    TexDabBasis {
        u,
        v,
        jitter,
        stencil_center: [0.0, 0.0],
        stencil_half: [1.0, 1.0],
        stencil_u: [1.0, 0.0],
        stencil_v: [0.0, 1.0],
        footprint,
        frame: ShapeFrame::Static,
    }
}

/// The **Shape** slot's door onto [`dab_basis`]: identical, plus the [`ShapeFrame`] the **Flow** mapping
/// needs. Separate from [`dab_basis`] so the stroke facts are a *parameter*, not a builder a caller can
/// forget — five Shape routes (relief, sculpt, smear, watercolor, blur/clone) once did exactly that, and
/// the failure is silent: Flow degrades to a per-dab phase reset with the dropdown still reading "Flow".
/// Arch-gated (`the_shape_slot_goes_through_the_shape_door`) so a new Shape route cannot use the Grain one.
#[must_use]
pub fn shape_basis(
    s: &TextureSettings,
    rng: &mut u64,
    canvas: [f32; 2],
    footprint: crate::footprint::FootprintDeform,
    frame: ShapeFrame,
) -> TexDabBasis {
    let mut b = dab_basis(s, rng, canvas, footprint);
    b.frame = frame;
    b
}

/// Combine a Grain `sample` (`[0, 1]`, the paper tooth) into a dab-coverage multiplier under the
/// Grain **Depth** and the watercolor **Granulation** gate. The single source shared by every stamp
/// path (per-pixel + the two constant-orientation / canvas-fixed caches) so they never diverge.
///
/// - **Depth** (`0..1`, Procreate): `base = 1 + (sample − 1)·depth` — how much the grain bites.
/// - **Granulation** (`0..1`, watercolor): scale `base` by the valley gate
///   [`granulation_gate`]`(sample) = 1 − (1 − sample)·granulation` — pigment settles on the tooth peaks
///   and is rejected in the valleys (Curtis 1997 §4.5 / the PH2D Wet Paint spec §10), turning the soft
///   multiply into a harder speckle. The multiplicative form composes with the coloured stamp's ramp.
///
/// `granulation == 0` returns `base` **exactly** (byte-identical), so a non-watercolor brush is
/// unchanged. Deterministic (mul only) — HR-5 safe. Both factors lie in `[0, 1]` so no clamp is needed.
#[must_use]
pub fn grain_coverage(sample: f32, depth: f32, granulation: f32) -> f32 {
    let base = if depth >= 1.0 {
        sample
    } else {
        1.0 + (sample - 1.0) * depth
    };
    base * granulation_gate(sample, granulation)
}

/// The watercolor **valley gate** for a Grain `sample`: `1` at a tooth peak (`sample = 1`), down to
/// `1 − granulation` in a valley (`sample = 0`). Rejects pigment in the paper's low areas → granulation
/// speckle. `granulation == 0` ⇒ `1` (no effect). Result ∈ `[1 − granulation, 1] ⊂ [0, 1]`.
#[must_use]
pub fn granulation_gate(sample: f32, granulation: f32) -> f32 {
    if granulation <= 0.0 {
        1.0
    } else {
        1.0 - (1.0 - sample.clamp(0.0, 1.0)) * granulation
    }
}

/// Sample the texture at canvas pixel `(px, py)` for a dab centred at `center` with `radius`. Returns the
/// coverage multiplier in `[0, 1]`; `1.0` when no texture is assigned (so the dab is unchanged).
#[must_use]
pub fn sample(
    s: &TextureSettings,
    b: &TexDabBasis,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
    image: Option<&ImageMask>,
) -> f32 {
    if !s.is_active() {
        return 1.0;
    }
    // Pixel centre, in canvas pixels.
    let p = [px as f32 + 0.5, py as f32 + 0.5];
    // Texture coordinates, by mapping. Stencil is special: it masks (deposits nothing) outside its
    // rect, and maps the rect onto the procedural's tile window — adjusted by the texture's own
    // Size/Offset/Angle so the pattern inside the rect is tunable independently of the gizmo.
    let tex = if s.mapping.is_stencil() {
        match stencil::stencil_tex_coord(s, b, p) {
            Some(t) => t,
            None => return 0.0, // outside the stencil → paint nothing
        }
    } else if s.flow {
        // FLOW (Shape slot): lay the pattern in the STROKE's own frame so its phase is CONTINUOUS from
        // dab to dab — the lines stay parallel through a curve instead of the per-stamp Rake resetting the
        // phase at every stamp and interfering.
        //
        // The frame is the dab FOOTPRINT, which already carries the stroke tangent
        // ([`crate::BrushSpec::dab_footprint`]), so its `+x` IS the along-stroke axis and the flatten
        // deforms the tip exactly as it deforms the falloff. `arc_len + along` is then the pixel's
        // ABSOLUTE position along the path, which telescopes exactly between neighbouring dabs — and
        // dividing it by the STROKE-CONSTANT `unit` is what keeps it that way (see [`ShapeFrame::Stroke`]:
        // a per-dab radius re-phases the whole accumulated history on every pressure wobble). The slot
        // **Angle** (`u`/`v`) rotates the pattern WITHIN the stroke frame, and Size scales AFTER the
        // rotation (Blender's order — a non-uniform Size applied first shears instead of rotating).
        let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        let (arc_len, unit) = b.frame.parts();
        let f = b
            .footprint
            .apply([(p[0] - center[0]) / unit, (p[1] - center[1]) / unit]);
        let rel = [arc_len / unit + f[0], f[1]];
        [
            (rel[0] * b.u[0] + rel[1] * b.u[1]) * sx + s.offset[0],
            (rel[0] * b.v[0] + rel[1] * b.v[1]) * sy + s.offset[1],
        ]
    } else {
        // Per-axis scale clamped away from zero. Size MULTIPLIES the coordinate (Blender's MTex
        // `texvec = size · co`): a LARGER Size scales coords up → the pattern reads SMALLER / denser.
        let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        // Raw coordinates before rotation, in tile units (footprint-relative or canvas-tiled).
        let rel = match s.mapping {
            TextureMapping::Tiled => {
                let base = TEX_TILE_BASE_PX;
                [p[0] / base, p[1] / base]
            }
            // View / Random anchor to the dab footprint; the dab flatten/rotate deforms it FIRST so
            // the pattern flattens with the falloff (the texture's own Size stays relative to it).
            _ => {
                let r = radius.max(1e-3);
                b.footprint
                    .apply([(p[0] - center[0]) / r, (p[1] - center[1]) / r])
            }
        };
        // ⚠️ **Size scales AFTER the rotation**, i.e. along the PATTERN's own axes — Blender's order
        // (`BKE_brush_sample_tex_3d` rotates the normalised coordinate, then `RE_texture_evaluate` applies
        // `mtex->size`). Scaling first stretches along the CANVAS axes, so a non-uniform Size + a non-zero
        // Angle SHEARED the pattern instead of rotating a stretched one. At Angle 0 the rotation is the
        // identity, so both orders are bit-identical for every Size; only rotated textures move.
        [
            (rel[0] * b.u[0] + rel[1] * b.u[1]) * sx + s.offset[0] + b.jitter[0],
            (rel[0] * b.v[0] + rel[1] * b.v[1]) * sy + s.offset[1] + b.jitter[1],
        ]
    };
    patterns::sample_kind(s.kind, tex, s.params, image).clamp(0.0, 1.0)
}

/// Sample the **View-mapped** texture at the dab-relative unit coord `(u, v) ∈ [-1, 1]` — the
/// scale-invariant form baking the cached brush stamp ([`crate::stamp`]): depends only on
/// `(u, v)·size` + rotation, never the radius. View-only; returns the coverage in `[0, 1]`.
#[must_use]
pub fn sample_unit(
    s: &TextureSettings,
    b: &TexDabBasis,
    u: f32,
    v: f32,
    image: Option<&ImageMask>,
) -> f32 {
    if !s.is_active() {
        return 1.0;
    }
    let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let rel = b.footprint.apply([u, v]); // dab flatten/rotate, before this texture's own Size/rotation
    // ⚠️ **Size scales AFTER the rotation**, i.e. along the PATTERN's own axes — Blender's order
    // (`BKE_brush_sample_tex_3d` rotates the normalised coordinate, then `RE_texture_evaluate` applies
    // `mtex->size`). Scaling first stretches along the CANVAS axes, so a non-uniform Size + a non-zero
    // Angle SHEARED the pattern instead of rotating a stretched one. At Angle 0 the rotation is the
    // identity, so both orders are bit-identical for every Size; only rotated textures move.
    let tex = [
        (rel[0] * b.u[0] + rel[1] * b.u[1]) * sx + s.offset[0],
        (rel[0] * b.v[0] + rel[1] * b.v[1]) * sy + s.offset[1],
    ];
    patterns::sample_kind(s.kind, tex, s.params, image).clamp(0.0, 1.0)
}

// ── Rotation / RNG helpers (transcendental-free) ────────────────────────────────────────────

/// Rotate the unit vector `(1, 0)` by `deg` whole degrees via repeated [`DEG_STEP`] application.
/// Transcendental-free (HR-5): the baked 1-degree step, applied `deg` times. Shared by [`crate::jitter`]
/// (the per-dab Jitter Rotate vector) and by the Impasto light pass (the light direction) — one rotor,
/// so every angle in the painter is built the same deterministic way.
pub fn rotate_by_degrees(deg: u16) -> [f32; 2] {
    let d = deg % 360;
    let (cs, sn) = (DEG_STEP[0], DEG_STEP[1]);
    let (mut x, mut y) = (1.0_f32, 0.0_f32);
    for _ in 0..d {
        let nx = x * cs - y * sn;
        let ny = x * sn + y * cs;
        x = nx;
        y = ny;
    }
    [x, y]
}

/// Left-perpendicular of a unit vector (90° rotation): `(x, y) → (-y, x)`.
fn perp(u: [f32; 2]) -> [f32; 2] {
    [-u[1], u[0]]
}

/// Normalise `v`, or return `fallback` if `v` is near-zero.
pub(crate) fn normalize_or(v: [f32; 2], fallback: [f32; 2]) -> [f32; 2] {
    let len2 = v[0] * v[0] + v[1] * v[1];
    if len2 > 1e-12 {
        let inv = 1.0 / len2.sqrt();
        [v[0] * inv, v[1] * inv]
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests;
