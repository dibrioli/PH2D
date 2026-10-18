//! **Os CAMPOS do estilo** — que campo um gesto do painel mudou, e em que campo a seleção múltipla
//! difere (plano 31, `docs/UI_New_and_Simple/vetor_nova_ui/31_plano_F20_misto_e_espessura_do_documento.md`).
//!
//! A MESMA máscara responde às duas perguntas, de propósito: o que o painel mostra como «misto» é
//! exatamente o que um gesto pode escrever campo a campo. Um bit numa e não na outra seria um
//! controlo que diz «misto» e, ao ser tocado, apaga os campos vizinhos (o defeito que o plano mediu).

use ph2d_vec_scene::MAX_DASH_PAIRS;

/// Um conjunto de campos do estilo.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleFields(u32);

impl StyleFields {
    pub const NONE: Self = Self(0);
    pub const FILL_RGB: Self = Self(1 << 0);
    pub const FILL_A: Self = Self(1 << 1);
    pub const STROKE_RGB: Self = Self(1 << 2);
    pub const STROKE_A: Self = Self(1 << 3);
    pub const WIDTH: Self = Self(1 << 4);
    pub const CAP: Self = Self(1 << 5);
    pub const JOIN: Self = Self(1 << 6);
    pub const ALIGN: Self = Self(1 << 7);
    pub const MARKER_START: Self = Self(1 << 8);
    pub const MARKER_END: Self = Self(1 << 9);
    pub const MARKER_SCALE_START: Self = Self(1 << 10);
    pub const MARKER_SCALE_END: Self = Self(1 << 11);
    pub const MARKER_ROUND: Self = Self(1 << 12);
    /// Os seis números do tracejado começam aqui: `[traço, vão]` do par `k` = bits `13 + 2k + gap`.
    const DASH_FIRST: u32 = 13;
    /// Todos os campos do traço (o que um `StrokeSpec` inteiro escreve).
    pub const STROKE: Self = Self(!(Self::FILL_RGB.0 | Self::FILL_A.0) & Self::ALL.0);
    pub const FILL: Self = Self(Self::FILL_RGB.0 | Self::FILL_A.0);
    pub const ALL: Self = Self((1 << (Self::DASH_FIRST + 2 * MAX_DASH_PAIRS as u32)) - 1);

    /// O número `gap` (`false` = traço, `true` = vão) do par `k` do tracejado.
    #[must_use]
    pub const fn dash(k: usize, gap: bool) -> Self {
        Self(1 << (Self::DASH_FIRST + 2 * k as u32 + gap as u32))
    }

    /// O tamanho da ponta do `slot` (`0` = começo, `1` = fim).
    #[must_use]
    pub const fn marker_scale(slot: usize) -> Self {
        if slot == 0 {
            Self::MARKER_SCALE_START
        } else {
            Self::MARKER_SCALE_END
        }
    }

    /// Todos os bits de `other` estão aqui (e `other` não é vazio)?
    #[must_use]
    pub const fn has(self, other: Self) -> bool {
        other.0 != 0 && self.0 & other.0 == other.0
    }

    /// Algum bit de `other` está aqui?
    #[must_use]
    pub const fn any(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Estes campos, menos os de `other`.
    #[must_use]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

impl std::ops::BitOr for StyleFields {
    type Output = Self;
    fn bitor(self, o: Self) -> Self {
        Self(self.0 | o.0)
    }
}

impl std::ops::BitOrAssign for StyleFields {
    fn bitor_assign(&mut self, o: Self) {
        self.0 |= o.0;
    }
}

impl std::ops::BitAnd for StyleFields {
    type Output = Self;
    fn bitand(self, o: Self) -> Self {
        Self(self.0 & o.0)
    }
}

#[cfg(test)]
mod tests {
    use super::StyleFields as F;

    #[test]
    fn cada_campo_tem_o_seu_bit() {
        let mut todos = vec![
            F::FILL_RGB,
            F::FILL_A,
            F::STROKE_RGB,
            F::STROKE_A,
            F::WIDTH,
            F::CAP,
            F::JOIN,
            F::ALIGN,
            F::MARKER_START,
            F::MARKER_END,
            F::MARKER_SCALE_START,
            F::MARKER_SCALE_END,
            F::MARKER_ROUND,
        ];
        for k in 0..ph2d_vec_scene::MAX_DASH_PAIRS {
            todos.push(F::dash(k, false));
            todos.push(F::dash(k, true));
        }
        let mut uniao = F::NONE;
        for (i, a) in todos.iter().enumerate() {
            assert!(!uniao.any(*a), "o bit {i} repete um anterior");
            uniao |= *a;
        }
        assert_eq!(uniao, F::ALL, "ALL é a união exata dos campos");
        assert_eq!(F::STROKE | F::FILL, F::ALL);
        assert!(!F::STROKE.any(F::FILL));
    }
}
