//! Testes da lei única (ADR-0197 §4.2): paridade com o oráculo, Lemas 1–3 + teorema, dono_6, N passadas.

mod dono_6;
mod lemas;
mod paridade;

use ph2d_paint_tinta::{Bandas, Pigmento};

/// Os pigmentos do oráculo (`lei_unica.py`): ultramar-ish e hansa-ish, R∞ linear e S em 1/µm.
pub fn azul() -> Pigmento<3> {
    Pigmento::de_rinf(Bandas([0.030, 0.060, 0.300]), Bandas::uniforme(0.4))
}
pub fn amarelo() -> Pigmento<3> {
    Pigmento::de_rinf(Bandas([0.800, 0.650, 0.035]), Bandas::uniforme(1.2))
}
pub const PAPEL: Bandas<3> = Bandas([0.85, 0.85, 0.82]);
