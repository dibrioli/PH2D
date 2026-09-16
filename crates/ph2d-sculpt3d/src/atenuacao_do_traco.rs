//! ⭐⭐ **O PASSO E A ATENUAÇÃO DO TRAÇO ARRASTADO** do [`Verb::Plane`]
//! (`SPEC_pincel_de_plano.md` §14.4).
//!
//! Num traço arrastado os dabs sobrepõem-se, e cada um é enfraquecido para que a soma não dependa
//! do espaçamento. O factor é o inverso do máximo, sobre DEZ fases, da soma da curva de queda nas
//! posições dos dabs vizinhos:
//!
//! ```text
//! a = 1 / max_fase Σ_j curva(|fase − 1 + j·h|)     h = espaçamento_% / 50,  j < ⌊100 / espaçamento_%⌋
//! força por dab = força² · pressão · (1 + a) / 2      (e 0,5 · força² · pressão · a no afastar invertido)
//! ```
//!
//! | curva | espaçamento | `a` | `(1 + a)/2` |
//! |---|---|---|---|
//! | suave | `7 %` (o perfil *aparar*) | `0,140` | **`0,570`** |
//! | suave | `10 %` | `0,200` | `0,600` |
//! | constante | `7 %` | `0,071` | `0,536` |
//!
//! ⚠️ **As fases são DEZ pontos** (`0; 0,1; …; 0,9`), não um máximo no contínuo — é essa
//! discretização que dá a tabela. A curva é a de queda do pincel **sem** a dureza.
//!
//! ⚠️ **Só o pincel de plano, e só arrastado.** A porta por script (a bancada, o oráculo por
//! script) dá os dabs um a um com o factor em `1`, e os outros verbos desta casa não têm esta lei —
//! dá-la a eles mudaria paridades medidas contra as referências deles.
//!
//! ⚠️ **O passo** é `raio × espaçamento_% / 50` — `0,14 R` a `7 %` —, contra os `0,15 R` que a
//! casa dá a todos os outros verbos. Escrito como o alvo o escreve e não como `0,14`: a `50` px de
//! raio `50 × 7 / 50` é **exactamente** `7`, e `0,14 × 50` não é.

use crate::{Brush, Falloff, PlanoInversao, Verb};

/// O espaçamento do pincel de plano, em % do DIÂMETRO — o perfil *aparar* do alvo (espec §14.2).
pub const ESPACAMENTO_DO_PLANO_PCT: f32 = 7.0;

/// **O passo entre dois dabs de um traço arrastado**, na régua em que `raio` vem (a app dá pixels).
#[must_use]
pub fn passo_do_traco(verb: Verb, raio: f32) -> f32 {
    if verb == Verb::Plane {
        raio * ESPACAMENTO_DO_PLANO_PCT / 50.0
    } else {
        crate::min_spacing(raio)
    }
}

/// **O factor de atenuação `a`** de uma curva a um espaçamento (em % do diâmetro). `1` fora de
/// `0 < espaçamento < 100`, onde a lei não existe.
#[must_use]
pub fn atenuacao_por_espacamento(curva: Falloff, espacamento_pct: f32) -> f32 {
    if !(espacamento_pct > 0.0 && espacamento_pct < 100.0) {
        return 1.0;
    }
    let h = espacamento_pct / 50.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let vizinhos = (100.0 / espacamento_pct).floor() as u32;
    let mut maior = 0.0f32;
    for k in 0..10u8 {
        let fase = f32::from(k) / 10.0;
        let mut soma = 0.0f32;
        for j in 0..vizinhos {
            #[allow(clippy::cast_precision_loss)]
            let x = (fase - 1.0 + j as f32 * h).abs();
            if x < 1.0 {
                soma += curva.weight(x);
            }
        }
        maior = maior.max(soma);
    }
    if maior > 0.0 { maior.recip() } else { 1.0 }
}

impl Brush {
    /// **O factor por dab do traço** (espec §14.4) — `1` fora do arrasto e fora do pincel de plano.
    ///
    /// ⚠️ Calcula-se UMA vez por dab (o plano guarda-o): são `140` avaliações da curva, e o alvo
    /// por-vértice não as pode pagar a cada vértice.
    #[must_use]
    pub fn factor_do_traco(&self) -> f32 {
        if !self.traco_arrastado || self.verb != Verb::Plane {
            return 1.0;
        }
        let a = atenuacao_por_espacamento(self.falloff, ESPACAMENTO_DO_PLANO_PCT);
        let afastar_invertido = self.plano_inversao == PlanoInversao::Afastar
            && self.invert
            && self.verb.honours_invert();
        if afastar_invertido {
            0.5 * a
        } else {
            (1.0 + a) / 2.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{atenuacao_por_espacamento, passo_do_traco};
    use crate::{Brush, Falloff, Verb};

    /// ⭐ **G-17, a metade da LEI** — a tabela da espec §14.4, ao milésimo que ela publica.
    #[test]
    fn a_atenuacao_do_traco_e_a_da_lei() {
        for (curva, pct, a) in [
            (Falloff::Smooth, 7.0, 0.140),
            (Falloff::Smooth, 10.0, 0.200),
            (Falloff::Constant, 7.0, 0.071),
        ] {
            let lida = atenuacao_por_espacamento(curva, pct);
            assert!(
                (lida - a).abs() < 5e-4,
                "{curva:?} a {pct} %: {lida} contra {a}"
            );
        }
        assert_eq!(atenuacao_por_espacamento(Falloff::Smooth, 100.0), 1.0);
        let pincel = Brush {
            verb: Verb::Plane,
            falloff: Falloff::Smooth,
            traco_arrastado: true,
            ..Brush::default()
        };
        assert!(
            (pincel.factor_do_traco() - 0.570).abs() < 5e-4,
            "{}",
            pincel.factor_do_traco()
        );
        let por_script = Brush {
            traco_arrastado: false,
            ..pincel.clone()
        };
        assert_eq!(
            por_script.factor_do_traco(),
            1.0,
            "a porta por script ganhou atenuacao"
        );
        let outro = Brush {
            verb: Verb::Draw,
            ..pincel
        };
        assert_eq!(
            outro.factor_do_traco(),
            1.0,
            "a lei do plano chegou a outro verbo"
        );
    }

    /// O passo do plano é `7 %` do diâmetro, EXACTO a 50 px; os outros verbos ficam nos `0,15 R`.
    #[test]
    fn o_passo_do_plano_e_sete_por_cento_do_diametro() {
        assert_eq!(passo_do_traco(Verb::Plane, 50.0), 7.0);
        assert_eq!(passo_do_traco(Verb::Draw, 50.0), crate::min_spacing(50.0));
    }
}
