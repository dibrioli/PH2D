//! Device-domain stubs — TIPOS TEMPORÁRIOS até T-input + W3 nascerem.
//!
//! Estes tipos existem aqui SOMENTE porque [`StrokeRecord`](crate::record::StrokeRecord)
//! e [`RawPointerSample`](crate::record::RawPointerSample) referenciam-nos
//! (ADR-0046 §2.2/§2.3) e o gate
//! `architecture_painter_contract_surface::stroke_history::raw_pointer_sample_has_device_source`
//! exige o campo `device_source: PointerSource` literal no body de
//! `RawPointerSample` (ADR-0050 §2.7).
//!
//! Quando os crates donos nascerem, o owner deve:
//!
//! 1. Mover `PointerSource` para `crates/ph2d-painter-input/src/pointer_source.rs`
//!    com o set completo de variants (ADR-0050 §2.2; cap ≤ 16) — Apple Pencil,
//!    Wacom Pro Pen 3, Surface Slim Pen 2, finger, mouse, etc. Adicionar dep
//!    `ph2d-painter-input` aqui e trocar `use crate::device::PointerSource`
//!    por `use ph2d_painter_input::PointerSource`.
//! 2. Mover `LayerId` / `LayerStack` / `LayerStackEntry` para
//!    `crates/ph2d-painter-layers/` (W3) com o sistema de layers completo
//!    (raster + group + mask + clipping + reference + alpha-lock + adjustment).
//!
//! Por hora, mantêm-se aqui com surface mínima: 1 variant default neutro
//! e tipos opacos suficientes para o schema postcard v1 (HR-14) compilar.

use serde::{Deserialize, Serialize};

/// Origem do pointer event que gerou um [`RawPointerSample`](crate::record::RawPointerSample).
///
/// ADR-0050 §2.2 estabelece o set canônico de 16 variants. T1.8 ship com
/// `Unknown` apenas — quando `ph2d-painter-input` nascer (T-input),
/// PointerSource migra pra lá e este enum é deletado.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PointerSource {
    /// Origem não classificada — desktop mouse, replay de strokes antigos, MCP-injected
    /// stroke sem device metadata. Default seguro.
    #[default]
    Unknown = 0,
}

/// Identidade de uma layer raster/group/mask dentro de um `LayerStack`.
///
/// Stub W3: newtype sobre `u32` (4G layers possíveis; cap operacional é
/// `HARD_CAP_LAYERS = 999` per spec §2.2 — espelha Procreate — mas o tipo
/// aguenta mais). O modelo runtime W3 vive em `ph2d_tool_painter::layers`
/// (`LayerId(u64)`); a ponte de persistência mapeia u64→u32 no save
/// (Coord ratification 2026-05-31, HANDOFF_painter_w3_layerstack_divergence_RATIFIED.md).
#[derive(
    Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct LayerId(pub u32);

/// Identidade de um canvas (PaintProject runtime instance). ADR-0052 §2.2.
/// Stub W11+: hoje newtype `u64`; futuro pode ganhar workspace prefix
/// quando multi-document workspace W17 nascer.
#[derive(
    Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct CanvasId(pub u64);

/// Stack de layers de um `PaintProject` (savefile congelado).
///
/// ADR-0046 §2.7.1 requer este field na canon savefile. **W3 (v2)** preenche
/// `layers` com [`LayerStackEntry::Node`] em **ordem de z top-first** (índice 0
/// = topo): a ordem da `Vec` É a z-order do root runtime, grupos aninham seus
/// filhos recursivamente, e a layer ativa é marcada por [`LAYER_FLAG_ACTIVE`].
/// O `next_id` do runtime NÃO é serializado — deriva-se de `max(id)+1` no load.
/// O modelo runtime canônico vive em `ph2d_tool_painter::layers`
/// (`LayerId(u64)`); a ponte (de)serialização mapeia u64↔u32 (HANDOFF
/// painter_w3_layerstack_divergence_RATIFIED.md). Files v1 carregam `Reserved`
/// e migram via [`crate::persistence::migrate_v1_to_v2`].
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LayerStack {
    /// Entries em z-order **top-first** (índice 0 = topo) — W3 v2. v1 files
    /// carregam `Reserved` (sem ordem semântica).
    pub layers: Vec<LayerStackEntry>,
}

/// Bit flags de [`LayerNode::modifiers`] — espelham os bools modifier do
/// runtime `ph2d_tool_painter::layers::Layer` (02_layers.md §2.1: clipping /
/// reference / alpha-lock são modifiers, não kinds). Bits 6-7 reservados.
pub const LAYER_FLAG_VISIBLE: u8 = 1 << 0;
/// Bloqueia pintura mas ainda compõe.
pub const LAYER_FLAG_LOCKED: u8 = 1 << 1;
/// Pintura restrita ao alpha existente.
pub const LAYER_FLAG_ALPHA_LOCKED: u8 = 1 << 2;
/// Clipa à layer imediatamente abaixo.
pub const LAYER_FLAG_CLIPPING: u8 = 1 << 3;
/// Marca como reference (geometry source pra ColorDrop/fill).
pub const LAYER_FLAG_IS_REFERENCE: u8 = 1 << 4;
/// A layer ativa (selecionada). Exatamente uma `Node` no stack carrega este bit.
pub const LAYER_FLAG_ACTIVE: u8 = 1 << 5;

/// Entry de um `LayerStack`.
///
/// **HR-14 forward-compat (audit T1.8 L1-F11):** `enum` com `Reserved` no
/// discriminant 0 FROZEN. postcard serializa enum como `varint(discriminant) +
/// payload`; reservar 0 garante que files v1 (que carregam `Reserved(vec![])`)
/// continuem discrimináveis. **W3 (v2)** adiciona `Node = 1` APÓS Reserved (sem
/// mexer no slot 0). Trocar `Reserved` por outro nome QUEBRA forward-compat;
/// novos variants entram em 2, 3, ….
///
/// **Audit T1.8 L4-H8:** derive `Default` + `#[default]` em enum só funciona em
/// variants UNIT — `Reserved(Vec<u8>)` é tuple, exige impl manual (abaixo).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum LayerStackEntry {
    /// Placeholder pré-W3. Discriminant 0 RESERVADO (payload cap
    /// `persistence::MAX_RESERVED_PAYLOAD`).
    Reserved(Vec<u8>) = 0,
    /// Uma layer top-level (W3 v2). Grupos aninham filhos recursivamente.
    Node(LayerNode) = 1,
}

impl Default for LayerStackEntry {
    fn default() -> Self {
        Self::Reserved(Vec::new())
    }
}

/// Uma layer serializada — **metadata only** (os pixels reconstroem-se via
/// replay de `history`, ADR-0046 §2.7.1). Espelha
/// `ph2d_tool_painter::layers::Layer` campo-a-campo, com o `mask:
/// Option<LayerId>` do runtime virando ownership aninhado ([`Self::mask`]) e os
/// 6 bools modifier compactados em [`Self::modifiers`]. 7 fields (cap gate
/// co-locado em `tests`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LayerNode {
    /// Id u32 (runtime `LayerId(u64)` narrowed no save; widened no load).
    pub id: LayerId,
    /// Nome exibido (cap `persistence::MAX_LAYER_NAME_BYTES`).
    pub name: String,
    /// Tipo + payload (raster dims / mask dims / group children).
    pub kind: LayerNodeKind,
    /// `BlendMode::to_u8` (≥22 degrada pra Normal via `from_u8` — nunca panica).
    pub blend_mode: u8,
    /// Opacidade `[0, 1]`.
    pub opacity: f32,
    /// Bitfield `LAYER_FLAG_*` (visible / locked / alpha_locked / clipping /
    /// is_reference / active).
    pub modifiers: u8,
    /// Mask grayscale filha desta layer, se houver (seu `kind` = `Mask`).
    pub mask: Option<Box<LayerNode>>,
}

/// Tipo de uma [`LayerNode`] + payload. Discriminantes wire-stable; espelha o
/// runtime `LayerKind` de 3 variants (os outros "tipos" da spec §2.1 —
/// clipping / reference / alpha-lock — são modifiers em [`LayerNode::modifiers`]).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum LayerNodeKind {
    /// Bitmap raster (dims do canvas).
    Raster { width: u32, height: u32 } = 0,
    /// Mask grayscale 8-bit (multiplica alpha da parent).
    Mask {
        width: u32,
        height: u32,
        inverted: bool,
    } = 1,
    /// Grupo: filhos em z-order top-first + estado collapsed.
    Group {
        children: Vec<LayerNode>,
        collapsed: bool,
    } = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_source_default_is_unknown() {
        assert_eq!(PointerSource::default(), PointerSource::Unknown);
    }

    #[test]
    fn layer_id_default_is_zero() {
        assert_eq!(LayerId::default().0, 0);
    }

    #[test]
    fn layer_stack_default_is_empty() {
        assert!(LayerStack::default().layers.is_empty());
    }

    fn sample_node() -> LayerNode {
        LayerNode {
            id: LayerId(3),
            name: "Foreground".to_string(),
            kind: LayerNodeKind::Group {
                children: vec![LayerNode {
                    id: LayerId(4),
                    name: "Line art".to_string(),
                    kind: LayerNodeKind::Raster {
                        width: 64,
                        height: 48,
                    },
                    blend_mode: 11, // HardLight
                    opacity: 0.5,
                    modifiers: LAYER_FLAG_VISIBLE | LAYER_FLAG_CLIPPING,
                    mask: Some(Box::new(LayerNode {
                        id: LayerId(5),
                        name: "Mask".to_string(),
                        kind: LayerNodeKind::Mask {
                            width: 64,
                            height: 48,
                            inverted: true,
                        },
                        blend_mode: 0,
                        opacity: 1.0,
                        modifiers: LAYER_FLAG_VISIBLE,
                        mask: None,
                    })),
                }],
                collapsed: false,
            },
            blend_mode: 1, // Multiply
            opacity: 0.8,
            modifiers: LAYER_FLAG_VISIBLE | LAYER_FLAG_ACTIVE,
            mask: None,
        }
    }

    #[test]
    fn layer_stack_entry_node_discriminant_is_one() {
        // W3 forward-compat: Reserved=0 frozen, Node=1 added after it.
        let bytes = postcard::to_allocvec(&LayerStackEntry::Reserved(vec![1, 2, 3])).unwrap();
        assert_eq!(bytes[0], 0u8, "Reserved must stay discriminant 0");
        let node_bytes = postcard::to_allocvec(&LayerStackEntry::Node(sample_node())).unwrap();
        assert_eq!(
            node_bytes[0], 1u8,
            "Node must be discriminant 1 (after Reserved)"
        );
    }

    #[test]
    fn layer_node_kind_discriminants_are_stable() {
        let raster = postcard::to_allocvec(&LayerNodeKind::Raster {
            width: 1,
            height: 1,
        })
        .unwrap();
        let mask = postcard::to_allocvec(&LayerNodeKind::Mask {
            width: 1,
            height: 1,
            inverted: false,
        })
        .unwrap();
        let group = postcard::to_allocvec(&LayerNodeKind::Group {
            children: vec![],
            collapsed: false,
        })
        .unwrap();
        assert_eq!((raster[0], mask[0], group[0]), (0u8, 1u8, 2u8));
    }

    #[test]
    fn layer_node_roundtrips_postcard_with_group_and_mask() {
        let entry = LayerStackEntry::Node(sample_node());
        let bytes = postcard::to_allocvec(&entry).unwrap();
        let back: LayerStackEntry = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(entry, back);
        // postcard determinism: re-serialize is byte-identical.
        assert_eq!(bytes, postcard::to_allocvec(&back).unwrap());
    }

    #[test]
    fn layer_node_field_count_is_capped() {
        // Exhaustive destructure (no `..`) — adding a field fails to compile
        // here, forcing an ADR-0046 amendment + cap review (record stays lean).
        let LayerNode {
            id: _,
            name: _,
            kind: _,
            blend_mode: _,
            opacity: _,
            modifiers: _,
            mask: _,
        } = sample_node();
    }

    #[test]
    fn layer_flags_are_distinct_bits() {
        let all = LAYER_FLAG_VISIBLE
            | LAYER_FLAG_LOCKED
            | LAYER_FLAG_ALPHA_LOCKED
            | LAYER_FLAG_CLIPPING
            | LAYER_FLAG_IS_REFERENCE
            | LAYER_FLAG_ACTIVE;
        assert_eq!(
            all.count_ones(),
            6,
            "the six modifier flags must be distinct bits"
        );
        assert_eq!(all & 0b1100_0000, 0, "bits 6-7 reserved (unused)");
    }

    #[test]
    fn pointer_source_roundtrips_postcard() {
        let v = PointerSource::Unknown;
        let bytes = postcard::to_allocvec(&v).unwrap();
        let back: PointerSource = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(v, back);
    }
}
