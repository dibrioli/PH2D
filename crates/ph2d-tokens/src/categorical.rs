//! ⭐ A paleta CATEGÓRICA: doze cores de DADO, uma por categoria (o Input Map pinta cada acção pelo
//! `ActionId`, `docs/InputMap/01_plano.md` §3). A ordem é a da maquete B para o mapa de fábrica:
//! move_left, move_right, jump, down, dash, grab, move_up, move_down, fire, reload, laser, e a 12.ª.
//!
//! Regra dos valores (`docs/design/tokens.json`, `categorical-*`): o ESCURO é a maquete byte a byte
//! (a 12.ª é a cor de croma baixo mais afastada das onze em OKLab); o CLARO guarda matiz e croma (cortado
//! ao gamut) e desce o L até ≥ 3,2:1 sobre o `bg-1..3` de Sunstone e Blueprint. Os temas derivados
//! emprestam (escuro = Forge, claro = Sunstone). Medido: par mais próximo 0,060 OKLab (escuro), 0,044 (claro).

use crate::color::ColorToken as T;

/// As doze cores, pelo índice da categoria.
pub const CATEGORICAL: [T; 12] = [
    T::Categorical01,
    T::Categorical02,
    T::Categorical03,
    T::Categorical04,
    T::Categorical05,
    T::Categorical06,
    T::Categorical07,
    T::Categorical08,
    T::Categorical09,
    T::Categorical10,
    T::Categorical11,
    T::Categorical12,
];

/// A cor da categoria `i` (dá a volta depois da 12.ª).
#[must_use]
pub const fn categorical(i: u32) -> T {
    CATEGORICAL[(i % 12) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{Color, srgb_to_oklch};
    use crate::derive::{CardTone, card_tone};
    use crate::theme::Theme;

    fn lab(c: Color) -> [f64; 3] {
        let (l, ch, h) = srgb_to_oklch(c.r, c.g, c.b);
        let h = h.to_radians();
        [l, ch * h.cos(), ch * h.sin()]
    }

    /// Pares distintos em TODO tema (≥ 0,04 OKLab) e ≥ 3:1 sobre as superfícies dos temas de base.
    #[test]
    fn as_doze_cores_sao_distintas_e_legiveis_em_cada_tema() {
        for theme in Theme::ALL {
            let cores: Vec<Color> = CATEGORICAL.iter().map(|t| t.resolve(theme)).collect();
            let mut menor = f64::MAX;
            for (i, &a) in cores.iter().enumerate() {
                for (j, &b) in cores.iter().enumerate().skip(i + 1) {
                    assert_ne!(a, b, "{theme:?}: {i} e {j} são a mesma cor");
                    let (p, q) = (lab(a), lab(b));
                    let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2))
                        .sqrt();
                    menor = menor.min(d);
                    assert!(d >= 0.04, "{theme:?}: {i} × {j} a {d:.4} OKLab");
                }
            }
            // As superfícies onde a cor pousa: o painel e o cartão (fundo, campo, segmento aceso).
            let base = !matches!(
                theme,
                Theme::PlumberRed | Theme::PlumberGreen | Theme::Sunset | Theme::Candy
            );
            let fundos = [
                ("panel-bg", T::PanelBg.resolve(theme)),
                ("bg-1", T::Bg1.resolve(theme)),
                ("card", card_tone(theme, CardTone::Surface)),
                ("field", card_tone(theme, CardTone::Field)),
                ("raised", card_tone(theme, CardTone::Raised)),
            ];
            let mut pior = f64::MAX;
            for (nome, f) in fundos {
                for (i, c) in cores.iter().enumerate() {
                    let r = f.contrast_ratio(c);
                    pior = pior.min(r);
                    if base {
                        assert!(r >= 3.0, "{theme:?}: cor {i} sobre {nome} a {r:.2}:1");
                    }
                }
            }
            eprintln!(
                "categórica {theme:?}: par mais próximo {menor:.4}, pior contraste {pior:.2}"
            );
        }
    }

    /// O escuro é a maquete B do Input Map, byte a byte (`critica_ph2d.html`, `#inputmap`).
    #[test]
    fn o_escuro_e_a_maquete() {
        let maquete = [
            0x6C_CB_FA, 0x49_E0_AE, 0xFF_CF_4D, 0xFF_9F_5A, 0xFF_7A_A8, 0xB7_9C_FF, 0x9B_E3_6A,
            0xF4_A3_FF, 0xFF_6B_6B, 0x8F_B8_FF, 0x7F_D1_C7,
        ];
        for (i, hex) in maquete.into_iter().enumerate() {
            assert_eq!(
                CATEGORICAL[i].resolve(Theme::Forge),
                Color::from_hex(hex),
                "cor {i}"
            );
        }
    }

    #[test]
    fn o_indice_da_a_volta() {
        assert_eq!(categorical(0), T::Categorical01);
        assert_eq!(categorical(11), T::Categorical12);
        assert_eq!(categorical(12), T::Categorical01);
        assert_eq!(categorical(u32::MAX), CATEGORICAL[(u32::MAX % 12) as usize]);
    }
}
