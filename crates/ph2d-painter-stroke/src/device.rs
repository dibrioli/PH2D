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

/// Stack de layers de um `PaintProject`.
///
/// Stub W3: contém apenas a sequência de entries opacas. ADR-0046 §2.7.1
/// requer este field na canon savefile; W3 ([ADR-0045](../../../../docs/architecture/decisions/0045-adjustment-layers.md)
/// + 02_layers.md spec) preenche com `enum LayerKind { Raster, Group,
/// Mask, ClippingMask, Reference, AlphaLocked, Adjustment }`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LayerStack {
    /// Entries em ordem bottom-up (índice 0 = bottom).
    pub layers: Vec<LayerStackEntry>,
}

/// Entry de um `LayerStack` — preenchida em W3.
///
/// **HR-14 forward-compat (audit T1.8 L1-F11):** declarado como `enum` com
/// `Reserved` no discriminant 0 pra preservar future-compat quando W3
/// substituir por `enum LayerKind { Raster, Group, Mask, ClippingMask,
/// Reference, AlphaLocked, Adjustment }`. postcard serializa enum como
/// `varint(discriminant) + payload`; reservando 0 aqui garante que
/// futuros variants (1..N) sejam adicionados sem mexer no discriminant 0
/// existente — files `.ph2d-painter` v1 carregam `Reserved(vec![])` que
/// um v2 reader pode discriminar (e tratar como "layer stack vazio" ou
/// migrar via [`crate::persistence::migrate_v1_to_v2`]).
///
/// Trocar `Reserved` por outro nome no futuro QUEBRA forward-compat. Add
/// novos variants APÓS Reserved (i.e., `Raster = 1`, `Group = 2`, ...).
///
/// **Audit T1.8 L4-H8 reverted:** Rust derive Default + #[default] em enum
/// só funciona em variants UNIT (sem fields). `Reserved(Vec<u8>)` é tuple
/// variant — exige impl manual (mantido abaixo).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum LayerStackEntry {
    /// Placeholder pré-W3. Discriminant 0 RESERVADO.
    Reserved(Vec<u8>) = 0,
}

impl Default for LayerStackEntry {
    fn default() -> Self {
        Self::Reserved(Vec::new())
    }
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

    #[test]
    fn pointer_source_roundtrips_postcard() {
        let v = PointerSource::Unknown;
        let bytes = postcard::to_allocvec(&v).unwrap();
        let back: PointerSource = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(v, back);
    }
}
