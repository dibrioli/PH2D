//! The texture **kind** — which built-in pattern a slot draws, and its wire-stable discriminant.
//! Split out of `texture.rs` to keep that file under the workspace LOC cap.

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
