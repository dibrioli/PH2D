//! `ProceduralGrain` enum — geração matemática de grain no compute shader.
//! 4 variants v1 (cap ≤ 8 ADR-0044 §2.7).
//!
//! Tiling zero, resolução infinita, VRAM econômico (atlas 64 MB → 32 MB).
//! Determinístico se `seed` estável (HR-5 opt-in).

use serde::{Deserialize, Serialize};

/// Procedural grain generator (compute-time). 4 variants v1.
///
/// **Determinismo:** todos os 4 são determinísticos se `seed` é fixo. CPU
/// fallback em `--features det-painter` produz bit-identical cross-OS.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProceduralGrain {
    /// Simplex Noise (Perlin moderno). Genérico noise-like (paper grain,
    /// ruído branco). Custo: ~50 ops/sample.
    /// Ref: Perlin, Ken (2002) "Improving Noise."
    SimplexNoise {
        /// Scale do noise relativo ao stamp (0.1..=4.0).
        scale: f32,
        /// Camadas de oitavas para multifractal (1..=8).
        octaves: u32,
        /// Persistência entre oitavas (0..=1).
        persistence: f32,
        /// Seed determinístico (HR-5 stable).
        seed: u64,
    },
    /// Gabor Noise. Textura anisotrópica (fibras orientadas).
    /// Custo: ~80 ops/sample.
    /// Ref: Lagae, Lefebvre, Drettakis, Dutré (2009).
    GaborNoise {
        frequency: f32,
        orientation: f32,
        anisotropy: f32,
        seed: u64,
    },
    /// Paper Weave — trama de papel/tela (cross-hatched Gabor + threshold).
    /// Custo: ~150 ops/sample. PH2D-specific composition.
    PaperWeave {
        fiber_density: f32,
        fiber_anisotropy: f32,
        crossweave: bool,
        seed: u64,
    },
    /// Spray Dot — Poisson-disk pseudo-aleatório para spray.
    /// Custo: ~30 ops/sample. Bridson 2007 Fast Poisson Disk Sampling.
    SprayDot {
        dot_density: f32,
        dot_size: f32,
        dot_jitter: f32,
        seed: u64,
    },
    // === 4 slots de headroom (cap ≤ 8 ADR-0044 §2.7):
    //   Worley, ReactionDiffusion, etc.
}

impl ProceduralGrain {
    /// Default Simplex preset (fast, neutro, paper-like).
    pub const fn default_simplex() -> Self {
        Self::SimplexNoise {
            scale: 1.0,
            octaves: 4,
            persistence: 0.5,
            seed: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_simplex_constructs() {
        let g = ProceduralGrain::default_simplex();
        if let ProceduralGrain::SimplexNoise {
            scale,
            octaves,
            persistence,
            seed,
        } = g
        {
            assert_eq!(scale, 1.0);
            assert_eq!(octaves, 4);
            assert_eq!(persistence, 0.5);
            assert_eq!(seed, 0);
        } else {
            panic!("default_simplex must be SimplexNoise");
        }
    }
}
