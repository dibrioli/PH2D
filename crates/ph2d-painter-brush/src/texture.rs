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

/// The built-in procedural texture patterns. `None` = no texture assigned (the dab is unmodulated;
/// matches the checkerboard "empty" placeholder in the panel).
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
            _ => Self::None,
        }
    }

    /// Number of selectable kinds (drives the dropdown decode range; includes `None`).
    pub const COUNT: u8 = 6;

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
        }
    }
}

/// How texture coordinates are derived from the dab. `3D` is dropped (2D adaptation); `Stencil` is a
/// later phase (needs a screen-space overlay), so P1 exposes View/Tiled/Random.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextureMapping {
    /// Coordinates relative to the dab footprint, centred on the cursor — the texture *follows the
    /// brush*. The 2D-paint default.
    #[default]
    ViewPlane,
    /// Coordinates from the canvas position — the texture is *fixed to the image* while you paint
    /// over it.
    Tiled,
    /// Like [`Self::ViewPlane`] but with a random per-dab offset.
    Random,
    /// A positioned/rotated/scaled rectangular **stencil** you paint *through*: the texture fills the
    /// rect once and masks outside it. 2D-adapted to **image space** (fixed to the canvas, not the
    /// screen), driven by Offset (centre) / Size (extent) / Angle (rotation). See [`dab_basis`].
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
            Self::Random => "Random",
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
}

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
        }
    }
}

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
/// splitmix64 `rng` (Random), and the canvas size (Stencil rect). Deterministic per input platform.
#[must_use]
pub fn dab_basis(
    s: &TextureSettings,
    dab_dir: [f32; 2],
    rng: &mut u64,
    canvas: [f32; 2],
) -> TexDabBasis {
    // Stencil has a single fixed image-space frame (Offset = centre, Size = extent, Angle =
    // rotation); the per-dab Rake / Random rotation and offset-jitter do not apply.
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
    let u = if s.random_angle {
        random_unit(rng)
    } else if s.rake {
        normalize_or(dab_dir, rotate_by_degrees(s.angle_deg))
    } else {
        rotate_by_degrees(s.angle_deg)
    };
    let v = perp(u);
    let jitter = if s.mapping.randomises_offset() {
        // A full-tile random shift per dab, in tile fractions.
        [next_f32(rng), next_f32(rng)]
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
        // Per-axis scale clamped away from zero so the division is finite.
        let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        // Raw coordinates before rotation, in tile units (footprint-relative or canvas-tiled).
        let rel = match s.mapping {
            TextureMapping::Tiled => {
                let base = TEX_TILE_BASE_PX;
                [p[0] / (base * sx), p[1] / (base * sy)]
            }
            // View Plane and Random both anchor to the dab footprint.
            _ => {
                let r = radius.max(1e-3);
                [(p[0] - center[0]) / (r * sx), (p[1] - center[1]) / (r * sy)]
            }
        };
        // Rotate into texture space (basis is already the dab's rotation), then translate.
        [
            rel[0] * b.u[0] + rel[1] * b.u[1] + s.offset[0] + b.jitter[0],
            rel[0] * b.v[0] + rel[1] * b.v[1] + s.offset[1] + b.jitter[1],
        ]
    };
    sample_kind(s.kind, tex, image).clamp(0.0, 1.0)
}

/// Evaluate the procedural / image kind at texture coords `tex` (after the mapping resolved them).
/// Shared by [`sample`] (per-pixel canvas path) and [`sample_unit`] (the cached stamp render).
fn sample_kind(kind: TextureKind, tex: [f32; 2], image: Option<&ImageMask>) -> f32 {
    match kind {
        TextureKind::None => 1.0,
        TextureKind::Noise => value_noise(tex[0], tex[1]),
        TextureKind::Checker => checker(tex[0], tex[1]),
        TextureKind::Voronoi => voronoi(tex[0], tex[1]),
        TextureKind::Stripes => stripes(tex[0]),
        TextureKind::Image => match image {
            Some(img) => sample_image(img, tex[0], tex[1]),
            None => 1.0, // kind is Image but no pixels supplied → inert
        },
    }
}

/// Sample the **View-mapped** texture at the dab-relative unit coord `(u, v) ∈ [-1, 1]` — the
/// scale-invariant form baking the cached brush stamp ([`crate::stamp`]): depends only on
/// `(u, v)/size` + rotation, never the radius. View-only; returns the coverage in `[0, 1]`.
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
    let rel = [u / sx, v / sy];
    let tex = [
        rel[0] * b.u[0] + rel[1] * b.u[1] + s.offset[0],
        rel[0] * b.v[0] + rel[1] * b.v[1] + s.offset[1],
    ];
    sample_kind(s.kind, tex, image).clamp(0.0, 1.0)
}

/// Bilinear sample of `img`'s luminance at tile coords `(u, v)` (1 unit = one image), tiled via
/// `fract`, centre-coord convention (memory `feedback_pixel_center_vs_edge_coord`). Returns `[0, 1]`;
/// transcendental-free. A malformed buffer reads `1.0` (inert) rather than panicking.
fn sample_image(img: &ImageMask, u: f32, v: f32) -> f32 {
    let (w, h) = (img.width.max(1), img.height.max(1));
    if img.lum.len() < (w as usize) * (h as usize) {
        return 1.0;
    }
    // Tile into [0,1), then to pixel space (centre-coord: pixel i spans [i, i+1), centre at i+0.5).
    let x = (u - u.floor()) * w as f32 - 0.5;
    let y = (v - v.floor()) * h as f32 - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let wrap = |i: i32, n: u32| (i.rem_euclid(n as i32)) as usize;
    let (xi0, xi1) = (wrap(x0 as i32, w), wrap(x0 as i32 + 1, w));
    let (yi0, yi1) = (wrap(y0 as i32, h), wrap(y0 as i32 + 1, h));
    let at = |xi: usize, yi: usize| f32::from(img.lum[yi * w as usize + xi]) / 255.0;
    let top = lerp(at(xi0, yi0), at(xi1, yi0), tx);
    let bot = lerp(at(xi0, yi1), at(xi1, yi1), tx);
    lerp(top, bot, ty)
}

// ── Procedural samplers (transcendental-free) ───────────────────────────────────────────────

/// Hard 2-colour checker: `0.0` / `1.0` by the parity of the integer cell.
fn checker(u: f32, v: f32) -> f32 {
    let cell = ifloor(u) ^ ifloor(v);
    (cell & 1) as f32
}

/// Soft parallel stripes along `u` — a unit-period triangle wave in `[0, 1]`.
fn stripes(u: f32) -> f32 {
    let f = u - u.floor(); // [0,1)
    if f < 0.5 { 2.0 * f } else { 2.0 * (1.0 - f) }
}

/// One octave of value noise in `[0, 1]`: hashed lattice values, smoothstep-interpolated.
fn value_noise(u: f32, v: f32) -> f32 {
    let x0 = ifloor(u);
    let y0 = ifloor(v);
    let fx = u - u.floor();
    let fy = v - v.floor();
    let sx = smoothstep(fx);
    let sy = smoothstep(fy);
    let n00 = hash2(x0, y0);
    let n10 = hash2(x0 + 1, y0);
    let n01 = hash2(x0, y0 + 1);
    let n11 = hash2(x0 + 1, y0 + 1);
    lerp(lerp(n00, n10, sx), lerp(n01, n11, sx), sy)
}

/// Voronoi F1: nearest-feature distance over the 3×3 neighbour cells, mapped to `[0, 1]`.
fn voronoi(u: f32, v: f32) -> f32 {
    let cx = ifloor(u);
    let cy = ifloor(v);
    let mut best = f32::INFINITY;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (gx, gy) = (cx + dx, cy + dy);
            // Feature point inside cell (gx, gy): cell corner + hashed in-cell offset.
            let fx = gx as f32 + hash2(gx, gy);
            let fy = gy as f32 + hash2(gy, gx);
            let (ex, ey) = (fx - u, fy - v);
            let d2 = ex * ex + ey * ey;
            if d2 < best {
                best = d2;
            }
        }
    }
    best.sqrt().clamp(0.0, 1.0)
}

// ── Rotation / RNG / math helpers (transcendental-free) ─────────────────────────────────────

/// Rotate the unit vector `(1, 0)` by `deg` whole degrees via repeated [`DEG_STEP`] application.
fn rotate_by_degrees(deg: u16) -> [f32; 2] {
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
        let x = next_f32(rng) * 2.0 - 1.0;
        let y = next_f32(rng) * 2.0 - 1.0;
        let d2 = x * x + y * y;
        if (1e-6..=1.0).contains(&d2) {
            let inv = 1.0 / d2.sqrt();
            return [x * inv, y * inv];
        }
    }
    [1.0, 0.0]
}

/// Dep-free deterministic `[0, 1)` RNG (splitmix64 → top 24 bits). Mirrors
/// `stroke.rs::Stroke::next_f32` so jitter/texture share one stream model.
fn next_f32(state: &mut u64) -> f32 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    ((z >> 40) as f32) / ((1u32 << 24) as f32)
}

/// Hash an integer lattice point to `[0, 1)` — the value-noise / Voronoi randomness.
fn hash2(ix: i32, iy: i32) -> f32 {
    let mut h = (ix as u32).wrapping_mul(0x9E37_79B1) ^ (iy as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    ((h >> 8) as f32) / ((1u32 << 24) as f32)
}

/// Hermite smoothstep `3t² − 2t³` on a value already in `[0, 1]` (polynomial — no transcendental).
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolate.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `floor` to `i32` (integer cell index) — avoids the `as i32` truncation-toward-zero bug for
/// negative coordinates.
fn ifloor(x: f32) -> i32 {
    x.floor() as i32
}

#[cfg(test)]
mod tests;
