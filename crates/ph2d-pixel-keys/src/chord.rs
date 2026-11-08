//! A tecla com os modificadores, e a forma de texto dela (a do Aseprite: `Ctrl+Shift+X`).

use std::fmt;
use std::str::FromStr;

use winit::keyboard::KeyCode;

/// Uma tecla premida com os modificadores; `space` = Space seguro (o `Space+Tab` do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chord {
    pub key: KeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub space: bool,
}

impl Chord {
    #[must_use]
    pub const fn new(key: KeyCode) -> Self {
        Self {
            key,
            ctrl: false,
            alt: false,
            shift: false,
            space: false,
        }
    }
    #[must_use]
    pub const fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }
    #[must_use]
    pub const fn alt(mut self) -> Self {
        self.alt = true;
        self
    }
    #[must_use]
    pub const fn shift(mut self) -> Self {
        self.shift = true;
        self
    }
    #[must_use]
    pub const fn space(mut self) -> Self {
        self.space = true;
        self
    }
}

/// Os nomes estáveis das teclas (gravados no ficheiro do utilizador: não mudar).
const NAMES: &[(KeyCode, &str)] = &[
    (KeyCode::KeyA, "A"),
    (KeyCode::KeyB, "B"),
    (KeyCode::KeyC, "C"),
    (KeyCode::KeyD, "D"),
    (KeyCode::KeyE, "E"),
    (KeyCode::KeyF, "F"),
    (KeyCode::KeyG, "G"),
    (KeyCode::KeyH, "H"),
    (KeyCode::KeyI, "I"),
    (KeyCode::KeyJ, "J"),
    (KeyCode::KeyK, "K"),
    (KeyCode::KeyL, "L"),
    (KeyCode::KeyM, "M"),
    (KeyCode::KeyN, "N"),
    (KeyCode::KeyO, "O"),
    (KeyCode::KeyP, "P"),
    (KeyCode::KeyQ, "Q"),
    (KeyCode::KeyR, "R"),
    (KeyCode::KeyS, "S"),
    (KeyCode::KeyT, "T"),
    (KeyCode::KeyU, "U"),
    (KeyCode::KeyV, "V"),
    (KeyCode::KeyW, "W"),
    (KeyCode::KeyX, "X"),
    (KeyCode::KeyY, "Y"),
    (KeyCode::KeyZ, "Z"),
    (KeyCode::Digit0, "0"),
    (KeyCode::Digit1, "1"),
    (KeyCode::Digit2, "2"),
    (KeyCode::Digit3, "3"),
    (KeyCode::Digit4, "4"),
    (KeyCode::Digit5, "5"),
    (KeyCode::Digit6, "6"),
    (KeyCode::Digit7, "7"),
    (KeyCode::Digit8, "8"),
    (KeyCode::Digit9, "9"),
    (KeyCode::F1, "F1"),
    (KeyCode::F2, "F2"),
    (KeyCode::F3, "F3"),
    (KeyCode::F4, "F4"),
    (KeyCode::F5, "F5"),
    (KeyCode::F6, "F6"),
    (KeyCode::F7, "F7"),
    (KeyCode::F8, "F8"),
    (KeyCode::F9, "F9"),
    (KeyCode::F10, "F10"),
    (KeyCode::F11, "F11"),
    (KeyCode::F12, "F12"),
    (KeyCode::Enter, "Enter"),
    (KeyCode::NumpadEnter, "NumpadEnter"),
    (KeyCode::Escape, "Esc"),
    (KeyCode::Tab, "Tab"),
    (KeyCode::Space, "Space"),
    (KeyCode::Delete, "Del"),
    (KeyCode::Backspace, "Backspace"),
    (KeyCode::Insert, "Ins"),
    (KeyCode::Home, "Home"),
    (KeyCode::End, "End"),
    (KeyCode::PageUp, "PgUp"),
    (KeyCode::PageDown, "PgDn"),
    (KeyCode::ArrowLeft, "Left"),
    (KeyCode::ArrowRight, "Right"),
    (KeyCode::ArrowUp, "Up"),
    (KeyCode::ArrowDown, "Down"),
    (KeyCode::Quote, "'"),
    (KeyCode::Minus, "-"),
    (KeyCode::Equal, "="),
    (KeyCode::BracketLeft, "["),
    (KeyCode::BracketRight, "]"),
    (KeyCode::Comma, ","),
    (KeyCode::Period, "."),
    (KeyCode::Backquote, "`"),
    (KeyCode::Slash, "/"),
    (KeyCode::Semicolon, ";"),
    (KeyCode::Backslash, "\\"),
];

/// O nome estável da tecla, ou `None` se ela não está na tabela.
#[must_use]
pub fn key_name(key: KeyCode) -> Option<&'static str> {
    NAMES.iter().find(|(k, _)| *k == key).map(|(_, n)| *n)
}

/// A tecla de um nome (sem distinguir maiúsculas nas letras).
#[must_use]
pub fn key_of_name(name: &str) -> Option<KeyCode> {
    NAMES
        .iter()
        .find(|(_, n)| n.eq_ignore_ascii_case(name))
        .map(|(k, _)| *k)
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (on, m) in [
            (self.ctrl, "Ctrl+"),
            (self.alt, "Alt+"),
            (self.shift, "Shift+"),
            (self.space, "Space+"),
        ] {
            if on {
                f.write_str(m)?;
            }
        }
        f.write_str(key_name(self.key).unwrap_or("?"))
    }
}

/// Porque um texto não é uma tecla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChordError {
    Empty,
    UnknownKey(String),
    UnknownModifier(String),
}

impl FromStr for Chord {
    type Err = ChordError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ChordError::Empty);
        }
        let mut parts: Vec<&str> = s.split('+').map(str::trim).collect();
        let last = parts.pop().unwrap_or_default();
        let key = key_of_name(last).ok_or_else(|| ChordError::UnknownKey(last.to_owned()))?;
        let mut c = Self::new(key);
        for m in parts {
            match m.to_ascii_lowercase().as_str() {
                "ctrl" => c.ctrl = true,
                "alt" => c.alt = true,
                "shift" => c.shift = true,
                "space" => c.space = true,
                _ => return Err(ChordError::UnknownModifier(m.to_owned())),
            }
        }
        Ok(c)
    }
}
