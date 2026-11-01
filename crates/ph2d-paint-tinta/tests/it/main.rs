//! Testes da lei única (ADR-0197, mistura Oklab decidida 31/10): paridade com o oráculo, a mistura
//! (associatividade, deriva, ruído, Lema 1 em L), Lemas 2–3 da camada, transferências, dono_6.

mod dono_6;
mod lemas;
mod meio;
mod mistura;
mod paridade;

use ph2d_paint_tinta::{Bandas, Tinta};

/// As tintas do oráculo (`lei_unica.py` `TINTAS`): ultramar-ish e hansa-ish, cor linear + transparência.
pub fn azul() -> Tinta {
    Tinta::de_linear([0.030, 0.060, 0.300], 0.35)
}
pub fn amarelo() -> Tinta {
    Tinta::de_linear([0.800, 0.650, 0.035], 0.11)
}
pub const PAPEL: Bandas<3> = Bandas([0.85, 0.85, 0.82]);

/// Cores de teste sRGB (inclui canais em 0 e 1, que o seletor do artista dá).
pub const CORES: [[f32; 3]; 10] = [
    [0.0, 0.0, 0.0],
    [1.0, 1.0, 1.0],
    [0.0, 0.0, 1.0],
    [1.0, 1.0, 0.0],
    [1.0, 0.0, 1.0],
    [0.0, 1.0, 0.0],
    [0.47, 0.63, 0.86],
    [0.94, 0.78, 0.16],
    [0.8, 0.3, 0.1],
    [0.5, 0.5, 0.5],
];
