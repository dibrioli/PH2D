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
                BezPath::from_svg(d)
                    .unwrap_or_else(|e| panic!("{id:?} cmd[{i}] = {d:?} failed to parse: {e:?}"));
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
    IconId::AlignBottom,
    IconId::AlignCenter,
    IconId::AlignLeft,
    IconId::AlignMiddle,
    IconId::AlignRight,
    IconId::AlignTop,
    IconId::ArrangeBack,
    IconId::ArrangeBackward,
    IconId::ArrangeForward,
    IconId::ArrangeFront,
    IconId::Asset,
    IconId::Audio,
    IconId::Backdrop,
    IconId::BgRemoval,
    IconId::Blur,
    IconId::Bolt,
    IconId::Bone,
    IconId::BoolExclude,
    IconId::BoolIntersect,
    IconId::BoolSubtract,
    IconId::BoolUnion,
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
    IconId::ConvertCurves,
    IconId::Copy,
    IconId::Cube,
    IconId::Database,
    IconId::Delete,
    IconId::DistributeH,
    IconId::DistributeV,
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
    IconId::Film,
    IconId::FitView,
    IconId::Flip,
    IconId::FlipH,
    IconId::FlipV,
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
    IconId::LayoutGrid,
    IconId::LetterH,
    IconId::LetterI,
    IconId::Light,
    IconId::Line,
    IconId::Link,
    IconId::Liquify,
    IconId::Lock,
    IconId::LockKeyhole,
    IconId::LockKeyholeOpen,
    IconId::Macro,
    IconId::MakeSquare,
    IconId::MarqueeBox,
    IconId::MarqueeLasso,
    IconId::Mask,
    IconId::Material,
    IconId::Maximize,
    IconId::Menu,
    IconId::MessageSquare,
    IconId::Minimize,
    IconId::Minus,
    IconId::Mixer,
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
    IconId::PathClose,
    IconId::PathJoin,
    IconId::PathReverse,
    IconId::PathSharpen,
    IconId::PathSimplify,
    IconId::PathSmooth,
    IconId::PathSubdivide,
    IconId::PathWeld,
    IconId::Pause,
    IconId::Physics,
    IconId::Pin,
    IconId::Pivot,
    IconId::Place,
    IconId::Play,
    IconId::Plus,
    IconId::Polygon,
    IconId::Prefab,
    IconId::Presentation,
    IconId::Probe,
    IconId::Rasterize,
    IconId::RealSize,
    IconId::Redo,
    IconId::Reset,
    IconId::Rigid,
    IconId::Rotate,
    IconId::RotateCcw,
    IconId::RotateCw,
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
    IconId::SnapMagnet,
    IconId::Spinner,
    IconId::SplitHorizontal,
    IconId::SplitVertical,
    IconId::Sprite,
    IconId::Step,
    IconId::Stop,
    IconId::Success,
    IconId::Tag,
    IconId::Text,
    IconId::Timer,
    IconId::Transform,
    IconId::Trash,
    IconId::TrimTransparency,
    IconId::Undo,
    IconId::Ungroup,
    IconId::Unlink,
    IconId::Unlock,
    IconId::Upscale,
    IconId::Vector,
    IconId::VectorBlend,
    IconId::VectorBucket,
    IconId::VectorBuild,
    IconId::VectorChamfer,
    IconId::VectorConnect,
    IconId::VectorFillet,
    IconId::VectorFrame,
    IconId::VectorNode,
    IconId::VectorPen,
    IconId::VectorPencil,
    IconId::VectorShape,
    IconId::VectorTrim,
    IconId::VectorWidth,
    IconId::VertexCorner,
    IconId::VertexSmooth,
    IconId::VertexSymmetric,
    IconId::Visible,
    IconId::Volume2,
    IconId::VolumeX,
    IconId::Vote,
    IconId::Warning,
    IconId::Zen,
];
