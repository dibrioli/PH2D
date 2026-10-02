//! ⭐⭐ **A escala da interface INTEIRA** (ordem do dono, 2026-10-02) — o *Resolution Scale* do
//! Blender, o `display_scale` da Godot. Spec: `docs/UI_New_and_Simple/spec/05_a_escala_da_interface.md`.
//!
//! Degraus como a Godot (corrida em 02/10: `Auto · 75 · 100 · 125 · 150 · 175 · 200 · Custom`): o
//! artista escolhe, não arrasta. ⚠️ `100 %` é `1,0` EXACTO — a fábrica não mexe um píxel. Os dois
//! degraus abaixo (`80`, `90`) são pedido do dono (02/10): mais área de trabalho num ecrã grande.

/// Um degrau da escala da interface.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum UiScale {
    P80,
    P90,
    /// A de sempre.
    #[default]
    P100,
    P125,
    P150,
    P175,
    P200,
}

impl UiScale {
    /// Os sete, pela ordem do menu.
    pub const ALL: [Self; 7] = [
        Self::P80,
        Self::P90,
        Self::P100,
        Self::P125,
        Self::P150,
        Self::P175,
        Self::P200,
    ];

    /// A percentagem, como o menu a escreve.
    #[must_use]
    pub const fn percent(self) -> u16 {
        match self {
            Self::P80 => 80,
            Self::P90 => 90,
            Self::P100 => 100,
            Self::P125 => 125,
            Self::P150 => 150,
            Self::P175 => 175,
            Self::P200 => 200,
        }
    }

    /// O factor: `1,0` exacto na fábrica.
    #[must_use]
    pub fn factor(self) -> f32 {
        f32::from(self.percent()) / 100.0
    }

    /// O nome estável no ficheiro de preferências (`"125"`).
    #[must_use]
    pub const fn wire(self) -> &'static str {
        match self {
            Self::P80 => "80",
            Self::P90 => "90",
            Self::P100 => "100",
            Self::P125 => "125",
            Self::P150 => "150",
            Self::P175 => "175",
            Self::P200 => "200",
        }
    }

    /// Inverso de [`Self::wire`].
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|z| z.wire() == s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fábrica é a identidade AO BIT, e os degraus sobem pela ordem do menu.
    #[test]
    fn the_factory_is_one_and_the_steps_climb() {
        assert_eq!(UiScale::default().factor().to_bits(), 1.0_f32.to_bits());
        let f = UiScale::ALL.map(UiScale::factor);
        assert!(f.windows(2).all(|w| w[0] < w[1]), "{f:?}");
        for z in UiScale::ALL {
            assert_eq!(UiScale::from_wire(z.wire()), Some(z));
        }
    }
}
