//! Icon glyph table (Phase 0.1 of the M13 UI sprint).
//!
//! 89 glyphs ported from `docs/design/icons/*.svg`. Each icon is a
//! list of [`IconCmd`] primitives (paths, polylines, lines, circles,
//! rounded rects) on a 24x24 viewBox. The paint helper strokes them
//! with `currentColor` semantics: caller passes the color, we apply
//! it to every command.
//!
//! Source SVGs are Lucide-derived (ISC license). They use
//! `fill="none" stroke="currentColor" stroke-width="1.5"` —
//! we replicate that contract here.

use ph2d_vector::BezPath;

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

/// All 89 glyphs in the canonical icon set.
///
/// Names mirror the source SVG filenames (kebab-case → CamelCase).
/// Order is alphabetical for grep-friendliness.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IconId {
    Add,
    Asset,
    Audio,
    Bolt,
    Bug,
    Build,
    Camera,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    Close,
    Cmd,
    Collider,
    Color,
    Command,
    Console,
    Copy,
    Cube,
    Database,
    Delete,
    Duplicate,
    Erase,
    Error,
    Export,
    EyePencil,
    File,
    Folder,
    Fps,
    Gizmo,
    Grid,
    Help,
    Hidden,
    Hierarchy,
    History,
    HotReload,
    Info,
    Inspector,
    Kbd,
    Layer,
    Layers,
    Light,
    Link,
    Lock,
    Material,
    Maximize,
    Menu,
    Minimize,
    Minus,
    Modify,
    More,
    MoreHorizontal,
    MoreVertical,
    Open,
    Pan,
    Particle,
    Pause,
    Pin,
    Pivot,
    Place,
    Play,
    Plus,
    Prefab,
    Redo,
    Reset,
    Rigid,
    Rotate,
    Save,
    Scale,
    Scene,
    Script,
    Search,
    Select,
    Settings,
    Spinner,
    Sprite,
    Step,
    Stop,
    Success,
    Tag,
    Text,
    Transform,
    Trash,
    Undo,
    Unlink,
    Unlock,
    Visible,
    Warning,
    Zen,
}

impl IconId {
    /// Drawing commands for this icon, in paint order.
    pub fn cmds(self) -> &'static [IconCmd] {
        match self {
            Self::Add => &[IconCmd::Path("M12 5v14M5 12h14")],
            Self::Asset => &[
                IconCmd::Rect(3.0, 3.0, 18.0, 18.0, 2.0),
                IconCmd::Path("M3 14l4-4 4 4 6-6 4 4"),
            ],
            Self::Audio => &[
                IconCmd::Path("M9 18V5l12-2v13"),
                IconCmd::Circle(6.0, 18.0, 3.0),
                IconCmd::Circle(18.0, 16.0, 3.0),
            ],
            Self::Bolt => &[IconCmd::Path("M13 2L3 14h9l-1 8 10-12h-9l1-8z")],
            Self::Bug => &[
                IconCmd::Rect(8.0, 6.0, 8.0, 14.0, 4.0),
                IconCmd::Path("M19 7l-3 2M5 7l3 2M19 13h-3M5 13h3M19 19l-3-2M5 19l3-2M12 2v4"),
            ],
            Self::Build => &[
                IconCmd::Path("M14.7 6.3a4 4 0 0 1 5.6 5.6L11 21H3v-8z"),
                IconCmd::Path("M13.3 7.7l3 3"),
            ],
            Self::Camera => &[
                IconCmd::Path(
                    "M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3z",
                ),
                IconCmd::Circle(12.0, 13.0, 4.0),
            ],
            Self::Check => &[IconCmd::Polyline("20 6 9 17 4 12")],
            Self::ChevronDown => &[IconCmd::Polyline("6 9 12 15 18 9")],
            Self::ChevronLeft => &[IconCmd::Polyline("15 18 9 12 15 6")],
            Self::ChevronRight => &[IconCmd::Polyline("9 18 15 12 9 6")],
            Self::ChevronUp => &[IconCmd::Polyline("18 15 12 9 6 15")],
            Self::Close => &[
                IconCmd::Line(18.0, 6.0, 6.0, 18.0),
                IconCmd::Line(6.0, 6.0, 18.0, 18.0),
            ],
            Self::Cmd => &[IconCmd::Path(
                "M6 9a3 3 0 1 1 3-3v12a3 3 0 1 1-3-3h12a3 3 0 1 1-3 3V6a3 3 0 1 1 3 3z",
            )],
            Self::Collider => &[IconCmd::Rect(3.0, 3.0, 18.0, 18.0, 2.0)],
            Self::Color => &[
                IconCmd::Circle(13.5, 6.5, 2.5),
                IconCmd::Circle(17.5, 10.5, 2.5),
                IconCmd::Circle(8.5, 7.5, 2.5),
                IconCmd::Circle(6.5, 12.5, 2.5),
                IconCmd::Path(
                    "M12 2a10 10 0 1 0 0 20 1.5 1.5 0 0 0 1-2.6 1.5 1.5 0 0 1 1-2.6h2.5a4.5 4.5 0 0 0 4.5-4.5A10 10 0 0 0 12 2z",
                ),
            ],
            Self::Command => &[IconCmd::Path(
                "M18 3a3 3 0 0 0-3 3v12a3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3H6a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3V6a3 3 0 0 0-3-3 3 3 0 0 0-3 3 3 3 0 0 0 3 3h12a3 3 0 0 0 3-3 3 3 0 0 0-3-3z",
            )],
            Self::Console => &[IconCmd::Path("M4 17l6-6-6-6M12 19h8")],
            Self::Copy => &[
                IconCmd::Rect(9.0, 9.0, 13.0, 13.0, 2.0),
                IconCmd::Path("M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"),
            ],
            Self::Cube => &[IconCmd::Path(
                "M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z",
            )],
            Self::Database => &[
                IconCmd::Path("M3 5v6c0 1.7 4 3 9 3s9-1.3 9-3V5"),
                IconCmd::Path("M3 11v6c0 1.7 4 3 9 3s9-1.3 9-3v-6"),
            ],
            Self::Delete => &[
                IconCmd::Polyline("3 6 5 6 21 6"),
                IconCmd::Path(
                    "M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6M10 11v6M14 11v6M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2",
                ),
            ],
            Self::Duplicate => &[
                IconCmd::Rect(8.0, 8.0, 13.0, 13.0, 2.0),
                IconCmd::Path("M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"),
            ],
            Self::Erase => &[
                IconCmd::Path(
                    "M21 21H8l-5-5a2 2 0 0 1 0-2.8L13.6 2.6a2 2 0 0 1 2.8 0l5 5a2 2 0 0 1 0 2.8L11 21",
                ),
                IconCmd::Path("M9 11l6 6"),
            ],
            Self::Error => &[
                IconCmd::Circle(12.0, 12.0, 10.0),
                IconCmd::Line(15.0, 9.0, 9.0, 15.0),
                IconCmd::Line(9.0, 9.0, 15.0, 15.0),
            ],
            Self::Export => &[
                IconCmd::Path("M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"),
                IconCmd::Path("M7 10l5-5 5 5"),
                IconCmd::Line(12.0, 5.0, 12.0, 15.0),
            ],
            Self::EyePencil => &[
                IconCmd::Path("M12 19l7-7 3 3-7 7H12v-3z"),
                IconCmd::Path("M18 13l-1.5-7.5L2 2l3.5 14.5L13 18z"),
                IconCmd::Path("M2 2l7.6 7.6"),
                IconCmd::Circle(11.0, 11.0, 2.0),
            ],
            Self::File => &[
                IconCmd::Path("M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"),
                IconCmd::Polyline("14 2 14 8 20 8"),
            ],
            Self::Folder => &[IconCmd::Path(
                "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z",
            )],
            Self::Fps => &[IconCmd::Polyline("22 12 18 12 15 21 9 3 6 12 2 12")],
            Self::Gizmo => &[
                IconCmd::Path("M12 12L4 4M12 12l8-8M12 12L4 20M12 12l8 8"),
                IconCmd::Circle(12.0, 12.0, 2.0),
            ],
            Self::Grid => &[
                IconCmd::Rect(3.0, 3.0, 18.0, 18.0, 1.0),
                IconCmd::Path("M3 9h18M3 15h18M9 3v18M15 3v18"),
            ],
            Self::Help => &[
                IconCmd::Circle(12.0, 12.0, 10.0),
                IconCmd::Path("M9.1 9a3 3 0 1 1 5.8 1c0 2-3 3-3 3"),
                IconCmd::Line(12.0, 17.0, 12.01, 17.0),
            ],
            Self::Hidden => &[
                IconCmd::Path(
                    "M17.9 17.9A11 11 0 0 1 12 20c-7 0-11-8-11-8a20 20 0 0 1 5.1-5.9M9.9 5A11 11 0 0 1 12 4c7 0 11 8 11 8a20 20 0 0 1-3.2 4.2M14.1 14.1a3 3 0 1 1-4.2-4.2",
                ),
                IconCmd::Line(1.0, 1.0, 23.0, 23.0),
            ],
            Self::Hierarchy => &[
                IconCmd::Rect(3.0, 3.0, 6.0, 6.0, 1.0),
                IconCmd::Rect(15.0, 3.0, 6.0, 6.0, 1.0),
                IconCmd::Rect(9.0, 15.0, 6.0, 6.0, 1.0),
                IconCmd::Path("M6 9v3h12V9M12 12v3"),
            ],
            Self::History => &[
                IconCmd::Path("M3 3v5h5"),
                IconCmd::Path("M3.05 13A9 9 0 1 0 6 5.3L3 8"),
                IconCmd::Path("M12 7v5l4 2"),
            ],
            Self::HotReload => &[IconCmd::Path(
                "M14.5 5l4 4M3 21l4.5-1L20 7.5 16.5 4 4 16.5 3 21z",
            )],
            Self::Info => &[
                IconCmd::Circle(12.0, 12.0, 10.0),
                IconCmd::Line(12.0, 16.0, 12.0, 12.0),
                IconCmd::Line(12.0, 8.0, 12.01, 8.0),
            ],
            Self::Inspector => &[
                IconCmd::Circle(11.0, 11.0, 8.0),
                IconCmd::Path("M21 21l-4.35-4.35M11 8v6M8 11h6"),
            ],
            Self::Kbd => &[
                IconCmd::Rect(2.0, 6.0, 20.0, 12.0, 2.0),
                IconCmd::Path("M6 10h.01M10 10h.01M14 10h.01M18 10h.01M6 14h12"),
            ],
            Self::Layer => &[
                IconCmd::Path("M12 2L2 7l10 5 10-5z"),
                IconCmd::Path("M2 17l10 5 10-5M2 12l10 5 10-5"),
            ],
            Self::Layers => &[
                IconCmd::Path("M12 2L2 7l10 5 10-5-10-5z"),
                IconCmd::Path("M2 17l10 5 10-5M2 12l10 5 10-5"),
            ],
            Self::Light => &[IconCmd::Path(
                "M9 18h6M10 22h4M12 2a7 7 0 0 0-4 12.74V17a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1v-2.26A7 7 0 0 0 12 2z",
            )],
            Self::Link => &[
                IconCmd::Path("M10 13a5 5 0 0 0 7 0l3-3a5 5 0 0 0-7-7l-1 1"),
                IconCmd::Path("M14 11a5 5 0 0 0-7 0l-3 3a5 5 0 0 0 7 7l1-1"),
            ],
            Self::Lock => &[
                IconCmd::Rect(3.0, 11.0, 18.0, 11.0, 2.0),
                IconCmd::Path("M7 11V7a5 5 0 0 1 10 0v4"),
            ],
            Self::Material => &[
                IconCmd::Circle(12.0, 12.0, 9.0),
                IconCmd::Path("M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"),
            ],
            Self::Maximize => &[IconCmd::Path(
                "M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3",
            )],
            Self::Menu => &[
                IconCmd::Line(3.0, 6.0, 21.0, 6.0),
                IconCmd::Line(3.0, 12.0, 21.0, 12.0),
                IconCmd::Line(3.0, 18.0, 21.0, 18.0),
            ],
            Self::Minimize => &[IconCmd::Path(
                "M8 3v3a2 2 0 0 1-2 2H3M21 8h-3a2 2 0 0 1-2-2V3M3 16h3a2 2 0 0 1 2 2v3M16 21v-3a2 2 0 0 1 2-2h3",
            )],
            Self::Minus => &[IconCmd::Path("M5 12h14")],
            Self::Modify => &[
                IconCmd::Rect(3.0, 3.0, 18.0, 18.0, 3.0),
                IconCmd::Circle(12.0, 12.0, 4.0),
            ],
            Self::More => &[
                IconCmd::Circle(5.0, 12.0, 1.5),
                IconCmd::Circle(12.0, 12.0, 1.5),
                IconCmd::Circle(19.0, 12.0, 1.5),
            ],
            Self::MoreHorizontal => &[
                IconCmd::Circle(12.0, 12.0, 1.0),
                IconCmd::Circle(19.0, 12.0, 1.0),
                IconCmd::Circle(5.0, 12.0, 1.0),
            ],
            Self::MoreVertical => &[
                IconCmd::Circle(12.0, 12.0, 1.0),
                IconCmd::Circle(12.0, 5.0, 1.0),
                IconCmd::Circle(12.0, 19.0, 1.0),
            ],
            Self::Open => &[IconCmd::Path(
                "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
            )],
            Self::Pan => &[
                IconCmd::Path("M9 11V5a2 2 0 1 1 4 0v6"),
                IconCmd::Path("M13 11V4a2 2 0 1 1 4 0v8"),
                IconCmd::Path(
                    "M17 12V7a2 2 0 1 1 4 0v9a6 6 0 0 1-6 6h-2a8 8 0 0 1-8-8v-3a2 2 0 1 1 4 0",
                ),
            ],
            Self::Particle => &[
                IconCmd::Circle(12.0, 12.0, 1.0),
                IconCmd::Circle(6.0, 6.0, 1.5),
                IconCmd::Circle(18.0, 6.0, 1.0),
                IconCmd::Circle(6.0, 18.0, 1.0),
                IconCmd::Circle(18.0, 18.0, 1.5),
                IconCmd::Circle(20.0, 12.0, 1.0),
                IconCmd::Circle(4.0, 12.0, 1.0),
            ],
            Self::Pause => &[
                IconCmd::Rect(6.0, 4.0, 4.0, 16.0, 0.0),
                IconCmd::Rect(14.0, 4.0, 4.0, 16.0, 0.0),
            ],
            Self::Pin => &[IconCmd::Path("M12 2v8M12 22v-4M5 9l7-7 7 7M9 18h6")],
            Self::Pivot => &[
                IconCmd::Circle(12.0, 12.0, 2.0),
                IconCmd::Path("M12 2v6M12 16v6M2 12h6M16 12h6"),
            ],
            Self::Place => &[
                IconCmd::Circle(12.0, 12.0, 3.0),
                IconCmd::Path(
                    "M12 2v4M12 18v4M2 12h4M18 12h4M5.6 5.6l2.8 2.8M15.6 15.6l2.8 2.8M5.6 18.4l2.8-2.8M15.6 8.4l2.8-2.8",
                ),
            ],
            Self::Play => &[IconCmd::Path("M5 3l16 9-16 9z")],
            Self::Plus => &[
                IconCmd::Line(12.0, 5.0, 12.0, 19.0),
                IconCmd::Line(5.0, 12.0, 19.0, 12.0),
            ],
            Self::Prefab => &[
                IconCmd::Path(
                    "M21 16V8a2 2 0 0 0-1-1.7l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.7l7 4a2 2 0 0 0 2 0l7-4a2 2 0 0 0 1-1.7z",
                ),
                IconCmd::Polyline("3.3 7 12 12 20.7 7"),
                IconCmd::Line(12.0, 22.0, 12.0, 12.0),
            ],
            Self::Redo => &[
                IconCmd::Path("M21 7v6h-6"),
                IconCmd::Path("M3 17a9 9 0 0 1 15-6.7L21 13"),
            ],
            Self::Reset => &[
                IconCmd::Path("M3 12a9 9 0 1 0 3-6.7L3 8"),
                IconCmd::Path("M3 3v5h5"),
            ],
            Self::Rigid => &[IconCmd::Path(
                "M12 2v6M12 22v-6M4.93 4.93l4.24 4.24M14.83 14.83l4.24 4.24M2 12h6M16 12h6M4.93 19.07l4.24-4.24M14.83 9.17l4.24-4.24",
            )],
            Self::Rotate => &[
                IconCmd::Path("M21 12a9 9 0 1 1-3.05-6.74"),
                IconCmd::Path("M21 4v5h-5"),
            ],
            Self::Save => &[
                IconCmd::Path("M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"),
                IconCmd::Path("M17 21v-8H7v8M7 3v5h8"),
            ],
            Self::Scale => &[IconCmd::Path(
                "M3 17l4 4M3 21h4v-4M21 7l-4-4M21 3h-4v4M3 3l18 18",
            )],
            Self::Scene => &[
                IconCmd::Rect(2.0, 3.0, 20.0, 14.0, 2.0),
                IconCmd::Line(8.0, 21.0, 16.0, 21.0),
                IconCmd::Line(12.0, 17.0, 12.0, 21.0),
            ],
            Self::Script => &[
                IconCmd::Path("M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"),
                IconCmd::Polyline("14 2 14 8 20 8"),
                IconCmd::Path("M9 13l-2 2 2 2M13 13l2 2-2 2M11 17l1-4"),
            ],
            Self::Search => &[
                IconCmd::Circle(11.0, 11.0, 7.0),
                IconCmd::Line(21.0, 21.0, 16.7, 16.7),
            ],
            Self::Select => &[IconCmd::Path("M3 3l7 17 2.5-7.5L20 10z")],
            Self::Settings => &[
                IconCmd::Path("M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"),
                IconCmd::Path(
                    "M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3h.1a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8v.1a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
                ),
            ],
            Self::Spinner => &[IconCmd::Path("M21 12a9 9 0 1 1-6.2-8.5")],
            Self::Sprite => &[
                IconCmd::Rect(3.0, 3.0, 18.0, 18.0, 2.0),
                IconCmd::Circle(9.0, 9.0, 2.0),
                IconCmd::Path("M21 15l-5-5-9 9"),
            ],
            Self::Step => &[
                IconCmd::Path("M5 4l8 8-8 8z"),
                IconCmd::Line(19.0, 4.0, 19.0, 20.0),
            ],
            Self::Stop => &[IconCmd::Rect(5.0, 5.0, 14.0, 14.0, 1.0)],
            Self::Success => &[
                IconCmd::Circle(12.0, 12.0, 10.0),
                IconCmd::Polyline("8 12 11 15 16 9"),
            ],
            Self::Tag => &[
                IconCmd::Path(
                    "M20.6 13.4 13.4 20.6a2 2 0 0 1-2.8 0L2 12V2h10l8.6 8.6a2 2 0 0 1 0 2.8z",
                ),
                IconCmd::Line(7.0, 7.0, 7.01, 7.0),
            ],
            Self::Text => &[IconCmd::Path("M4 7V5h16v2M9 19h6M12 5v14")],
            Self::Transform => &[IconCmd::Path(
                "M12 2v20M2 12h20M8 6l4-4 4 4M8 18l4 4 4-4M6 8L2 12l4 4M18 8l4 4-4 4",
            )],
            Self::Trash => &[IconCmd::Path(
                "M3 6h18M19 6l-1.5 14a2 2 0 0 1-2 1.84H8.5a2 2 0 0 1-2-1.84L5 6M10 11v6M14 11v6M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2",
            )],
            Self::Undo => &[
                IconCmd::Path("M3 7v6h6"),
                IconCmd::Path("M21 17a9 9 0 0 0-15-6.7L3 13"),
            ],
            Self::Unlink => &[IconCmd::Path(
                "M18.8 13.2L14 18a5 5 0 0 1-7-7l1-1M5.2 10.8L10 6a5 5 0 0 1 7 7l-1 1M2 2l20 20",
            )],
            Self::Unlock => &[
                IconCmd::Rect(3.0, 11.0, 18.0, 11.0, 2.0),
                IconCmd::Path("M7 11V7a5 5 0 0 1 9.9-1"),
            ],
            Self::Visible => &[
                IconCmd::Path("M1 12s4-8 11-8 11 8 11 8-4 8-11 8S1 12 1 12z"),
                IconCmd::Circle(12.0, 12.0, 3.0),
            ],
            Self::Warning => &[
                IconCmd::Path(
                    "M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z",
                ),
                IconCmd::Line(12.0, 9.0, 12.0, 13.0),
                IconCmd::Line(12.0, 17.0, 12.01, 17.0),
            ],
            Self::Zen => &[IconCmd::Path(
                "M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3",
            )],
        }
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
            Circle::new(Point::new(cx as f64, cy as f64), r as f64).into_path(0.1)
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
                RoundedRect::new(x0, y0, x1, y1, rx as f64).into_path(0.1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every icon must produce at least one drawing command.
    #[test]
    fn every_icon_has_at_least_one_cmd() {
        for id in ALL_ICONS {
            assert!(!id.cmds().is_empty(), "{id:?} has no commands");
        }
    }

    /// Every Path command parses cleanly via kurbo's SVG reader.
    #[test]
    fn every_path_string_parses() {
        for id in ALL_ICONS {
            for (i, cmd) in id.cmds().iter().enumerate() {
                if let IconCmd::Path(d) = cmd {
                    BezPath::from_svg(d).unwrap_or_else(|e| {
                        panic!("{id:?} cmd[{i}] = {d:?} failed to parse: {e:?}")
                    });
                }
            }
        }
    }

    /// Every Polyline command yields a path with at least one segment.
    #[test]
    fn every_polyline_has_segments() {
        for id in ALL_ICONS {
            for cmd in id.cmds() {
                if let IconCmd::Polyline(p) = cmd {
                    let path = polyline_to_path(p);
                    assert!(
                        path.elements().len() >= 2,
                        "{id:?} polyline {p:?} produced too few elements"
                    );
                }
            }
        }
    }

    /// `cmd_to_path` returns a non-empty path for every command in
    /// every icon.
    #[test]
    fn cmd_to_path_is_non_empty_for_all() {
        for id in ALL_ICONS {
            for cmd in id.cmds() {
                let path = cmd_to_path(*cmd);
                assert!(
                    !path.elements().is_empty(),
                    "{id:?} produced empty path for {cmd:?}"
                );
            }
        }
    }

    /// Hand-pick five icons from different shape families and verify
    /// that we can compose them into a single [`BezPath`] for the
    /// scene without panicking. This is the "smoke render" the
    /// 0.1 acceptance asks for, minus the actual GPU step.
    #[test]
    fn smoke_render_five_icons() {
        let picks = [
            IconId::Add,
            IconId::Check,
            IconId::Save,
            IconId::Settings,
            IconId::Sprite,
        ];
        for id in picks {
            let mut combined = BezPath::new();
            for cmd in id.cmds() {
                combined.extend(cmd_to_path(*cmd));
            }
            assert!(!combined.elements().is_empty());
        }
    }

    /// Full enumeration used by every "for each icon" test above.
    /// Lives at module scope so adding a new icon means adding
    /// exactly one entry both to `IconId` and to `ALL_ICONS`.
    const ALL_ICONS: &[IconId] = &[
        IconId::Add,
        IconId::Asset,
        IconId::Audio,
        IconId::Bolt,
        IconId::Bug,
        IconId::Build,
        IconId::Camera,
        IconId::Check,
        IconId::ChevronDown,
        IconId::ChevronLeft,
        IconId::ChevronRight,
        IconId::ChevronUp,
        IconId::Close,
        IconId::Cmd,
        IconId::Collider,
        IconId::Color,
        IconId::Command,
        IconId::Console,
        IconId::Copy,
        IconId::Cube,
        IconId::Database,
        IconId::Delete,
        IconId::Duplicate,
        IconId::Erase,
        IconId::Error,
        IconId::Export,
        IconId::EyePencil,
        IconId::File,
        IconId::Folder,
        IconId::Fps,
        IconId::Gizmo,
        IconId::Grid,
        IconId::Help,
        IconId::Hidden,
        IconId::Hierarchy,
        IconId::History,
        IconId::HotReload,
        IconId::Info,
        IconId::Inspector,
        IconId::Kbd,
        IconId::Layer,
        IconId::Layers,
        IconId::Light,
        IconId::Link,
        IconId::Lock,
        IconId::Material,
        IconId::Maximize,
        IconId::Menu,
        IconId::Minimize,
        IconId::Modify,
        IconId::More,
        IconId::MoreHorizontal,
        IconId::MoreVertical,
        IconId::Open,
        IconId::Pan,
        IconId::Particle,
        IconId::Pause,
        IconId::Pin,
        IconId::Pivot,
        IconId::Place,
        IconId::Play,
        IconId::Plus,
        IconId::Prefab,
        IconId::Redo,
        IconId::Reset,
        IconId::Rigid,
        IconId::Rotate,
        IconId::Save,
        IconId::Scale,
        IconId::Scene,
        IconId::Script,
        IconId::Search,
        IconId::Select,
        IconId::Settings,
        IconId::Spinner,
        IconId::Sprite,
        IconId::Step,
        IconId::Stop,
        IconId::Success,
        IconId::Tag,
        IconId::Text,
        IconId::Transform,
        IconId::Trash,
        IconId::Undo,
        IconId::Unlink,
        IconId::Unlock,
        IconId::Visible,
        IconId::Warning,
        IconId::Zen,
    ];
}
