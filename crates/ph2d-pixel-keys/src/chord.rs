//! A tecla com os modificadores, e a forma de texto dela (a do Aseprite: `Ctrl+Shift+X`).

use std::fmt;
use std::str::FromStr;

use winit::keyboard::{KeyCode, KeyCode as K};

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

/// Os nomes estáveis das teclas (gravados no ficheiro do utilizador: não mudar). Nomear não é tomar:
/// o `Tab` sozinho é do modo do app (`RESERVED_GLOBAL`, recusado por `Keymap::assign` e por
/// `input_teclas::key_with`), só o acorde `Space+Tab` é do Pixel — prova `bare_tab_stays_the_modes`.
const NAMES: &[(K, &str)] = &[
    (K::KeyA, "A"),
    (K::KeyB, "B"),
    (K::KeyC, "C"),
    (K::KeyD, "D"),
    (K::KeyE, "E"),
    (K::KeyF, "F"),
    (K::KeyG, "G"),
    (K::KeyH, "H"),
    (K::KeyI, "I"),
    (K::KeyJ, "J"),
    (K::KeyK, "K"),
    (K::KeyL, "L"),
    (K::KeyM, "M"),
    (K::KeyN, "N"),
    (K::KeyO, "O"),
    (K::KeyP, "P"),
    (K::KeyQ, "Q"),
    (K::KeyR, "R"),
    (K::KeyS, "S"),
    (K::KeyT, "T"),
    (K::KeyU, "U"),
    (K::KeyV, "V"),
    (K::KeyW, "W"),
    (K::KeyX, "X"),
    (K::KeyY, "Y"),
    (K::KeyZ, "Z"),
    (K::Digit0, "0"),
    (K::Digit1, "1"),
    (K::Digit2, "2"),
    (K::Digit3, "3"),
    (K::Digit4, "4"),
    (K::Digit5, "5"),
    (K::Digit6, "6"),
    (K::Digit7, "7"),
    (K::Digit8, "8"),
    (K::Digit9, "9"),
    (K::F1, "F1"),
    (K::F2, "F2"),
    (K::F3, "F3"),
    (K::F4, "F4"),
    (K::F5, "F5"),
    (K::F6, "F6"),
    (K::F7, "F7"),
    (K::F8, "F8"),
    (K::F9, "F9"),
    (K::F10, "F10"),
    (K::F11, "F11"),
    (K::F12, "F12"),
    (K::Enter, "Enter"),
    (K::NumpadEnter, "NumpadEnter"),
    (K::Escape, "Esc"),
    (K::Tab, "Tab"),
    (K::Space, "Space"),
    (K::Delete, "Del"),
    (K::Backspace, "Backspace"),
    (K::Insert, "Ins"),
    (K::Home, "Home"),
    (K::End, "End"),
    (K::PageUp, "PgUp"),
    (K::PageDown, "PgDn"),
    (K::ArrowLeft, "Left"),
    (K::ArrowRight, "Right"),
    (K::ArrowUp, "Up"),
    (K::ArrowDown, "Down"),
    (K::Quote, "'"),
    (K::Minus, "-"),
    (K::Equal, "="),
    (K::BracketLeft, "["),
    (K::BracketRight, "]"),
    (K::Comma, ","),
    (K::Period, "."),
    (K::Backquote, "`"),
    (K::Slash, "/"),
    (K::Semicolon, ";"),
    (K::Backslash, "\\"),
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
