//! A escala de nascença de um [`super::Style`] ([`BirthScale`], formato 17) e a espessura que a
//! acompanha (cortado do `element.rs` pelo tecto de LOC).

use serde::{Deserialize, Serialize};

use super::Style;

/// ⭐ **A escala do ecrã em que o elemento NASCEU** (decisão do dono, 21/10: «nasce no tamanho do
/// ecrã»): `2^(k/4)` unidades do mundo por px. O traço à mão gera-se à escala do ecrã de quem o
/// desenhou e fica assim (aproximar mostra-o maior, como um desenho); a espessura de mundo nasce com
/// ela ([`Style::reborn`]). `0` = zoom 1. Um byte em quartos de oitava: o [`Style`] fica nos 40 bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BirthScale(pub i8);

impl BirthScale {
    /// `2^±12` unidades por px: o que um `i8` em quartos de oitava guarda com folga de 2,6×.
    const STEPS: f64 = 48.0;

    /// A escala de quem desenha com UM px de ecrã a medir `px` unidades do mundo (`1 / zoom`).
    #[must_use]
    pub fn of_px(px: f64) -> Self {
        if !(px.is_finite() && px > 0.0) {
            return Self::default();
        }
        Self((4.0 * px.log2()).round().clamp(-Self::STEPS, Self::STEPS) as i8)
    }

    /// Unidades do mundo por px de ecrã.
    #[must_use]
    pub fn factor(self) -> f64 {
        (f64::from(self.0) / 4.0).exp2()
    }
}

impl Style {
    /// Renasce à escala `birth`: a espessura de MUNDO acompanha-a, a que se escolheu fica.
    pub fn reborn(&mut self, birth: BirthScale) {
        self.stroke_width *= birth.factor() / self.birth.factor();
        self.birth = birth;
    }

    /// A espessura como a barra a oferece: a do ecrã em que nasceu.
    #[must_use]
    pub fn chosen_width(&self) -> f64 {
        self.stroke_width / self.birth.factor()
    }

    /// Escolhe a espessura `w` da barra, à escala em que nasceu.
    pub fn choose_width(&mut self, w: f64) {
        self.stroke_width = w * self.birth.factor();
    }
}
