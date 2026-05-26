//! `BrushHandle` — opaco u32 referência a um brush no atlas/library. ADR-0044 §2.8.
//!
//! Bit-31 = imported flag (1 = atlas layer livre, blake3-keyed; 0 = built-in
//! Library fixed slot 0..63). Bits 0..30 = slot index.

use serde::{Deserialize, Serialize};

/// Reference opaca para um brush. **NÃO** copia o `Brush` struct — usa atlas
/// resolution via `Library` lookup.
///
/// Layout (audit 2026-05-26 + ADR-0044 §2.8):
/// - Bit 31 = 1 → imported (atlas layer livre, blake3-keyed)
/// - Bit 31 = 0 → built-in (Library fixed slot 0..63 reservado)
/// - Bits 0..30 = slot index
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BrushHandle(pub u32);

impl BrushHandle {
    /// Flag bit 31 — imported (atlas layer livre) vs built-in (Library slot).
    pub const IMPORTED_FLAG: u32 = 1 << 31;

    /// Construct a built-in BrushHandle. `slot` deve ser < 64 (built-in
    /// Library reserva os 64 primeiros slots).
    pub fn new_builtin(slot: u32) -> Self {
        debug_assert!(
            slot < 64,
            "built-in BrushHandle slot must be < 64; got {}",
            slot
        );
        Self(slot)
    }

    /// Construct an imported BrushHandle. `atlas_layer` é o slot livre no
    /// brush atlas; bit 31 é setado para marcar como imported.
    pub fn new_imported(atlas_layer: u32) -> Self {
        debug_assert!(
            atlas_layer & Self::IMPORTED_FLAG == 0,
            "atlas_layer must fit in 31 bits; got {:#x}",
            atlas_layer
        );
        Self(atlas_layer | Self::IMPORTED_FLAG)
    }

    /// `true` se este handle aponta para um brush imported (vs built-in).
    pub fn is_imported(&self) -> bool {
        self.0 & Self::IMPORTED_FLAG != 0
    }

    /// Slot index (bits 0..30; bit 31 mascarado).
    pub fn slot(&self) -> u32 {
        self.0 & !Self::IMPORTED_FLAG
    }
}

impl Default for BrushHandle {
    fn default() -> Self {
        // Default = slot 0 built-in = ROUND_HARD (Library §1.6.7).
        Self::new_builtin(0)
    }
}

/// Hash blake3 (32 bytes) dos `Brush.params` para dedup em stroke history.
/// Usado por `StrokeRecord.brush_params_hash` (ADR-0046 §2.2): mesma cor +
/// mesma brush mid-stroke ⇒ mesmo hash ⇒ atlas reusa entry em `brush_snapshots`
/// dedup table no `.ph2d-painter` persisted (ADR-0046 §2.7.1).
pub type BrushParamsHash = [u8; 32];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_builtin_clears_imported_flag() {
        let h = BrushHandle::new_builtin(5);
        assert!(!h.is_imported());
        assert_eq!(h.slot(), 5);
    }

    #[test]
    fn new_imported_sets_imported_flag() {
        let h = BrushHandle::new_imported(42);
        assert!(h.is_imported());
        assert_eq!(h.slot(), 42);
    }

    #[test]
    fn slot_round_trips_for_builtin() {
        for slot in [0, 1, 32, 63] {
            let h = BrushHandle::new_builtin(slot);
            assert_eq!(h.slot(), slot);
            assert!(!h.is_imported());
        }
    }

    #[test]
    fn slot_round_trips_for_imported() {
        for slot in [0, 1, 64, 1024, 0x7FFF_FFFF] {
            let h = BrushHandle::new_imported(slot);
            assert_eq!(h.slot(), slot);
            assert!(h.is_imported());
        }
    }

    #[test]
    fn default_is_builtin_slot_0() {
        let h = BrushHandle::default();
        assert!(!h.is_imported());
        assert_eq!(h.slot(), 0);
    }
}
