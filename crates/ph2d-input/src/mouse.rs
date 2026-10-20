//! **OS BOTÕES DO RATO** — a forma do [`crate::keyboard::KeyboardState`]: em baixo agora, e as
//! bordas que aconteceram desde o `begin_frame` (eventos, para um clique dentro de um quadro não se
//! perder). A POSIÇÃO não mora aqui: ela só tem sentido depois da câmara, e quem a tem é a shell.

/// Um botão do rato.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    /// Os três, pela ordem.
    pub const ALL: [MouseButton; 3] = [Self::Left, Self::Right, Self::Middle];

    /// O nome que um script escreve (`ph2d.mouse_down("left")`).
    #[must_use]
    pub fn script_name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Middle => "middle",
        }
    }

    /// O inverso; `None` = nome desconhecido.
    #[must_use]
    pub fn from_script_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.script_name() == name)
    }

    fn bit(self) -> u8 {
        match self {
            Self::Left => 1,
            Self::Right => 2,
            Self::Middle => 4,
        }
    }
}

/// Retrato dos botões.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MouseState {
    down: u8,
    went_down: u8,
    went_up: u8,
}

impl MouseState {
    /// Abre um quadro: esquece as bordas do anterior.
    pub fn begin_frame(&mut self) {
        self.went_down = 0;
        self.went_up = 0;
    }

    /// O botão desceu.
    pub fn handle_button_down(&mut self, b: MouseButton) {
        if self.down & b.bit() == 0 {
            self.down |= b.bit();
            self.went_down |= b.bit();
        }
    }

    /// O botão subiu.
    pub fn handle_button_up(&mut self, b: MouseButton) {
        if self.down & b.bit() != 0 {
            self.down &= !b.bit();
            self.went_up |= b.bit();
        }
    }

    /// Larga tudo (a janela perdeu o foco) — cada botão largado é uma borda de subida.
    pub fn release_all(&mut self) {
        self.went_up |= self.down;
        self.down = 0;
    }

    /// Está em baixo agora.
    #[must_use]
    pub fn held(&self, b: MouseButton) -> bool {
        self.down & b.bit() != 0
    }

    /// Desceu neste quadro.
    #[must_use]
    pub fn pressed(&self, b: MouseButton) -> bool {
        self.went_down & b.bit() != 0
    }

    /// Subiu neste quadro.
    #[must_use]
    pub fn released(&self, b: MouseButton) -> bool {
        self.went_up & b.bit() != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_inside_one_frame_gives_both_edges() {
        let mut m = MouseState::default();
        m.begin_frame();
        m.handle_button_down(MouseButton::Left);
        m.handle_button_up(MouseButton::Left);
        assert!(!m.held(MouseButton::Left));
        assert!(m.pressed(MouseButton::Left) && m.released(MouseButton::Left));
        assert!(!m.pressed(MouseButton::Right));
        m.begin_frame();
        assert!(!m.pressed(MouseButton::Left) && !m.released(MouseButton::Left));
    }

    #[test]
    fn names_round_trip() {
        for b in MouseButton::ALL {
            assert_eq!(MouseButton::from_script_name(b.script_name()), Some(b));
        }
        assert_eq!(MouseButton::from_script_name("Left"), None);
    }
}
