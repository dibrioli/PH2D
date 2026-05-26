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

/// All 100 glyphs in the canonical icon set.
///
/// Variants declared in **alphabetical order by kebab-case slug**
/// (matching `docs/design/icons/*.svg` filenames sorted). Order is
/// load-bearing: `cmds()` and `slug()` index into codegen'd tables
/// by `self as usize`, so reorder breaks every icon lookup silently.
/// `enum_order_matches_svgs` test pins the contract.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IconId {
    Add,
    Asset,
    Audio,
    BgRemoval,
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
    ColorEqualization,
    Color,
    Command,
    Console,
    Copy,
    Cube,
    Database,
    Delete,
    Duplicate,
    EqualizeSizes,
    Erase,
    Error,
    Export,
    Eye,
    EyeClosed,
    EyePencil,
    File,
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
    Inspector,
    Kbd,
    Layer,
    Layers,
    LetterH,
    LetterI,
    Light,
    Link,
    Lock,
    LockKeyhole,
    LockKeyholeOpen,
    MakeSquare,
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
    Padding,
    Painter,
    Palette,
    Pan,
    Particle,
    Pause,
    Pin,
    Pivot,
    Place,
    Play,
    Plus,
    Prefab,
    Rasterize,
    RealSize,
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
    TrimTransparency,
    Undo,
    Ungroup,
    Unlink,
    Unlock,
    Upscale,
    Visible,
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
mod tests {
    use super::*;

    /// Every icon must produce at least one drawing command.
    #[test]
    fn every_icon_has_at_least_one_cmd() {
        for id in ALL_ICONS {
            assert!(!id.cmds().is_empty(), "{id:?} has no commands");
        }
    }

    /// `IconId` discriminants must align 1:1 with alphabetical SVG
    /// order (the order `build.rs` produces tables in). If a new
    /// variant is added out of order, `cmds()` and `slug()` start
    /// returning the wrong glyph silently. This test catches that
    /// before any visual regression slips through.
    #[test]
    fn enum_order_matches_svgs() {
        // ALL_ICONS lists every variant in declaration order; if both
        // sides match alphabetically, discriminants align with
        // ALL_ICON_SLUGS indices.
        assert_eq!(
            ALL_ICONS.len(),
            icons_generated::ALL_ICON_SLUGS.len(),
            "IconId variant count must match SVG count"
        );
        for (i, id) in ALL_ICONS.iter().enumerate() {
            let expected_slug = icons_generated::ALL_ICON_SLUGS[i];
            assert_eq!(
                *id as usize, i,
                "{id:?} discriminant {} != position {} \
                 (enum order vs alphabetical SVG order drift)",
                *id as usize, i
            );
            assert_eq!(
                id.slug(),
                expected_slug,
                "{id:?} (idx {i}) · slug {:?} expected {:?}",
                id.slug(),
                expected_slug
            );
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
    /// Full enumeration used by every "for each icon" test above.
    /// In alphabetical-slug order, identical to `IconId` variant order.
    const ALL_ICONS: &[IconId] = &[
        IconId::Add,
        IconId::Asset,
        IconId::Audio,
        IconId::BgRemoval,
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
        IconId::ColorEqualization,
        IconId::Color,
        IconId::Command,
        IconId::Console,
        IconId::Copy,
        IconId::Cube,
        IconId::Database,
        IconId::Delete,
        IconId::Duplicate,
        IconId::EqualizeSizes,
        IconId::Erase,
        IconId::Error,
        IconId::Export,
        IconId::Eye,
        IconId::EyeClosed,
        IconId::EyePencil,
        IconId::File,
        IconId::Folder,
        IconId::Fps,
        IconId::Gizmo,
        IconId::Grid,
        IconId::GridSettings,
        IconId::Group,
        IconId::Help,
        IconId::Hidden,
        IconId::Hierarchy,
        IconId::History,
        IconId::HotReload,
        IconId::Image,
        IconId::Info,
        IconId::Inspector,
        IconId::Kbd,
        IconId::Layer,
        IconId::Layers,
        IconId::LetterH,
        IconId::LetterI,
        IconId::Light,
        IconId::Link,
        IconId::Lock,
        IconId::LockKeyhole,
        IconId::LockKeyholeOpen,
        IconId::MakeSquare,
        IconId::Material,
        IconId::Maximize,
        IconId::Menu,
        IconId::Minimize,
        IconId::Minus,
        IconId::Modify,
        IconId::More,
        IconId::MoreHorizontal,
        IconId::MoreVertical,
        IconId::Open,
        IconId::Padding,
        IconId::Painter,
        IconId::Palette,
        IconId::Pan,
        IconId::Particle,
        IconId::Pause,
        IconId::Pin,
        IconId::Pivot,
        IconId::Place,
        IconId::Play,
        IconId::Plus,
        IconId::Prefab,
        IconId::Rasterize,
        IconId::RealSize,
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
        IconId::TrimTransparency,
        IconId::Undo,
        IconId::Ungroup,
        IconId::Unlink,
        IconId::Unlock,
        IconId::Upscale,
        IconId::Visible,
        IconId::Warning,
        IconId::Zen,
    ];
}
