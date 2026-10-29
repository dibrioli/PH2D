//! As TABELAS do rail esquerdo do Painter — as ferramentas, o flyout da Máscara e o das Formas — partidas do
//! [`super`] (tecto de tamanho); a pintura e o registo continuam lá.

use crate::icons::IconId;
use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_i18n::TextKey;

/// Uma ferramenta do rail: `(id, NOME, ícone, SUB-RÓTULO vertical)` — os dois textos são CHAVES
/// (`chrome.rail.*` / `chrome.rail.sub.*`), traduzidas no [`super::tool_entry`], que é quem as pinta.
pub(in crate::screens::hero) type RailTool = (NodeId, TextKey, IconId, TextKey);

/// Painter-mode paint tools, in rail order: `(id, a11y label, icon, vertical
/// sub-label)`. Replaces the object-mode transform block while the Painter tool
/// is active. Exclusive radio selection (`chrome/rail_painter_tools.rs`); the
/// last entry (Shapes) also owns the shape flyout ([`PAINTER_SHAPES`]). The water's tools and the Digital
/// Smear/Blur are never both on the rail ([`super::no_trilho`]).
pub(in crate::screens::hero) const PAINTER_TOOLS: [RailTool; 16] = [
    (
        ids::PAINTER_RAIL_BRUSH,
        TextKey::new("chrome.rail.brush"),
        IconId::Painter,
        TextKey::new("chrome.rail.sub.brush"),
    ),
    (
        ids::PAINTER_RAIL_EYEDROPPER,
        TextKey::new("chrome.rail.eyedropper"),
        IconId::Eyedropper,
        TextKey::new("chrome.rail.sub.pick"),
    ),
    (
        ids::PAINTER_RAIL_ERASER,
        TextKey::new("chrome.rail.eraser"),
        IconId::Erase,
        TextKey::new("chrome.rail.sub.erase"),
    ),
    (
        ids::PAINTER_RAIL_WET_BLEND,
        TextKey::new("chrome.rail.wet_blend"),
        IconId::WetBlend,
        TextKey::new("chrome.rail.sub.wet_blend"),
    ),
    (
        ids::PAINTER_RAIL_WET_SMEAR,
        TextKey::new("chrome.rail.wet_smear"),
        IconId::Smear,
        TextKey::new("chrome.rail.sub.smear"),
    ),
    (
        ids::PAINTER_RAIL_WET_WATER,
        TextKey::new("chrome.rail.wet_water"),
        IconId::WetWater,
        TextKey::new("chrome.rail.sub.wet_water"),
    ),
    (
        ids::PAINTER_RAIL_WET_DRY,
        TextKey::new("chrome.rail.wet_dry"),
        IconId::WetDry,
        TextKey::new("chrome.rail.sub.wet_dry"),
    ),
    (
        ids::PAINTER_RAIL_WET_BLOW,
        TextKey::new("chrome.rail.wet_blow"),
        IconId::WetBlow,
        TextKey::new("chrome.rail.sub.wet_blow"),
    ),
    (
        ids::PAINTER_RAIL_CLONE,
        TextKey::new("chrome.rail.clone"),
        IconId::Clone,
        TextKey::new("chrome.rail.sub.clone"),
    ),
    (
        ids::PAINTER_RAIL_SMEAR,
        TextKey::new("chrome.rail.smear"),
        IconId::Smear,
        TextKey::new("chrome.rail.sub.smear"),
    ),
    (
        ids::PAINTER_RAIL_BLUR,
        TextKey::new("chrome.rail.blur"),
        IconId::Blur,
        TextKey::new("chrome.rail.sub.blur"),
    ),
    (
        ids::PAINTER_RAIL_LIQUIFY,
        TextKey::new("chrome.rail.liquify"),
        IconId::Liquify,
        TextKey::new("chrome.rail.sub.liqfy"),
    ),
    (
        ids::PAINTER_RAIL_TRANSFORM,
        TextKey::new("chrome.rail.transform"),
        IconId::Transform,
        TextKey::new("chrome.rail.sub.xform"),
    ),
    (
        ids::PAINTER_RAIL_MASK_GROUP,
        TextKey::new("chrome.rail.mask"),
        IconId::Mask,
        TextKey::new("chrome.rail.sub.mask"),
    ),
    (
        ids::PAINTER_RAIL_INPAINT,
        TextKey::new("chrome.rail.inpaint"),
        IconId::Inpaint,
        TextKey::new("chrome.rail.sub.inpnt"),
    ),
    (
        ids::PAINTER_RAIL_SHAPES,
        TextKey::new("chrome.rail.shapes"),
        IconId::VectorShape,
        TextKey::new("chrome.rail.sub.shape"),
    ),
];

/// The Shapes flyout options, in flyout order: `(id, a11y label, icon, sub)`.
/// Same chip appearance as the rail tools (icon + short vertical sub-label, same
/// background) — only the icon differs. Revealed right of the Shapes button.
/// The Mask-group flyout options, in flyout order: `(id, a11y label, icon, sub)`. Same chip appearance
/// as the rail tools. Revealed right of the Mask button. Mask is the default sub-tool (its icon is the
/// group's default face); Selection is the Procreate-style marquee.
pub(in crate::screens::hero) const PAINTER_MASK_SUBS: [RailTool; 2] = [
    (
        ids::PAINTER_RAIL_MASK,
        TextKey::new("chrome.rail.mask"),
        IconId::Mask,
        TextKey::new("chrome.rail.sub.mask"),
    ),
    (
        ids::PAINTER_RAIL_SELECTION,
        TextKey::new("chrome.rail.select"),
        IconId::Select,
        TextKey::new("chrome.rail.sub.sel"),
    ),
];

pub(in crate::screens::hero) const PAINTER_SHAPES: [RailTool; 5] = [
    (
        ids::PAINTER_RAIL_SHAPE_FREEHAND,
        TextKey::new("chrome.rail.free_hand"),
        IconId::VectorPencil,
        TextKey::new("chrome.rail.sub.free"),
    ),
    (
        ids::PAINTER_RAIL_SHAPE_LINE,
        TextKey::new("chrome.rail.line"),
        IconId::Line,
        TextKey::new("chrome.rail.sub.line"),
    ),
    (
        ids::PAINTER_RAIL_SHAPE_CURVE,
        TextKey::new("chrome.rail.curve"),
        IconId::VectorPen,
        TextKey::new("chrome.rail.sub.curve"),
    ),
    (
        ids::PAINTER_RAIL_SHAPE_ELLIPSE,
        TextKey::new("chrome.rail.ellipse"),
        IconId::Circle,
        TextKey::new("chrome.rail.sub.elli"),
    ),
    (
        ids::PAINTER_RAIL_SHAPE_POLYGON,
        TextKey::new("chrome.rail.polygon"),
        IconId::Polygon,
        TextKey::new("chrome.rail.sub.poly"),
    ),
];
