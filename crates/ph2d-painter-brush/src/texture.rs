//! Brush **texture** — a per-texel mask that modulates the dab's coverage.
//!
//! Clean-room model of Blender's brush texture (`editors/sculpt_paint/mesh/paint_image_2d.cc`,
//! `brush_painter_2d_tex_mapping` + `BKE_brush_sample_tex_3d`), **adapted to 2D**: the `3D` mapping
//! mode and the `Z` axis are dropped (degenerate for raster paint). The texture multiplies the
//! falloff mask per pixel, exactly as the falloff weight does — see [`crate::dab::stamp_dab`].
//!
//! **Determinism (HR-5).** Transcendental-free: rotation is carried as a unit *vector*, never an
//! angle (as in `stroke/ellipse.rs` / `stroke/polygon.rs`). **Angle** rotates `(1,0)` by repeated
//! application of the baked 1° step [`DEG_STEP`]; **Rake** uses the stroke tangent; **Random** builds
//! a per-dab unit vector from the dep-free splitmix64 RNG. The sampler uses only `floor`/`*`/`+`/`sqrt`.

pub(crate) mod patterns;
mod shape;
mod stencil;
use crate::heading::rotate;
pub use shape::{
    ImageRgb, compose_shape_silhouette_kind, remap_shape_value, render_shape_preview,
    sample_shape_rgb_unit, sample_shape_silhouette, sample_shape_silhouette_unit,
};
pub use stencil::{render_stencil_preview, stencil_frame, stencil_gate};

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

/// The built-in texture patterns — the Blender texture set (clean-room, [`patterns`]) plus
/// painting-useful extras. `None` = no texture assigned (the dab is unmodulated). The discriminants
/// `0..=5` are wire-stable from the original set; new kinds append from `6`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextureKind {
    /// No texture — [`sample`] returns `1.0` (full coverage), so the dab is unchanged.
    #[default]
    None,
    /// Value noise (grain) — the canonical brush texture (pencil / charcoal tooth).
    Noise,
    /// Hard 2-colour checker — useful for reading the mapping, and a crisp pattern.
    Checker,
    /// Voronoi cells (F1 distance) — organic, blotchy.
    Voronoi,
    /// Soft parallel stripes (triangle wave) — hatching.
    Stripes,
    /// An imported image's luminance, supplied separately (the pixels are heavy, so they don't live
    /// in the `Copy` settings — the caller passes an [`ImageMask`] to [`sample`]). Without one, the
    /// texture is inert (returns `1.0`).
    Image,
    /// Fractal noise — soft billowy cloud field (Blender `Clouds`).
    Clouds,
    /// Value noise sampled through a noise-warped coordinate (Blender `Distorted Noise`).
    DistortedNoise,
    /// Nested interference waves — swirly organic pattern (Blender `Magic`).
    Magic,
    /// Turbulence-distorted veins / bands (Blender `Marble`).
    Marble,
    /// Ridged multifractal — sharp creases at many scales (Blender `Musgrave`).
    Musgrave,
    /// Concentric growth rings with light turbulence (Blender `Wood`).
    Wood,
    /// Thresholded fractal noise — rough plaster relief (Blender `Stucci`).
    Stucci,
    /// Smooth linear ramp repeating per tile (Blender `Blend`).
    Gradient,
    /// Fine multi-frequency grain — paper / canvas tooth for dry media.
    Grain,
    /// Crossed diagonal hatch lines (ink-hatch shading).
    Crosshatch,
    /// Soft round dots centred per tile (halftone).
    Dots,
    /// Thin lattice lines (mesh / graph-paper).
    Grid,
    /// Running-bond rectangles with mortar gaps (bricks).
    Bricks,
    /// Smooth horizontal bands rippled along x (water / silk).
    Waves,
    /// V-shaped zigzag bands.
    Chevron,
    /// 45°-rotated checker of diamonds (harlequin).
    Diamonds,
    /// Two-tone triangular tiling.
    Triangles,
    /// Honeycomb hexagon cells with bright rims.
    Hexagons,
    /// Overlapping ringed discs (fish-scale / scallops).
    Scales,
    /// Over-under woven bands (basketweave).
    Weave,
    /// **Cold-press** watercolor paper — a medium random tooth with mild laid-line fibre (the classic
    /// "NOT" surface). Procedural height-field; feeds the watercolor granulation (`docs/Painter/10…`).
    PaperCold,
    /// **Rough** watercolor paper — a deep, pronounced tooth with strong fibre creases (heavy pooling).
    PaperRough,
    /// **Hot-press** watercolor paper — a fine, smooth grain with a soft felt mottle (minimal tooth).
    PaperHot,
}

impl TextureKind {
    /// Stable wire discriminant for the panel dropdown / round-trip tests.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Noise => 1,
            Self::Checker => 2,
            Self::Voronoi => 3,
            Self::Stripes => 4,
            Self::Image => 5,
            Self::Clouds => 6,
            Self::DistortedNoise => 7,
            Self::Magic => 8,
            Self::Marble => 9,
            Self::Musgrave => 10,
            Self::Wood => 11,
            Self::Stucci => 12,
            Self::Gradient => 13,
            Self::Grain => 14,
            Self::Crosshatch => 15,
            Self::Dots => 16,
            Self::Grid => 17,
            Self::Bricks => 18,
            Self::Waves => 19,
            Self::Chevron => 20,
            Self::Diamonds => 21,
            Self::Triangles => 22,
            Self::Hexagons => 23,
            Self::Scales => 24,
            Self::Weave => 25,
            Self::PaperCold => 26,
            Self::PaperRough => 27,
            Self::PaperHot => 28,
        }
    }

    /// Inverse of [`Self::to_u8`]; unknown values fall back to [`Self::None`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Noise,
            2 => Self::Checker,
            3 => Self::Voronoi,
            4 => Self::Stripes,
            5 => Self::Image,
            6 => Self::Clouds,
            7 => Self::DistortedNoise,
            8 => Self::Magic,
            9 => Self::Marble,
            10 => Self::Musgrave,
            11 => Self::Wood,
            12 => Self::Stucci,
            13 => Self::Gradient,
            14 => Self::Grain,
            15 => Self::Crosshatch,
            16 => Self::Dots,
            17 => Self::Grid,
            18 => Self::Bricks,
            19 => Self::Waves,
            20 => Self::Chevron,
            21 => Self::Diamonds,
            22 => Self::Triangles,
            23 => Self::Hexagons,
            24 => Self::Scales,
            25 => Self::Weave,
            26 => Self::PaperCold,
            27 => Self::PaperRough,
            28 => Self::PaperHot,
            _ => Self::None,
        }
    }

    /// Number of selectable kinds (drives the dropdown decode range; includes `None`).
    pub const COUNT: u8 = 29;

    /// English label for the picker (HR-15 / app-UI-english-only).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Noise => "Noise",
            Self::Checker => "Checker",
            Self::Voronoi => "Voronoi",
            Self::Stripes => "Stripes",
            Self::Image => "Image",
            Self::Clouds => "Clouds",
            Self::DistortedNoise => "Distorted Noise",
            Self::Magic => "Magic",
            Self::Marble => "Marble",
            Self::Musgrave => "Musgrave",
            Self::Wood => "Wood",
            Self::Stucci => "Stucci",
            Self::Gradient => "Gradient",
            Self::Grain => "Grain",
            Self::Crosshatch => "Crosshatch",
            Self::Dots => "Dots",
            Self::Grid => "Grid",
            Self::Bricks => "Bricks",
            Self::Waves => "Waves",
            Self::Chevron => "Chevron",
            Self::Diamonds => "Diamonds",
            Self::Triangles => "Triangles",
            Self::Hexagons => "Hexagons",
            Self::Scales => "Scales",
            Self::Weave => "Weave",
            Self::PaperCold => "Paper Cold Press",
            Self::PaperRough => "Paper Rough",
            Self::PaperHot => "Paper Hot Press",
        }
    }
}

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

    /// Whether the per-dab Rake / Random rotation applies (Stencil has its own fixed frame → ignores them).
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
    /// **Rake**: the rotation follows the stroke direction, with [`Self::angle_deg`] composed as an offset.
    pub rake: bool,
    /// **Random**: the rotation is randomised per dab (overrides Rake and [`Self::angle_deg`]).
    pub random_angle: bool,
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
            random_angle: false,
            offset: [0.0, 0.0],
            size: [1.0, 1.0],
            stencil_offset: [0.0, 0.0],
            stencil_size: [0.5, 0.5],
            stencil_angle_deg: 0,
            params: [0.5; MAX_TEX_PARAMS],
        }
    }
}

pub use patterns::{ParamSpec, param_specs};

impl TextureSettings {
    /// Whether the texture actually modulates anything (a kind is assigned).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.kind != TextureKind::None
    }

    /// Stamp is dab-relative + constant across a stroke → render once into a mask + scale-blit
    /// (Blender's brush-image cache; see [`crate::stamp`]). True with no texture or a static **View**
    /// texture; Rake / Random (per-dab rotation) and Tiled / Stencil (canvas-relative) stay per-pixel.
    #[must_use]
    pub fn is_cacheable(&self) -> bool {
        !self.is_active()
            || (matches!(self.mapping, TextureMapping::ViewPlane)
                && !self.rake
                && !self.random_angle)
    }

    /// Texture is canvas-fixed + dab-independent → cache each canvas pixel once per stroke
    /// ([`crate::stamp::blit_canvas_cached`]). Static **Tiled** / **Stencil**; Rake / Random stay per-pixel.
    #[must_use]
    pub fn is_canvas_cacheable(&self) -> bool {
        self.is_active()
            && matches!(
                self.mapping,
                TextureMapping::Tiled | TextureMapping::Stencil
            )
            && !self.rake
            && !self.random_angle
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
        }
    }
}

/// Resolve the per-dab texture frame from the settings, the stroke tangent `dab_dir` (Rake; falls back to
/// [`TextureSettings::angle_deg`]), a splitmix64 `rng` (Random), the canvas (Stencil rect), the per-dab
/// **Jitter Rotate** `extra_rot` (`[1,0]` = none) and the dab `footprint` (Tiled / Stencil ignore it).
#[must_use]
pub fn dab_basis(
    s: &TextureSettings,
    dab_dir: [f32; 2],
    rng: &mut u64,
    canvas: [f32; 2],
    extra_rot: [f32; 2],
    footprint: crate::footprint::FootprintDeform,
) -> TexDabBasis {
    // Stencil = a fixed image-space rect (canvas-fixed, so no Rake/Random/Jitter and no dab flatten).
    if s.mapping.is_stencil() {
        return stencil::stencil_basis(s, canvas);
    }
    let base = if s.random_angle {
        random_unit(rng)
    } else if s.rake {
        // Rake follows the stroke heading, with Angle composed on top (empty heading ⇒ Angle alone).
        let h = normalize_or(dab_dir, [1.0, 0.0]);
        rotate(h, rotate_by_degrees(s.angle_deg))
    } else {
        rotate_by_degrees(s.angle_deg)
    };
    // Compose the per-dab Jitter Rotate (2D rotation); `extra_rot = [1, 0]` = no jitter (bit-identical).
    let u = rotate(base, extra_rot);
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
    }
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
    } else {
        // Per-axis scale clamped away from zero. Size MULTIPLIES the coordinate (Blender's MTex
        // `texvec = size · co`): a LARGER Size scales coords up → the pattern reads SMALLER / denser.
        let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        // Raw coordinates before rotation, in tile units (footprint-relative or canvas-tiled).
        let rel = match s.mapping {
            TextureMapping::Tiled => {
                let base = TEX_TILE_BASE_PX;
                [p[0] * sx / base, p[1] * sy / base]
            }
            // View / Random anchor to the dab footprint; the dab flatten/rotate deforms it FIRST so
            // the pattern flattens with the falloff (the texture's own Size stays relative to it).
            _ => {
                let r = radius.max(1e-3);
                let f = b
                    .footprint
                    .apply([(p[0] - center[0]) / r, (p[1] - center[1]) / r]);
                [f[0] * sx, f[1] * sy]
            }
        };
        [
            rel[0] * b.u[0] + rel[1] * b.u[1] + s.offset[0] + b.jitter[0],
            rel[0] * b.v[0] + rel[1] * b.v[1] + s.offset[1] + b.jitter[1],
        ]
    };
    patterns::sample_kind(s.kind, tex, s.params, image).clamp(0.0, 1.0)
}

/// Sample a **canvas-anchored** (Tiled) texture at canvas pixel `(px, py)` — the paper as a fixed
/// height-field over the image, independent of any dab (the watercolor render-path reads the Grain slot
/// through this to granulate the wash, `docs/Painter/10…`). Identity basis, no rotation/jitter; honours
/// the texture's Size + Offset. `image` backs [`TextureKind::Image`] (a tagged layer used as paper);
/// without it that kind is inert. Returns the coverage in `[0, 1]`; `1.0` when the slot is inactive.
#[must_use]
pub fn sample_tiled(s: &TextureSettings, px: i64, py: i64, image: Option<&ImageMask>) -> f32 {
    sample_tiled_rot(s, px, py, image, angle_basis(s.angle_deg))
}

/// The rotation basis `[cos, sin]` for a whole-degree angle (transcendental-free, HR-5) — precompute it
/// once, then feed it to [`sample_tiled_rot`] per pixel so the per-call cost is a few mults, not trig.
#[must_use]
pub fn angle_basis(deg: u16) -> [f32; 2] {
    rotate_by_degrees(deg)
}

/// [`sample_tiled`] with a caller-precomputed rotation basis `rot` (from [`angle_basis`]) — the hot-loop
/// form: the tile coords are rotated by `rot` before the pattern is sampled, so the paper honours its
/// **Angle** (fibre orientation). `rot = [1, 0]` is no rotation.
#[must_use]
pub fn sample_tiled_rot(
    s: &TextureSettings,
    px: i64,
    py: i64,
    image: Option<&ImageMask>,
    rot: [f32; 2],
) -> f32 {
    if !s.is_active() {
        return 1.0;
    }
    let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let p = [px as f32 + 0.5, py as f32 + 0.5];
    let rel = [p[0] * sx / TEX_TILE_BASE_PX, p[1] * sy / TEX_TILE_BASE_PX];
    // Rotate the tile coordinates by the Angle basis, then apply the Offset.
    let tex = [
        rel[0] * rot[0] - rel[1] * rot[1] + s.offset[0],
        rel[0] * rot[1] + rel[1] * rot[0] + s.offset[1],
    ];
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
    let f = b.footprint.apply([u, v]); // dab flatten/rotate, before this texture's own Size/rotation
    let rel = [f[0] * sx, f[1] * sy];
    let tex = [
        rel[0] * b.u[0] + rel[1] * b.u[1] + s.offset[0],
        rel[0] * b.v[0] + rel[1] * b.v[1] + s.offset[1],
    ];
    patterns::sample_kind(s.kind, tex, s.params, image).clamp(0.0, 1.0)
}

// ── Rotation / RNG helpers (transcendental-free) ────────────────────────────────────────────

/// Rotate the unit vector `(1, 0)` by `deg` whole degrees via repeated [`DEG_STEP`] application.
/// `pub(crate)` so [`crate::jitter`] builds the per-dab Jitter Rotate vector from the same baked step.
pub(crate) fn rotate_by_degrees(deg: u16) -> [f32; 2] {
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
fn normalize_or(v: [f32; 2], fallback: [f32; 2]) -> [f32; 2] {
    let len2 = v[0] * v[0] + v[1] * v[1];
    if len2 > 1e-12 {
        let inv = 1.0 / len2.sqrt();
        [v[0] * inv, v[1] * inv]
    } else {
        fallback
    }
}

/// A deterministic random unit vector via rejection sampling in the unit disc (sqrt only). Bounded
/// to a few tries; falls back to `(1, 0)` in the vanishingly unlikely all-reject case.
fn random_unit(rng: &mut u64) -> [f32; 2] {
    for _ in 0..8 {
        let x = crate::jitter::next_f32(rng) * 2.0 - 1.0;
        let y = crate::jitter::next_f32(rng) * 2.0 - 1.0;
        let d2 = x * x + y * y;
        if (1e-6..=1.0).contains(&d2) {
            let inv = 1.0 / d2.sqrt();
            return [x * inv, y * inv];
        }
    }
    [1.0, 0.0]
}

#[cfg(test)]
mod tests;
