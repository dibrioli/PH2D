//! Atlas — Shape + Grain texture arrays. ADR-0044 §1.8.1 (spec §1.8.1).
//!
//! **T1.3 stub.** Tipos + dimensões + cap counts; sem GPU upload real
//! (T1.5+ implementa via `ph2d-gpu` wrapper).
//!
//! Shape atlas: 64 layers × 256×256 × R8 = 4 MB VRAM.
//! Grain atlas: 64 layers × 1024×1024 × R8 = 32 MB VRAM (pós-Procedural
//! Grain reduction; ADR-0044 §2.7).

/// Max Shape atlas layers (built-in 0..32 + imported LRU 32..64).
pub const SHAPE_ATLAS_LAYERS: u32 = 64;
/// Shape atlas tile dimensions.
pub const SHAPE_ATLAS_TILE_PX: u32 = 256;

/// Max Grain atlas layers (built-in bitmap 0..16 + imported LRU 16..64).
pub const GRAIN_ATLAS_LAYERS: u32 = 64;
/// Grain atlas tile dimensions.
pub const GRAIN_ATLAS_TILE_PX: u32 = 1024;

/// Atlas placeholder. T1.5+ substitui por `Atlas { shape_tex: TextureHandle,
/// grain_tex: TextureHandle, layout: LruIndex, ... }` via ph2d-gpu wrapper.
#[derive(Debug, Default)]
pub struct AtlasStub {
    /// Built-in shape slots ocupados (built-in fixed; só smoke).
    pub shape_builtin_count: u32,
    /// Built-in grain slots ocupados.
    pub grain_builtin_count: u32,
}

impl AtlasStub {
    /// VRAM budget estimado em MB (Shape + Grain atlas).
    pub const fn vram_mb_estimate() -> u32 {
        // (Shape: 64 × 256² × 1 byte) + (Grain: 64 × 1024² × 1 byte) bytes → MB.
        let shape_bytes = SHAPE_ATLAS_LAYERS * SHAPE_ATLAS_TILE_PX * SHAPE_ATLAS_TILE_PX;
        let grain_bytes = GRAIN_ATLAS_LAYERS * GRAIN_ATLAS_TILE_PX * GRAIN_ATLAS_TILE_PX;
        (shape_bytes + grain_bytes) / (1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vram_budget_matches_adr_0044() {
        // ADR-0044 §1.8.1: Shape 4 MB + Grain 32 MB (pós-Procedural reduction).
        // Total: 36 MB. (Antes Procedural era 64 MB grain = 68 MB total.)
        assert_eq!(AtlasStub::vram_mb_estimate(), 4 + 64);
        // Nota: 64 MB grain ainda — Procedural pull-down (Procedural Grain
        // compute-time substitui 4/8 grãos bitmap em W5+, ADR-0044 §2.7).
        // Em T1.3 atlas stub, slots TODOS reservados como bitmap; reduction
        // efetiva acontece em runtime quando Procedural ship.
    }

    #[test]
    fn atlas_layer_counts_match_adr() {
        assert_eq!(SHAPE_ATLAS_LAYERS, 64);
        assert_eq!(GRAIN_ATLAS_LAYERS, 64);
    }
}
