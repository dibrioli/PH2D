//! **Os ids do painel PIXEL** — os que ESTA crate pinta e despacha. O rect do painel e o abridor do
//! menu vivem no `ph2d_editor_core::ids` (o chrome lê-os); a barra de rolagem é o
//! `ph2d_editor_core::widget::PIXEL_SCROLLBAR_ID`.
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_editor_core::pixel_panel::{PixelBrushShape, PixelTool};
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
pub const NUMBERS: [(NodeId, (f64, f64, f64)); 6] = [
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
