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
    Asset,
    Audio,
    Backdrop,
    BgRemoval,
    Blur,
    Bolt,
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
    Eyedropper,
    File,
    FitView,
    Flip,
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
    Mask,
    Material,
    Maximize,
    Menu,
    Minimize,
    Minus,
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
    Pause,
    Physics,
    Pin,
    Pivot,
    Place,
    Play,
    Plus,
    Polygon,
    Prefab,
    Probe,
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
    SkipBack,
    SkipForward,
    Smear,
    Spinner,
    SplitHorizontal,
    SplitVertical,
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
    Vector,
    VectorPen,
    VectorPencil,
    VectorShape,
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
mod tests {
    use super::*;

    /// **O slug ida-e-volta é TOTAL, e um desconhecido degrada.**
    ///
    /// ⚠️ Ele é o gate da chave durável: sem ele, um `from_slug` que devolvesse o ícone da posição
    /// errada pintaria outro glifo com toda a suíte verde — a lista e os slugs são indexados pelo
    /// MESMO número, então uma mistura entre eles é invisível a olho.
    #[test]
    fn the_slug_round_trip_is_total_and_the_unknown_degrades() {
        assert_eq!(IconId::all().len(), icons_generated::ALL_ICON_SLUGS.len());
        for id in IconId::all() {
            assert_eq!(
                IconId::from_slug(id.slug()),
                Some(*id),
                "{id:?} nao voltou do proprio slug ({})",
                id.slug()
            );
        }
        assert_eq!(IconId::from_slug("nao-existe"), None);
        assert_eq!(IconId::from_slug(""), None);
    }

    /// **O NOME de cada variante é a PascalCase do slug dele** — e isto não é estilo.
    ///
    /// # O defeito VIVO que este gate encontrou (2026-08-09)
    ///
    /// O `enum_order_matches_svgs` ao lado pina `*id as usize == i`, que é **trivialmente
    /// verdadeiro para qualquer lista em ordem de declaração** — ele prova que a lista está
    /// completa, nunca que cada variante está no lugar certo. E dois não estavam: o enum declarava
    /// `ColorEqualization, Color` sobre os slugs `color, color-equalization`, então
    /// **`IconId::Color` desenhava os sliders e `IconId::ColorEqualization` desenhava a paleta**.
    ///
    /// ⚠️ **Com consequência de produto:** o botão *"Add adjustment"* do painel de camadas do
    /// Painter pede `ColorEqualization` — e recebia a PALETA. Um ano de screenshots com o glifo
    /// errado, e nenhum teste a falhar, porque o único gate media a contagem e a ordem.
    ///
    /// ⚠️ E é ele que torna o `ALL_ICON_IDS` do `build.rs` **provado** em vez de meramente
    /// mecânico: o gerador impede um variante AUSENTE (o arquivo não compila) e é cego a um
    /// variante MAL-NOMEADO — que é exactamente a forma que este defeito tinha.
    #[test]
    fn every_variant_is_named_after_its_slug() {
        let pascal = |slug: &str| -> String {
            slug.split('-')
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        None => String::new(),
                    }
                })
                .collect()
        };
        for id in IconId::all() {
            assert_eq!(
                format!("{id:?}"),
                pascal(id.slug()),
                "o variante {id:?} esta' na posicao do slug {:?} — ele desenha OUTRO glifo",
                id.slug()
            );
        }
    }

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
        // ⚠️ **As duas listas têm papéis DIFERENTES, e é por isso que ambas existem.** Esta,
        // escrita à mão a partir do ENUM, é o que apanha um variante que ninguém desenhou (o
        // gerador só conhece os SVGs, então um variante a mais é invisível para ele); a gerada é
        // o que apanha um SVG que ninguém declarou (o arquivo não compila). Esta linha é o que
        // as ata — sem ela, cada uma vigiaria metade e nenhuma diria que discordam.
        assert_eq!(
            ALL_ICONS,
            IconId::all(),
            "a lista escrita a mao e a gerada discordam"
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
        IconId::Backdrop,
        IconId::BgRemoval,
        IconId::Blur,
        IconId::Bolt,
        IconId::Bug,
        IconId::Build,
        IconId::Camera,
        IconId::Check,
        IconId::ChevronDown,
        IconId::ChevronLeft,
        IconId::ChevronRight,
        IconId::ChevronUp,
        IconId::Circle,
        IconId::Clone,
        IconId::Close,
        IconId::Cmd,
        IconId::Collider,
        IconId::Color,
        IconId::ColorEqualization,
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
        IconId::Eyedropper,
        IconId::File,
        IconId::FitView,
        IconId::Flip,
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
        IconId::Inpaint,
        IconId::Inspector,
        IconId::Kbd,
        IconId::Knife,
        IconId::Layer,
        IconId::Layers,
        IconId::LetterH,
        IconId::LetterI,
        IconId::Light,
        IconId::Line,
        IconId::Link,
        IconId::Liquify,
        IconId::Lock,
        IconId::LockKeyhole,
        IconId::LockKeyholeOpen,
        IconId::MakeSquare,
        IconId::Mask,
        IconId::Material,
        IconId::Maximize,
        IconId::Menu,
        IconId::Minimize,
        IconId::Minus,
        IconId::Modify,
        IconId::More,
        IconId::MoreHorizontal,
        IconId::MoreVertical,
        IconId::MotionNodes,
        IconId::Open,
        IconId::Padding,
        IconId::Painter,
        IconId::Palette,
        IconId::Pan,
        IconId::Particle,
        IconId::Pause,
        IconId::Physics,
        IconId::Pin,
        IconId::Pivot,
        IconId::Place,
        IconId::Play,
        IconId::Plus,
        IconId::Polygon,
        IconId::Prefab,
        IconId::Probe,
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
        IconId::SkipBack,
        IconId::SkipForward,
        IconId::Smear,
        IconId::Spinner,
        IconId::SplitHorizontal,
        IconId::SplitVertical,
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
        IconId::Vector,
        IconId::VectorPen,
        IconId::VectorPencil,
        IconId::VectorShape,
        IconId::Visible,
        IconId::Warning,
        IconId::Zen,
    ];
}
