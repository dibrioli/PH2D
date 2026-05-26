//! `Stamp` — unidade atômica do brush engine GPU. ABI FROZEN por ADR-0044 §2.3.
//!
//! 96 bytes, `repr(C, align(16))`, `bytemuck::Pod + Zeroable`. Pool de 4096
//! stamps × 96B = 384 KB / frame, ring buffer reusado (HR-3 zero-alloc).
//!
//! **Não reordene fields.** A ordem é ABI — shader WGSL lê os offsets exatos.
//! Mudanças exigem ADR-amend explícito + bump de version.

use bytemuck::{Pod, Zeroable};

/// Stamp atômico enviado CPU→GPU por frame. Cada brush stroke produz N stamps
/// via `StampScheduler::emit_stamps`; o compute shader carimba cada um na
/// layer texture conforme `rendering_mode` + `pigment_mode`.
///
/// Layout binário FROZEN em ADR-0044 §2.3 (offsets exatos em comment).
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Stamp {
    // ↓ ordem é ABI — não reordene sem amendment de ADR-0044.
    pub position_world: [f32; 2],  //  0..8   x, y in canvas-world pixels
    pub size_px: f32,              //  8..12  diameter em pixels
    pub rotation_rad: f32,         // 12..16  rotação do stamp
    pub pressure: f32,             // 16..20  [0, 1] pós pressure_curve
    pub tilt: f32,                 // 20..24  [0, π/2] pós tilt_curve
    pub azimuth: f32,              // 24..28  [0, 2π)
    pub barrel_roll: f32,          // 28..32  [0, 2π); 0 se device não suporta
    pub color_oklab: [f32; 4],     // 32..48  OKLab (premultiplied alpha em a)
    pub opacity: f32,              // 48..52  [0, 1] pós flow + taper opacity
    pub flow: f32,                 // 52..56  [0, 1] flow do brush
    pub wet_amount: f32,           // 56..60  [0, 1] dilution * wet_load (se wet_mix_enabled)
    pub shape_layer: u32,          // 60..64  índice no shape atlas (Builtin slot OR Imported)
    pub grain_layer: u32, // 64..68  0xFFFFFFFF se sem grain; Procedural usa bit-flag em flags
    pub grain_offset_uv: [f32; 2], // 68..76  offset da grain dentro do shape
    pub grain_scale: f32, // 76..80  scale da grain
    pub flags: u32,       // 80..84  bitmask (vide FLAG_* consts abaixo)
    pub rendering_mode: u32, // 84..88  0..=5 conforme `RenderingMode` enum
    pub pigment_mode: u32, // 88..92  0=Linear, 1=Mixbox (ADR-0044 §2.5)
    pub _pad: u32,        // 92..96  alinhamento 16 (reservado)
}

// Compile-time ABI guards (ADR-0044 §2.3) — falham build se layout mudar.
const _: () = assert!(core::mem::size_of::<Stamp>() == 96);
const _: () = assert!(core::mem::align_of::<Stamp>() == 16);

impl Stamp {
    /// Stamp zero (safety helper para pool init via `bytemuck::zeroed`).
    pub const fn zeroed() -> Self {
        // SAFETY proxy via const expression — Pod+Zeroable garantem layout válido.
        Self {
            position_world: [0.0, 0.0],
            size_px: 0.0,
            rotation_rad: 0.0,
            pressure: 0.0,
            tilt: 0.0,
            azimuth: 0.0,
            barrel_roll: 0.0,
            color_oklab: [0.0, 0.0, 0.0, 0.0],
            opacity: 0.0,
            flow: 0.0,
            wet_amount: 0.0,
            shape_layer: 0,
            grain_layer: u32::MAX, // sem grain
            grain_offset_uv: [0.0, 0.0],
            grain_scale: 1.0,
            flags: 0,
            rendering_mode: 0,
            pigment_mode: 0,
            _pad: 0,
        }
    }
}

// `Stamp::flags` bitmask layout. Manter sync com WGSL shader (ADR-0044 §1.8.2).
pub const FLAG_SHAPE_FLIP_X: u32 = 1 << 0;
pub const FLAG_SHAPE_FLIP_Y: u32 = 1 << 1;
pub const FLAG_GRAIN_BEHAVIOR_MOVING: u32 = 1 << 2;
pub const FLAG_BURNT_EDGES: u32 = 1 << 3;
pub const FLAG_WET_EDGES: u32 = 1 << 4;
pub const FLAG_LUMINANCE_BLENDING: u32 = 1 << 5;
pub const FLAG_GRAIN_PROCEDURAL: u32 = 1 << 6;
// bits 7..32 reservados para flags futuras.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamp_size_is_exactly_96_bytes() {
        assert_eq!(core::mem::size_of::<Stamp>(), 96);
    }

    #[test]
    fn stamp_align_is_16() {
        assert_eq!(core::mem::align_of::<Stamp>(), 16);
    }

    #[test]
    fn stamp_zeroed_constructs() {
        let s = Stamp::zeroed();
        assert_eq!(s.size_px, 0.0);
        assert_eq!(s.grain_layer, u32::MAX); // sentinel "no grain"
    }

    #[test]
    fn stamp_is_pod_zeroable() {
        // Type-system check: Pod + Zeroable derivados via bytemuck.
        fn assert_pod<T: Pod + Zeroable>() {}
        assert_pod::<Stamp>();
    }

    #[test]
    fn stamp_zeroed_round_trips_bytes() {
        let s = Stamp::zeroed();
        let bytes = bytemuck::bytes_of(&s);
        assert_eq!(bytes.len(), 96);
        let s2: Stamp = bytemuck::pod_read_unaligned(bytes);
        assert_eq!(s2.size_px, s.size_px);
        assert_eq!(s2.grain_layer, s.grain_layer);
    }

    #[test]
    fn flag_constants_are_distinct_bits() {
        let all = [
            FLAG_SHAPE_FLIP_X,
            FLAG_SHAPE_FLIP_Y,
            FLAG_GRAIN_BEHAVIOR_MOVING,
            FLAG_BURNT_EDGES,
            FLAG_WET_EDGES,
            FLAG_LUMINANCE_BLENDING,
            FLAG_GRAIN_PROCEDURAL,
        ];
        // Cada flag é uma única bit-position; OR de todos = soma.
        let or_all: u32 = all.iter().fold(0, |acc, f| acc | f);
        let sum: u32 = all.iter().sum();
        assert_eq!(or_all, sum, "flags must be distinct bits");
    }
}
