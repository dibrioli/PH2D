//! **Os ids da linha do tempo do Pixel** — os que ESTA crate pinta e despacha. O rect do painel é o
//! `ph2d_editor_core::ids::PIXEL_TIMELINE_PANEL` (o chrome lê-o).
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::pixel_layer_row::PixelLayerWidget;
use ph2d_editor_core::pixel_panel::{PixelDirection, PixelFrameAction as A};
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// O `×` do cabeçalho.
pub const CLOSE: NodeId = hash_node_id("pixel_timeline.close");

/// A navegação e o tocar (Home · ← · Enter · → · End): a acção, o id, o ícone e a dica. O ícone do
/// tocar troca para *Stop* enquanto toca.
pub const TRANSPORT: [(A, NodeId, IconId, &str); 5] = [
    (
        A::First,
        hash_node_id("pixel_timeline.first"),
        IconId::SkipBack,
        "panel.pixel.timeline.first",
    ),
    (
        A::Previous,
        hash_node_id("pixel_timeline.previous"),
        IconId::ChevronLeft,
        "panel.pixel.timeline.previous",
    ),
    (
        A::Play,
        hash_node_id("pixel_timeline.play"),
        IconId::Play,
        "panel.pixel.timeline.play",
    ),
    (
        A::Next,
        hash_node_id("pixel_timeline.next"),
        IconId::ChevronRight,
        "panel.pixel.timeline.next",
    ),
    (
        A::Last,
        hash_node_id("pixel_timeline.last"),
        IconId::SkipForward,
        "panel.pixel.timeline.last",
    ),
];

/// O papel vegetal e a tinta dele (dois interruptores): a acção, o id e o rótulo.
pub const TOGGLES: [(A, NodeId, &str); 2] = [
    (
        A::Onion,
        hash_node_id("pixel_timeline.onion"),
        "panel.pixel.timeline.onion",
    ),
    (
        A::OnionTint,
        hash_node_id("pixel_timeline.onion_tint"),
        "panel.pixel.timeline.onion_tint",
    ),
];

/// O menu Frame (e o do Cel e das Tags) na barra, em ícones compactos com a dica do nome do
/// Aseprite: a acção, o id, o ícone e a chave da dica. ⚠️ Ícones e não palavras: a banda de fábrica
/// tem 240 px, e dez palavras ocupavam duas linhas dela (smoke da cena 8, 25/10).
pub const ACTIONS: [(A, NodeId, IconId, &str); 10] = [
    (
        A::NewFrame,
        hash_node_id("pixel_timeline.new_frame"),
        IconId::Duplicate,
        "panel.pixel.timeline.new_frame",
    ),
    (
        A::NewEmptyFrame,
        hash_node_id("pixel_timeline.new_empty_frame"),
        IconId::Plus,
        "panel.pixel.timeline.new_empty_frame",
    ),
    (
        A::DuplicateLinked,
        hash_node_id("pixel_timeline.duplicate_linked"),
        IconId::Clone,
        "panel.pixel.timeline.duplicate_linked",
    ),
    (
        A::Delete,
        hash_node_id("pixel_timeline.delete_frame"),
        IconId::Trash,
        "panel.pixel.timeline.delete_frame",
    ),
    (
        A::MoveLeft,
        hash_node_id("pixel_timeline.move_left"),
        IconId::ArrangeBackward,
        "panel.pixel.timeline.move_left",
    ),
    (
        A::MoveRight,
        hash_node_id("pixel_timeline.move_right"),
        IconId::ArrangeForward,
        "panel.pixel.timeline.move_right",
    ),
    (
        A::Reverse,
        hash_node_id("pixel_timeline.reverse"),
        IconId::FlipH,
        "panel.pixel.timeline.reverse",
    ),
    (
        A::LinkPrevious,
        hash_node_id("pixel_timeline.link_previous"),
        IconId::Link,
        "panel.pixel.timeline.link_previous",
    ),
    (
        A::Unlink,
        hash_node_id("pixel_timeline.unlink"),
        IconId::Unlink,
        "panel.pixel.timeline.unlink",
    ),
    (
        A::NewTag,
        hash_node_id("pixel_timeline.new_tag"),
        IconId::Tag,
        "panel.pixel.timeline.new_tag",
    ),
];

/// Os grupos da barra de acções (índices em [`ACTIONS`]): criar · apagar · mover · ligar · tag.
pub const ACTION_GROUPS: [&[usize]; 5] = [&[0, 1, 2], &[3], &[4, 5, 6], &[7, 8], &[9]];

/// O cabeçalho do frame `frame` (o id estável do frame; a população é a linha do tempo).
#[must_use]
pub fn header_id(frame: u32) -> NodeId {
    hash_node_id_runtime(&format!("pixel_timeline.frame.{frame}"))
}

/// A célula da camada `layer` no frame `frame` (ids estáveis).
#[must_use]
pub fn cell_id(layer: u32, frame: u32) -> NodeId {
    hash_node_id_runtime(&format!("pixel_timeline.cel.{layer}.{frame}"))
}

/// A barra da tag `i` (a posição na lista).
#[must_use]
pub fn tag_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel_timeline.tag.{i}"))
}

/// A parte `w` da linha da camada `layer` na coluna das camadas desta linha do tempo.
#[must_use]
pub fn layer_widget_id(layer: u32, w: PixelLayerWidget) -> NodeId {
    hash_node_id_runtime(&format!("pixel_timeline.layer.{layer}.{}", w.tag()))
}

/// O campo do renomear da camada (duplo clique no nome).
pub const RENAME_LAYER: NodeId = hash_node_id("pixel_timeline.layer.rename");
/// O campo do renomear da tag (duplo clique na barra dela).
pub const RENAME_TAG: NodeId = hash_node_id("pixel_timeline.tag.rename");

/// Frame Properties ▸ Duration (ms) do frame activo.
pub const FRAME_DURATION: NodeId = hash_node_id("pixel_timeline.frame.duration");
/// Cel Properties do cel activo: a opacidade (`0..=255`) e o z.
pub const CEL_OPACITY: NodeId = hash_node_id("pixel_timeline.cel.opacity");
pub const CEL_Z: NodeId = hash_node_id("pixel_timeline.cel.z");
/// Tag Properties da tag sob o frame activo: a direcção (menu), as repetições e apagar.
pub const TAG_DIRECTION: NodeId = hash_node_id("pixel_timeline.tag.direction");
pub const TAG_REPEAT: NodeId = hash_node_id("pixel_timeline.tag.repeat");
pub const TAG_DELETE: NodeId = hash_node_id("pixel_timeline.tag.delete");

/// Os campos numéricos: id, faixa `(min, max, passo)`.
pub const NUMBERS: [(NodeId, (f64, f64, f64)); 4] = [
    (FRAME_DURATION, (1.0, u16::MAX as f64, 1.0)),
    (CEL_OPACITY, (0.0, u8::MAX as f64, 1.0)),
    (CEL_Z, (i16::MIN as f64, i16::MAX as f64, 1.0)),
    (TAG_REPEAT, (0.0, u16::MAX as f64, 1.0)),
];

/// A opção `i` do menu da direcção (a ordem de [`PixelDirection::ALL`]).
#[must_use]
pub fn direction_option_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel_timeline.tag.direction.{i}"))
}

/// O rótulo de uma direcção (o menu) e o glifo dela (a barra da tag).
#[must_use]
pub const fn direction_keys(d: PixelDirection) -> (&'static str, &'static str) {
    match d {
        PixelDirection::Forward => (
            "panel.pixel.timeline.dir.forward",
            "panel.pixel.timeline.glyph.forward",
        ),
        PixelDirection::Reverse => (
            "panel.pixel.timeline.dir.reverse",
            "panel.pixel.timeline.glyph.reverse",
        ),
        PixelDirection::PingPong => (
            "panel.pixel.timeline.dir.ping_pong",
            "panel.pixel.timeline.glyph.ping_pong",
        ),
        PixelDirection::PingPongReverse => (
            "panel.pixel.timeline.dir.ping_pong_reverse",
            "panel.pixel.timeline.glyph.ping_pong_reverse",
        ),
    }
}
