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
pub use shape::{
    compose_shape_silhouette_kind, remap_shape_value, render_shape_preview,
    sample_shape_silhouette, sample_shape_silhouette_unit,
};

/// Largest **Angle** the slider reaches, in whole degrees (one full turn).
pub const TEX_ANGLE_MAX_DEG: u16 = 360;
/// **Offset** range, in tile fractions (one unit = one full tile shift). Symmetric about 0.
pub const TEX_OFFSET_MIN: f32 = -1.0;
/// See [`TEX_OFFSET_MIN`].
pub const TEX_OFFSET_MAX: f32 = 1.0;
/// **Size** (scale) range. `1.0` = one tile per footprint (View) / per [`TEX_TILE_BASE_PX`] (Tiled).
pub const TEX_SIZE_MIN: f32 = 0.1;
/// See [`TEX_SIZE_MIN`].
pub const TEX_SIZE_MAX: f32 = 10.0;
/// Canvas pixels spanned by one texture tile at Size `1.0` under the **Tiled** mapping.
pub const TEX_TILE_BASE_PX: f32 = 256.0;
/// Procedural tiles the **Stencil** rect spans per axis, so the pattern reads (one tile would be a
/// flat cell). Density is fixed; the rect's on-canvas size is [`TextureSettings::size`].
const STENCIL_TILES: f32 = 4.0;

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
            _ => Self::None,
        }
    }

    /// Number of selectable kinds (drives the dropdown decode range; includes `None`).
    pub const COUNT: u8 = 26;

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

    /// Whether the per-dab Rake / Random rotation applies. Stencil has its own fixed frame, so it
    /// ignores them (the panel hides those controls for Stencil).
    #[must_use]
    pub fn uses_dab_rotation(self) -> bool {
        !matches!(self, Self::Stencil)
    }

    /// Whether this is the [`Self::Stencil`] mapping (image-space positioned mask).
    #[must_use]
    pub fn is_stencil(self) -> bool {
        matches!(self, Self::Stencil)
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
    /// **Rake**: the rotation follows the stroke direction (overrides [`Self::angle_deg`]).
    pub rake: bool,
    /// **Random**: the rotation is randomised per dab (overrides Rake and [`Self::angle_deg`]).
    pub random_angle: bool,
    /// Translation in tile fractions, each component in `[`[`TEX_OFFSET_MIN`]`, `[`TEX_OFFSET_MAX`]`]`.
    pub offset: [f32; 2],
    /// Per-axis scale, each in `[`[`TEX_SIZE_MIN`]`, `[`TEX_SIZE_MAX`]`]` (`1.0` = one tile).
    pub size: [f32; 2],
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

    /// Texture is canvas-fixed + dab-independent → cache each canvas pixel's value once per stroke
    /// (see [`crate::stamp::blit_canvas_cached`]). True for static **Tiled** / **Stencil**; Rake /
    /// Random vary per dab, so they stay per-pixel.
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

/// A borrowed grayscale (luminance) image supplied to [`sample`] for [`TextureKind::Image`]. One
/// byte per texel, row-major; the engine samples it bilinearly (centre-coord) and tiles it (`fract`)
/// so it composes with every mapping. Kept borrowed so the heavy pixels stay owned by the caller
/// (the `Copy` settings can't hold them).
#[derive(Clone, Copy, Debug)]
pub struct ImageMask<'a> {
    /// Luminance bytes, row-major, at least `width * height` long.
    pub lum: &'a [u8],
    /// Image width in texels.
    pub width: u32,
    /// Image height in texels.
    pub height: u32,
}

/// Per-dab resolved texture frame: the rotated unit basis (`u`, `v`), the per-dab random translation
/// (tile fractions; `[0,0]` unless randomised), and — for Stencil — the rect's centre + half-extent
/// in canvas px. Computed once per dab by [`dab_basis`] so the per-pixel [`sample`] is cheap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TexDabBasis {
    u: [f32; 2],
    v: [f32; 2],
    jitter: [f32; 2],
    /// Stencil rect centre in canvas px (unused unless the mapping is Stencil).
    stencil_center: [f32; 2],
    /// Stencil rect half-extent in canvas px, per axis (unused unless Stencil).
    stencil_half: [f32; 2],
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
        }
    }
}

/// Resolve the per-dab texture frame from the settings, the stroke tangent `dab_dir` (Rake; need not
/// be normalised — a near-zero tangent falls back to [`TextureSettings::angle_deg`]), a mutable
/// splitmix64 `rng` (Random), the canvas size (Stencil rect), and the per-dab **Jitter Rotate** unit
/// vector `extra_rot` (identity `[1, 0]` = none). Deterministic per input platform.
#[must_use]
pub fn dab_basis(
    s: &TextureSettings,
    dab_dir: [f32; 2],
    rng: &mut u64,
    canvas: [f32; 2],
    extra_rot: [f32; 2],
) -> TexDabBasis {
    // Stencil has a single fixed image-space frame (Offset = centre, Size = extent, Angle =
    // rotation); the per-dab Rake / Random / Jitter rotation and offset-jitter do not apply.
    if s.mapping.is_stencil() {
        let (center, half, u) = stencil_frame(s, canvas);
        return TexDabBasis {
            u,
            v: perp(u),
            jitter: [0.0, 0.0],
            stencil_center: center,
            stencil_half: half,
        };
    }
    let base = if s.random_angle {
        random_unit(rng)
    } else if s.rake {
        normalize_or(dab_dir, rotate_by_degrees(s.angle_deg))
    } else {
        rotate_by_degrees(s.angle_deg)
    };
    // Compose the per-dab Jitter Rotate on top (2D rotation = complex multiply). `extra_rot = [1, 0]`
    // leaves `base` unchanged, so a non-jittering brush is bit-identical to the old single-rotation frame.
    let u = [
        base[0] * extra_rot[0] - base[1] * extra_rot[1],
        base[0] * extra_rot[1] + base[1] * extra_rot[0],
    ];
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
    }
}

/// The Stencil rect's centre + half-extent (canvas px) + rotation unit vector. Offset maps `[-1, 1]`
/// onto the canvas span (centre at `0`); Size is the half-extent as a canvas fraction (`1.0` ≈ full);
/// rotation is the baked [`TextureSettings::angle_deg`]. Shared by [`dab_basis`] and the tool's
/// overlay so the painted mask and its outline agree exactly.
#[must_use]
pub fn stencil_frame(s: &TextureSettings, canvas: [f32; 2]) -> ([f32; 2], [f32; 2], [f32; 2]) {
    let center = [
        (0.5 + 0.5 * s.offset[0]) * canvas[0],
        (0.5 + 0.5 * s.offset[1]) * canvas[1],
    ];
    let half = [
        (s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX) * 0.5 * canvas[0]).max(1e-3),
        (s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX) * 0.5 * canvas[1]).max(1e-3),
    ];
    (center, half, rotate_by_degrees(s.angle_deg))
}

/// Sample the texture at canvas pixel `(px, py)` for a dab centred at `center` with `radius` (the
/// values already known inside [`crate::dab::stamp_dab`]). Returns the coverage multiplier in
/// `[0, 1]`; `1.0` when no texture is assigned (so the dab is unchanged).
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
    // rect, and maps the rect onto one tile of the procedural.
    let tex = if s.mapping.is_stencil() {
        let rel = [p[0] - b.stencil_center[0], p[1] - b.stencil_center[1]];
        let lx = (rel[0] * b.u[0] + rel[1] * b.u[1]) / b.stencil_half[0];
        let ly = (rel[0] * b.v[0] + rel[1] * b.v[1]) / b.stencil_half[1];
        if lx.abs() > 1.0 || ly.abs() > 1.0 {
            return 0.0; // outside the stencil → paint nothing
        }
        // Map [-1,1]² onto a fixed-density tile window so the procedural pattern reads in the rect.
        [
            (lx + 1.0) * 0.5 * STENCIL_TILES,
            (ly + 1.0) * 0.5 * STENCIL_TILES,
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
                [p[0] * sx / base, p[1] * sy / base]
            }
            // View Plane and Random both anchor to the dab footprint.
            _ => {
                let r = radius.max(1e-3);
                [(p[0] - center[0]) * sx / r, (p[1] - center[1]) * sy / r]
            }
        };
        // Rotate into texture space (basis is already the dab's rotation), then translate.
        [
            rel[0] * b.u[0] + rel[1] * b.u[1] + s.offset[0] + b.jitter[0],
            rel[0] * b.v[0] + rel[1] * b.v[1] + s.offset[1] + b.jitter[1],
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
    let rel = [u * sx, v * sy];
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
