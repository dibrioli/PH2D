//! Icon glyph table (Phase 0.1 of the M13 UI sprint).
//!
//! Every glyph in `docs/design/icons/*.svg`, ported. Each icon is a
//! list of [`IconCmd`] primitives (paths, polylines, lines, circles,
//! rounded rects) on a 24x24 viewBox. The paint helper strokes them
//! with `currentColor` semantics: caller passes the color, we apply
//! it to every command.
//!
//! Source SVGs are Lucide-derived (ISC license). They use
//! `fill="none" stroke="currentColor" stroke-width="1.5"` —
//! we replicate that contract here.
//!
//! **Wave 2 PR 11.2: command lists are codegen'd by `build.rs` from
//! `docs/design/icons/*.svg`.** The 715-LOC manually-ported match
//! arms that previously lived here are gone — values flow from
//! canonical SVGs at every `cargo build`. Adding an icon: drop a SVG
//! in `docs/design/icons/<slug>.svg`, add the matching CamelCase
//! variant to `enum IconId` below in alphabetical order (matches the
//! sort `build.rs` applies). Order check enforced by test.

use ph2d_vector::BezPath;

/// Codegen'd from `docs/design/icons/*.svg`. Exposed under
/// `pub(crate)` because consumers in this crate may want the slug
/// lookup; runtime API is the `IconId`/`cmds()` pair.
///
/// `excessive_precision` allow: build.rs emits f32 literals padded to
/// 6 decimal places for consistent diff output. Clippy flags exact
/// representations like `12.010000` — purely cosmetic, hence allowed
/// only inside this codegen'd module.
///
/// `dead_code` allow: `lookup_cmds` + `ALL_ICON_SLUGS` are exported
/// for future chrome derivation (PR 11.4); `cmds()` consumes only
/// `ICON_CMDS_BY_ID` for now.
#[allow(clippy::excessive_precision, dead_code)]
pub(crate) mod icons_generated {
    include!(concat!(env!("OUT_DIR"), "/icons_generated.rs"));
}

/// Compact representation of one drawing command in an icon.
///
/// Numbers are in the source 24x24 viewBox; the paint helper applies
/// the affine that maps that box into the destination rect.
#[derive(Clone, Copy, Debug)]
pub enum IconCmd {
    /// Raw SVG path "d" attribute. Parsed via [`BezPath::from_svg`].
    Path(&'static str),
    /// SVG polyline points: `"x1 y1 x2 y2 ..."`.
    Polyline(&'static str),
    /// SVG line: `(x1, y1, x2, y2)`.
    Line(f32, f32, f32, f32),
    /// SVG circle: `(cx, cy, r)`.
    Circle(f32, f32, f32),
    /// SVG rect with optional corner radius: `(x, y, w, h, rx)`.
    /// `rx == 0.0` means sharp corners.
    Rect(f32, f32, f32, f32, f32),
}

/// The canonical icon set — one variant per `docs/design/icons/*.svg`.
///
/// ⚠️ No count in this prose, on purpose: the two numbers that used to be here ("89" and "100") were
/// both stale against a set of 136, and a number that only a human re-counts is a number that lies.
/// The array below IS the count, and `enum_order_matches_svgs` is what keeps it true.
///
/// Variants declared in **alphabetical order by kebab-case slug**
/// (matching `docs/design/icons/*.svg` filenames sorted). Order is
/// load-bearing: `cmds()` and `slug()` index into codegen'd tables
/// by `self as usize`, so reorder breaks every icon lookup silently.
/// `enum_order_matches_svgs` test pins the contract.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IconId {
    Add,
    AlignBottom,
    AlignCenter,
    AlignLeft,
    AlignMiddle,
    AlignRight,
    AlignTop,
    ArrangeBack,
    ArrangeBackward,
    ArrangeForward,
    ArrangeFront,
    Asset,
    Audio,
    Backdrop,
    BgRemoval,
    Blur,
    Bolt,
    Bone,
    BoolExclude,
    BoolIntersect,
    BoolSubtract,
    BoolUnion,
    Bug,
    Build,
    Camera,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    Circle,
    Clone,
    Close,
    Cmd,
    Collider,
    // ⚠️ **`Color` ANTES de `ColorEqualization`, e a ordem é o glifo.** `"color"` ordena antes de
    // `"color-equalization"` (prefixo), e a posição do variante É o índice nas tabelas geradas.
    // Declarados ao contrário, `IconId::Color` desenhava os SLIDERS e `IconId::ColorEqualization`
    // desenhava a PALETA — ver o gate `every_variant_is_named_after_its_slug`.
    Color,
    ColorEqualization,
    Command,
    Console,
    ConvertCurves,
    Copy,
    Cube,
    Database,
    Delete,
    DistributeH,
    DistributeV,
    Duplicate,
    EqualizeSizes,
    Erase,
    Error,
    Export,
    Eye,
    EyeClosed,
    EyePencil,
    Eyedropper,
    File,
    Film,
    FitView,
    Flip,
    FlipH,
    FlipV,
    Folder,
    Fps,
    Gizmo,
    Grid,
    GridSettings,
    Group,
    Help,
    Hidden,
    Hierarchy,
    History,
    HotReload,
    Image,
    Info,
    Inpaint,
    Inspector,
    Kbd,
    Knife,
    Layer,
    Layers,
    LayoutGrid,
    LetterH,
    LetterI,
    Light,
    Line,
    Link,
    /// Liquify — the inward spiral. The universal figure for a warp that swirls pixels (Photoshop
    /// Twirl, Procreate/Affinity Liquify), and deliberately a NESTED double loop: the app's other
    /// circular glyphs (Rotate / Reset / History / Spinner) are all a single open arc with a tail, so
    /// the two families cannot be confused at chip size.
    Liquify,
    Lock,
    LockKeyhole,
    LockKeyholeOpen,
    MakeSquare,
    MarqueeBox,
    MarqueeLasso,
    Mask,
    Material,
    Maximize,
    Menu,
    MessageSquare,
    Minimize,
    Minus,
    // ⭐ **Os faders VERTICAIS de uma mesa de mistura** — `"minus" < "mixer" < "modify"`,
    // e a posição do variante É o índice nas tabelas geradas.
    Mixer,
    Modify,
    More,
    MoreHorizontal,
    MoreVertical,
    MotionNodes,
    Open,
    Padding,
    Painter,
    Palette,
    Pan,
    Particle,
    PathClose,
    PathJoin,
    PathReverse,
    PathWeld,
    Pause,
    Physics,
    Pin,
    Pivot,
    Place,
    Play,
    Plus,
    Polygon,
    Prefab,
    Presentation,
    Probe,
    Rasterize,
    RealSize,
    Redo,
    Reset,
    Rigid,
    Rotate,
    RotateCcw,
    RotateCw,
    Save,
    Scale,
    Scene,
    Script,
    Search,
    Select,
    Settings,
    /// Sheet Packer — a folha e o arranjo (plano `docs/Sprite_projeto/17` §7).
    SkipBack,
    SkipForward,
    Smear,
    SnapMagnet,
    Spinner,
    SplitHorizontal,
    SplitVertical,
    Sprite,
    Step,
    Stop,
    Success,
    Tag,
    Text,
    Timer,
    Transform,
    Trash,
    TrimTransparency,
    Undo,
    Ungroup,
    Unlink,
    Unlock,
    Upscale,
    Vector,
    VectorBlend,
    VectorBucket,
    VectorBuild,
    VectorChamfer,
    VectorConnect,
    VectorFillet,
    VectorFrame,
    VectorNode,
    VectorPen,
    VectorPencil,
    VectorShape,
    VectorTrim,
    VectorWidth,
    VertexCorner,
    VertexSmooth,
    VertexSymmetric,
    Visible,
    Volume2,
    VolumeX,
    Vote,
    Warning,
    Zen,
}

impl IconId {
    /// Drawing commands for this icon, in paint order. Looked up in
    /// the codegen'd `ICON_CMDS_BY_ID` table by enum discriminant.
    ///
    /// **Invariant:** the variants in `enum IconId` must be declared
    /// in the SAME alphabetical order as `docs/design/icons/*.svg`
    /// filenames sort (which is the order `build.rs` emits). The
    /// `enum_order_matches_svgs` test below pins that contract — if
    /// you add a variant out of order, that test fails.
    pub fn cmds(self) -> &'static [IconCmd] {
        icons_generated::ICON_CMDS_BY_ID[self as usize]
    }

    /// Kebab-case slug matching the source SVG filename
    /// (`docs/design/icons/<slug>.svg`). Used by chrome derivation
    /// (manifest `icon_slug` cross-validation) + a11y label fallback.
    pub fn slug(self) -> &'static str {
        icons_generated::ALL_ICON_SLUGS[self as usize]
    }

    /// **Todo ícone canônico**, na ordem alfabética de slug — emitida pelo `build.rs`.
    ///
    /// ⚠️ Ela vem do gerador e **não é escrita à mão**: uma lista mantida aqui seria uma segunda
    /// cópia do enum, e ela drifta em silêncio no dia em que alguém acrescentar um variante sem a
    /// tocar. Do lado do gerador, um variante que falte faz o arquivo gerado **não compilar**.
    #[must_use]
    pub fn all() -> &'static [IconId] {
        icons_generated::ALL_ICON_IDS
    }

    /// **De volta ao ícone, a partir do SLUG** — a chave durável.
    ///
    /// ⚠️ **O DISCRIMINANTE NÃO É DURÁVEL, e é por isso que esta porta existe.** O
    /// `enum_order_matches_svgs` abaixo pina *ordem do enum == ordem alfabética dos SVGs*, então
    /// acrescentar `docs/design/icons/blob.svg` empurra **todo ícone depois de `blob`** uma casa —
    /// e um número guardado num documento passaria a nomear outro glifo, em silêncio. O slug é o
    /// nome do arquivo: ele não se move quando um vizinho nasce.
    ///
    /// `None` para um slug que este build não conhece — o mesmo caminho de compatibilidade do
    /// `WidgetKind::from_code`: um documento autorado por um build mais novo **degrada**, nunca
    /// recusa.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        icons_generated::ALL_ICON_SLUGS
            .iter()
            .position(|s| *s == slug)
            .map(|i| icons_generated::ALL_ICON_IDS[i])
    }
}

/// Convert a polyline-style point string ("x1 y1 x2 y2 ...") into an
/// SVG move/line path so it can flow through the same `from_svg`
/// pipeline. Ignores malformed input (returns empty path).
fn polyline_to_path(points: &str) -> BezPath {
    let mut path = BezPath::new();
    let mut nums = points
        .split_ascii_whitespace()
        .filter_map(|s| s.parse::<f64>().ok());
    if let (Some(x), Some(y)) = (nums.next(), nums.next()) {
        path.move_to((x, y));
        while let (Some(x), Some(y)) = (nums.next(), nums.next()) {
            path.line_to((x, y));
        }
    }
    path
}

/// Build a single [`BezPath`] approximating one [`IconCmd`].
///
/// All commands are emitted as paths so the consumer (the `paint_icon`
/// helper) can call `Scene::stroke` once per command with a uniform
/// brush. Circles and rounded rects use kurbo's path approximation,
/// which is dense enough at icon-rendering resolutions.
pub fn cmd_to_path(cmd: IconCmd) -> BezPath {
    use ph2d_vector::{Circle, Point, RoundedRect, Shape};
    match cmd {
        IconCmd::Path(d) => BezPath::from_svg(d).unwrap_or_default(),
        IconCmd::Polyline(p) => polyline_to_path(p),
        IconCmd::Line(x1, y1, x2, y2) => {
            let mut p = BezPath::new();
            p.move_to((x1 as f64, y1 as f64));
            p.line_to((x2 as f64, y2 as f64));
            p
        }
        IconCmd::Circle(cx, cy, r) => {
            // Tolerance is in viewbox units (24-unit grid). After
            // scaling to a 32-px icon the factor is ~1.33×, so a
            // 0.1 tolerance bleeds ~0.13 px of polygon facetting
            // into the stroke — visibly stepped on the larger
            // circles (LetterH/I, Settings: r=10). 0.005 is below
            // half a sub-pixel even on 2× SSAA pipelines and
            // produces a smoother cubic approximation; the extra
            // segments are free for Vello.
            Circle::new(Point::new(cx as f64, cy as f64), r as f64).into_path(0.005)
        }
        IconCmd::Rect(x, y, w, h, rx) => {
            let x0 = x as f64;
            let y0 = y as f64;
            let x1 = x0 + w as f64;
            let y1 = y0 + h as f64;
            if rx <= 0.0 {
                let mut p = BezPath::new();
                p.move_to((x0, y0));
                p.line_to((x1, y0));
                p.line_to((x1, y1));
                p.line_to((x0, y1));
                p.close_path();
                p
            } else {
                // Same reasoning as `Circle` above — tight tolerance
                // for the rounded corners.
                RoundedRect::new(x0, y0, x1, y1, rx as f64).into_path(0.005)
            }
        }
    }
}

#[cfg(test)]
#[path = "icons_tests.rs"]
mod tests;
