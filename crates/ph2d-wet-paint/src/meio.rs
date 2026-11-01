//! ⭐ **O LÍQUIDO da tinta numa lei só** (dono 01/11, `project_paint_type_water_oil_one_law_2026_11_01`, ADR-0197): o
//! diluente é SEMPRE líquido — a receita pousa `(1−φ)·h ÷ K_W` com qualquer meio. O meio `m` da célula
//! ([`crate::grid::oleo_celula::OleoCelula::meio`]) só muda as PROPRIEDADES desse líquido (F2 §2):
//!
//! - viscosidade `log η = (1−m)·log η_água + m·log η_óleo` ⇒ a mobilidade do escoamento `η_água ÷ η = ETA^−m`;
//! - evaporação `E·(1−m)` (a orla escura é evaporação: o óleo não a tem) mais o ENDURECIMENTO `E·m·ENDURECE` (o óleo
//!   não evapora, mas fica preso com o tempo — sem isto, numa tela que não bebe, o solver corria para sempre);
//! - absorção pelo papel (Washburn, `ℓ ∝ 1/√η`) ⇒ a taxa de fuga `k·ETA^(−m/2)`.
//!
//! `m = 0` devolve o motor de antes ao bit (cada função devolve a entrada sem aritmética). Medido:
//! `docs/Painter/Arquitetura/E10g_oleo_diluido.md`.

use std::sync::atomic::{AtomicU32, Ordering};

/// `η_óleo ÷ η_água` do diluente.
pub const ETA_OLEO: f64 = 8.0;
/// A taxa com que o líquido do óleo fica preso, em fracção da evaporação da água.
pub const ENDURECE: f64 = 0.1;

/// O líquido que o pigmento PRENDE no óleo (o índice de absorção de óleo), por unidade de pigmento: a tinta do tubo
/// (`Dilution ≤ L/(1+L)` = 0,1) é pasta sem líquido livre; só o diluente acima disso corre. Na água não há.
pub const LIGADO_NO_OLEO: f64 = 1.0 / 9.0;

/// Ferramenta de MEDIÇÃO (sondas), nunca o produto: os bits de um `f32` — `0` = [`ETA_OLEO`].
#[doc(hidden)]
pub static PAPEL_ETA: AtomicU32 = AtomicU32::new(0);
/// Ferramenta de MEDIÇÃO: os bits de um `f32` — `0` = [`ENDURECE`].
#[doc(hidden)]
pub static PAPEL_ENDURECE: AtomicU32 = AtomicU32::new(0);

fn medida(a: &AtomicU32, omissao: f64) -> f64 {
    match a.load(Ordering::Relaxed) {
        0 => omissao,
        b => f64::from(f32::from_bits(b)),
    }
}

/// A fracção LIVRE do volume de um dab: `(1−φ) − m·L·φ`, nunca negativa (`m = 0` ⇒ `1−φ`, ao bit).
#[inline]
#[must_use]
pub fn liquido_livre(phi: f64, m: f64) -> f64 {
    if m <= 0.0 {
        return 1.0 - phi;
    }
    ((1.0 - phi) - m.min(1.0) * LIGADO_NO_OLEO * phi).max(0.0)
}

/// `η_água ÷ η(m)`: o factor do incremento de velocidade da célula.
#[inline]
#[must_use]
pub fn mobilidade(m: f64) -> f64 {
    if m <= 0.0 {
        return 1.0;
    }
    medida(&PAPEL_ETA, ETA_OLEO).powf(-m.min(1.0))
}

/// O factor da perda linear do filme (evaporação + endurecimento): `(1−m) + m·ENDURECE`.
#[inline]
#[must_use]
pub fn evaporacao(m: f64) -> f64 {
    if m <= 0.0 {
        return 1.0;
    }
    let m = m.min(1.0);
    (1.0 - m) + m * medida(&PAPEL_ENDURECE, ENDURECE)
}

/// A retenção do filme por passe com o meio: `r^(ETA^(−m/2))` (a fuga `k` mais lenta por Washburn).
#[inline]
#[must_use]
pub fn retencao(r: f64, m: f64) -> f64 {
    if m <= 0.0 {
        return r;
    }
    r.powf(mobilidade(m).sqrt())
}

/// O meio de uma célula que recebe tinta: a média pelo volume MOLHADO (o líquido + o pigmento em suspensão, em
/// massa: `susp + K_W·film`) do que estava e do que chega (`v + K_W·água`). A tinta seca não tem voto — a aguada
/// sobre o óleo seco é água (E10e2 §E3: o Bingham prendia-a); o óleo grosso sobre papel limpo é óleo.
#[inline]
#[must_use]
pub fn mistura(m_old: f32, molhado_old: f64, m_novo: f32, v: f64, agua: f64) -> f32 {
    let novo = v + crate::painter::receita::K_W * agua.max(0.0);
    let total = molhado_old.max(0.0) + novo;
    if m_old == m_novo {
        return m_old;
    }
    if total <= 0.0 {
        return m_novo;
    }
    (f64::from(m_old) + (f64::from(m_novo) - f64::from(m_old)) * novo / total) as f32
}

#[cfg(test)]
#[path = "meio_tests.rs"]
mod tests;
