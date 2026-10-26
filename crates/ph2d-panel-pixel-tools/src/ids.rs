//! **Os ids do painel PIXEL** — os que ESTA crate pinta e despacha. O rect do painel e o abridor do
//! menu vivem no `ph2d_editor_core::ids` (o chrome lê-os); a barra de rolagem é o
//! `ph2d_editor_core::widget::PIXEL_SCROLLBAR_ID`.
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_editor_core::pixel_panel::{
    PixelBrushShape, PixelInk, PixelLayerAction, PixelRequest, PixelSelector, PixelSortKey,
    PixelTool,
};
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// O `×` do cabeçalho.
pub const PIXEL_CLOSE: NodeId = hash_node_id("pixel.close");
/// As amostras de frente e de fundo: o chrome conhece-as (a família lê o seletor por elas).
pub use ph2d_editor_core::ids::{PIXEL_BG, PIXEL_FG};
/// **Trocar** frente e fundo (o `X` do Aseprite).
pub const PIXEL_SWAP: NodeId = hash_node_id("pixel.swap");

/// As ferramentas, na ordem da coluna: pintar, registar e despachar leem a MESMA tabela.
pub const TOOLS: [(PixelTool, NodeId); 14] = [
    (PixelTool::Pencil, hash_node_id("pixel.tool.pencil")),
    (PixelTool::Spray, hash_node_id("pixel.tool.spray")),
    (PixelTool::Eraser, hash_node_id("pixel.tool.eraser")),
    (PixelTool::Eyedropper, hash_node_id("pixel.tool.eyedropper")),
    (PixelTool::Bucket, hash_node_id("pixel.tool.bucket")),
    (PixelTool::Line, hash_node_id("pixel.tool.line")),
    (PixelTool::Curve, hash_node_id("pixel.tool.curve")),
    (PixelTool::Rectangle, hash_node_id("pixel.tool.rectangle")),
    (
        PixelTool::FilledRectangle,
        hash_node_id("pixel.tool.filled_rectangle"),
    ),
    (PixelTool::Ellipse, hash_node_id("pixel.tool.ellipse")),
    (
        PixelTool::FilledEllipse,
        hash_node_id("pixel.tool.filled_ellipse"),
    ),
    (PixelTool::Contour, hash_node_id("pixel.tool.contour")),
    (PixelTool::Polygon, hash_node_id("pixel.tool.polygon")),
    (PixelTool::Hand, hash_node_id("pixel.tool.hand")),
];

/// A forma do pincel (os três segmentos).
pub const SHAPES: [(PixelBrushShape, NodeId); 3] = [
    (
        PixelBrushShape::Circle,
        hash_node_id("pixel.opt.shape.circle"),
    ),
    (
        PixelBrushShape::Square,
        hash_node_id("pixel.opt.shape.square"),
    ),
    (PixelBrushShape::Line, hash_node_id("pixel.opt.shape.line")),
];

/// Os campos numéricos das opções.
pub const SIZE: NodeId = hash_node_id("pixel.opt.size");
pub const ANGLE: NodeId = hash_node_id("pixel.opt.angle");
pub const CORNER: NodeId = hash_node_id("pixel.opt.corner");
pub const TOLERANCE: NodeId = hash_node_id("pixel.opt.tolerance");
pub const SPRAY_WIDTH: NodeId = hash_node_id("pixel.opt.spray_width");
pub const SPRAY_SPEED: NodeId = hash_node_id("pixel.opt.spray_speed");

/// As caixas de marcar das opções.
pub const PIXEL_PERFECT: NodeId = hash_node_id("pixel.opt.pixel_perfect");
pub const CONTIGUOUS: NodeId = hash_node_id("pixel.opt.contiguous");
pub const SYM_H: NodeId = hash_node_id("pixel.opt.sym_h");
pub const SYM_V: NodeId = hash_node_id("pixel.opt.sym_v");
pub const SYM_D1: NodeId = hash_node_id("pixel.opt.sym_d1");
pub const SYM_D2: NodeId = hash_node_id("pixel.opt.sym_d2");
pub const TILE_X: NodeId = hash_node_id("pixel.opt.tile_x");
pub const TILE_Y: NodeId = hash_node_id("pixel.opt.tile_y");

/// A conectividade do balde (4 · 8) e o que ele compara (camada · visível).
pub const CONNECT_4: NodeId = hash_node_id("pixel.opt.connect4");
pub const CONNECT_8: NodeId = hash_node_id("pixel.opt.connect8");
pub const SAMPLE_LAYER: NodeId = hash_node_id("pixel.opt.sample_layer");
pub const SAMPLE_VISIBLE: NodeId = hash_node_id("pixel.opt.sample_visible");

/// Os campos numéricos: id, faixa `(min, max, passo)`.
pub const NUMBERS: [(NodeId, (f64, f64, f64)); 7] = [
    (
        SIZE,
        (
            1.0,
            ph2d_editor_core::pixel_panel::PIXEL_MAX_BRUSH as f64,
            1.0,
        ),
    ),
    (ANGLE, (-180.0, 180.0, 1.0)),
    (CORNER, (0.0, 64.0, 1.0)),
    (TOLERANCE, (0.0, 255.0, 1.0)),
    (SPRAY_WIDTH, (1.0, 64.0, 1.0)),
    (SPRAY_SPEED, (1.0, 128.0, 1.0)),
    (REPLACE_TOLERANCE, (0.0, 255.0, 1.0)),
];

/// As caixas de marcar.
pub const CHECKS: [NodeId; 8] = [
    PIXEL_PERFECT,
    CONTIGUOUS,
    SYM_H,
    SYM_V,
    SYM_D1,
    SYM_D2,
    TILE_X,
    TILE_Y,
];

/// Os segmentos que não são a forma.
pub const SEGMENTS: [NodeId; 4] = [CONNECT_4, CONNECT_8, SAMPLE_LAYER, SAMPLE_VISIBLE];

/// **A célula `i` da paleta** — o clique esquerdo (cor de frente). A população é a paleta, sem tecto.
#[must_use]
pub fn palette_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.palette.{i}"))
}

/// O GÉMEO do botão direito da célula `i` (cor de fundo) — `HitIndex::register_with_secondary`.
#[must_use]
pub fn palette_bg_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.palette_bg.{i}"))
}

/// As tintas, na ordem do Aseprite (pintar, registar e despachar leem a MESMA tabela).
pub const INKS: [(PixelInk, NodeId); 5] = [
    (PixelInk::Simple, hash_node_id("pixel.opt.ink.simple")),
    (
        PixelInk::AlphaCompositing,
        hash_node_id("pixel.opt.ink.alpha"),
    ),
    (PixelInk::CopyAlphaColor, hash_node_id("pixel.opt.ink.copy")),
    (PixelInk::LockAlpha, hash_node_id("pixel.opt.ink.lock")),
    (PixelInk::Shading, hash_node_id("pixel.opt.ink.shading")),
];

/// Os critérios do Sort.
pub const SORT_KEYS: [(PixelSortKey, NodeId); 8] = [
    (PixelSortKey::Hue, hash_node_id("pixel.sort.hue")),
    (
        PixelSortKey::Saturation,
        hash_node_id("pixel.sort.saturation"),
    ),
    (
        PixelSortKey::Brightness,
        hash_node_id("pixel.sort.brightness"),
    ),
    (
        PixelSortKey::Luminance,
        hash_node_id("pixel.sort.luminance"),
    ),
    (PixelSortKey::Red, hash_node_id("pixel.sort.red")),
    (PixelSortKey::Green, hash_node_id("pixel.sort.green")),
    (PixelSortKey::Blue, hash_node_id("pixel.sort.blue")),
    (PixelSortKey::Alpha, hash_node_id("pixel.sort.alpha")),
];

/// O campo *Palette Size* (o valor é o tamanho da paleta do sprite, nunca uma opção).
pub const PALETTE_SIZE: NodeId = hash_node_id("pixel.palette.size");
/// A tolerância do *Replace Color* (um campo das opções).
pub const REPLACE_TOLERANCE: NodeId = hash_node_id("pixel.opt.replace_tolerance");
/// A amostra do *Edit color* (o chrome conhece-a: a família lê o seletor por ela).
pub use ph2d_editor_core::ids::PIXEL_PAL_EDIT;
/// Abre o diálogo *Color Mode* sobre o sprite em edição.
pub const COLOR_MODE: NodeId = hash_node_id("pixel.color_mode.open");

/// Os botões das acções da paleta e da cor: o id, o pedido que fazem e o rótulo.
pub const ACTIONS: [(NodeId, PixelRequest, &str); 10] = [
    (
        hash_node_id("pixel.palette.move_left"),
        PixelRequest::MoveSelection(-1),
        "panel.pixel.move_left",
    ),
    (
        hash_node_id("pixel.palette.move_right"),
        PixelRequest::MoveSelection(1),
        "panel.pixel.move_right",
    ),
    (
        hash_node_id("pixel.palette.sort_ascending"),
        PixelRequest::Sort { descending: false },
        "panel.pixel.sort.ascending",
    ),
    (
        hash_node_id("pixel.palette.sort_descending"),
        PixelRequest::Sort { descending: true },
        "panel.pixel.sort.descending",
    ),
    (
        hash_node_id("pixel.palette.select_used"),
        PixelRequest::SelectUsed,
        "panel.pixel.select_used",
    ),
    (
        hash_node_id("pixel.palette.from_sprite"),
        PixelRequest::PaletteFromSprite,
        "panel.pixel.from_sprite",
    ),
    (
        hash_node_id("pixel.palette.load"),
        PixelRequest::LoadPalette,
        "panel.pixel.load",
    ),
    (
        hash_node_id("pixel.palette.save"),
        PixelRequest::SavePalette,
        "panel.pixel.save",
    ),
    (
        hash_node_id("pixel.palette.default"),
        PixelRequest::DefaultPalette,
        "panel.pixel.default_palette",
    ),
    (
        hash_node_id("pixel.replace.apply"),
        PixelRequest::ReplaceColor,
        "panel.pixel.replace_apply",
    ),
];

// ── as camadas (P3) ───────────────────────────────────────────────────────────

/// As partes de uma linha da lista de camadas (a pintura é partilhada com a linha do tempo).
pub use ph2d_editor_core::pixel_layer_row::PixelLayerWidget as LayerWidget;

/// O id da parte `w` da linha da camada `layer` (a população é a lista, sem tecto).
#[must_use]
pub fn layer_widget_id(layer: u32, w: LayerWidget) -> NodeId {
    hash_node_id_runtime(&format!("pixel.layer.{layer}.{}", w.tag()))
}

/// O campo do renomear (duplo clique no nome).
pub const LAYER_RENAME: NodeId = hash_node_id("pixel.layer.rename");
/// A opacidade da camada activa (`0..=255`, a escala do Aseprite).
pub const LAYER_OPACITY: NodeId = hash_node_id("pixel.layer.opacity");
/// O modo da camada activa (um menu de escolha).
pub const LAYER_BLEND: NodeId = hash_node_id("pixel.layer.blend");

/// A opção `i` do menu do modo (a ordem de [`ph2d_editor_core::pixel_panel::PixelBlend::ALL`]).
#[must_use]
pub fn blend_option_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.layer.blend.{i}"))
}

/// Os botões do menu Layer: o id, a acção e o rótulo, nas filas em que se pintam.
pub const LAYER_ACTIONS: [(NodeId, PixelLayerAction, &str); 16] = [
    (hash_node_id("pixel.layer.new"), PixelLayerAction::NewLayer, "panel.pixel.layer.new"),
    (hash_node_id("pixel.layer.new_group"), PixelLayerAction::NewGroup, "panel.pixel.layer.new_group"),
    (hash_node_id("pixel.layer.duplicate"), PixelLayerAction::Duplicate, "panel.pixel.layer.duplicate"),
    (hash_node_id("pixel.layer.delete"), PixelLayerAction::Delete, "panel.pixel.layer.delete"),
    (hash_node_id("pixel.layer.up"), PixelLayerAction::MoveUp, "panel.pixel.layer.up"),
    (hash_node_id("pixel.layer.down"), PixelLayerAction::MoveDown, "panel.pixel.layer.down"),
    (hash_node_id("pixel.layer.merge_down"), PixelLayerAction::MergeDown, "panel.pixel.layer.merge_down"),
    (hash_node_id("pixel.layer.flatten"), PixelLayerAction::Flatten, "panel.pixel.layer.flatten"),
    (hash_node_id("pixel.layer.flatten_visible"), PixelLayerAction::FlattenVisible, "panel.pixel.layer.flatten_visible"),
    (hash_node_id("pixel.layer.via_copy"), PixelLayerAction::ViaCopy, "panel.pixel.layer.via_copy"),
    (hash_node_id("pixel.layer.via_cut"), PixelLayerAction::ViaCut, "panel.pixel.layer.via_cut"),
    (hash_node_id("pixel.layer.copy"), PixelLayerAction::Copy, "panel.pixel.layer.copy"),
    (hash_node_id("pixel.layer.paste"), PixelLayerAction::PasteAsLayer, "panel.pixel.layer.paste"),
    (hash_node_id("pixel.layer.paste_reference"), PixelLayerAction::PasteAsReference, "panel.pixel.layer.paste_reference"),
    (hash_node_id("pixel.layer.select_all"), PixelLayerAction::SelectAll, "panel.pixel.layer.select_all"),
    (hash_node_id("pixel.layer.deselect"), PixelLayerAction::Deselect, "panel.pixel.layer.deselect"),
];

// ── o seletor de cor e as predefinições da paleta (P2) ──────────────────────────

/// Os cinco seletores do menu ≡ da paleta (pintar, registar e despachar leem a MESMA tabela).
pub const SELECTORS: [(PixelSelector, NodeId); 5] = [
    (PixelSelector::TintShadeTone, hash_node_id("pixel.selector.tint")),
    (PixelSelector::Spectrum, hash_node_id("pixel.selector.spectrum")),
    (PixelSelector::RgbWheel, hash_node_id("pixel.selector.rgb_wheel")),
    (PixelSelector::RybWheel, hash_node_id("pixel.selector.ryb_wheel")),
    (PixelSelector::NormalMap, hash_node_id("pixel.selector.normal_map")),
];

/// As três áreas do seletor (quadro, barra do modo, alfa), cada uma com o gémeo do botão direito
/// (a cor de fundo): `(esquerdo, direito)`.
pub const SELECTOR_AREAS: [(NodeId, NodeId); 3] = [
    (
        hash_node_id("pixel.selector.square"),
        hash_node_id("pixel.selector.square_bg"),
    ),
    (
        hash_node_id("pixel.selector.bar"),
        hash_node_id("pixel.selector.bar_bg"),
    ),
    (
        hash_node_id("pixel.selector.alpha"),
        hash_node_id("pixel.selector.alpha_bg"),
    ),
];

/// *Save Palette as Preset* · *Save Palette as Default* (id, pedido, rótulo).
pub const PRESET_ACTIONS: [(NodeId, PixelRequest, &str); 2] = [
    (
        hash_node_id("pixel.palette.save_preset"),
        PixelRequest::SavePreset,
        "panel.pixel.save_preset",
    ),
    (
        hash_node_id("pixel.palette.save_default"),
        PixelRequest::SaveDefault,
        "panel.pixel.save_default",
    ),
];

/// A predefinição `i` da lista (a população é a pasta do artista, sem tecto).
#[must_use]
pub fn preset_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.palette.preset.{i}"))
}
