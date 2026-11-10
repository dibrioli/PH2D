//! A imagem do mapa é a tabela do `ph2d_input` — nem uma tecla a mais, nem uma a menos.

use super::*;
use KeyCode as K;
use ph2d_input::Key;
use std::collections::BTreeMap;

/// Toda tecla física que o mapa traduz (a do teclado de notebook da foto do dono, §13.1).
const MAPPED: &[KeyCode] = &[
    K::KeyA,
    K::KeyB,
    K::KeyC,
    K::KeyD,
    K::KeyE,
    K::KeyF,
    K::KeyG,
    K::KeyH,
    K::KeyI,
    K::KeyJ,
    K::KeyK,
    K::KeyL,
    K::KeyM,
    K::KeyN,
    K::KeyO,
    K::KeyP,
    K::KeyQ,
    K::KeyR,
    K::KeyS,
    K::KeyT,
    K::KeyU,
    K::KeyV,
    K::KeyW,
    K::KeyX,
    K::KeyY,
    K::KeyZ,
    K::Digit0,
    K::Digit1,
    K::Digit2,
    K::Digit3,
    K::Digit4,
    K::Digit5,
    K::Digit6,
    K::Digit7,
    K::Digit8,
    K::Digit9,
    K::F1,
    K::F2,
    K::F3,
    K::F4,
    K::F5,
    K::F6,
    K::F7,
    K::F8,
    K::F9,
    K::F10,
    K::F11,
    K::F12,
    K::Tab,
    K::Enter,
    K::Space,
    K::Escape,
    K::Backspace,
    K::Delete,
    K::ArrowUp,
    K::ArrowDown,
    K::ArrowLeft,
    K::ArrowRight,
    K::ShiftLeft,
    K::ShiftRight,
    K::ControlLeft,
    K::ControlRight,
    K::AltLeft,
    K::AltRight,
    K::Home,
    K::BracketLeft,
    K::BracketRight,
    K::Comma,
    K::Period,
    K::Backquote,
    K::Minus,
    K::Equal,
    K::Backslash,
    K::Semicolon,
    K::Quote,
    K::Slash,
    K::Insert,
    K::End,
    K::PageUp,
    K::PageDown,
    K::CapsLock,
    K::NumLock,
    K::Numpad0,
    K::Numpad1,
    K::Numpad2,
    K::Numpad3,
    K::Numpad4,
    K::Numpad5,
    K::Numpad6,
    K::Numpad7,
    K::Numpad8,
    K::Numpad9,
    K::NumpadDecimal,
    K::NumpadDivide,
    K::NumpadMultiply,
    K::NumpadSubtract,
    K::NumpadAdd,
    K::NumpadEnter,
];

/// ⭐ O mapa é injectivo e a imagem dele é EXACTAMENTE `Key::all_named()`: toda tecla com nome de
/// script tem uma tecla física que a dá, e nenhuma tecla física cai num código sem rótulo.
#[test]
fn the_map_covers_exactly_the_named_keys() {
    let mut image: BTreeMap<u32, KeyCode> = BTreeMap::new();
    for &code in MAPPED {
        let v = winit_to_input_keycode(code).unwrap_or_else(|| panic!("{code:?} sem tradução"));
        if let Some(prev) = image.insert(v, code) {
            panic!("{prev:?} e {code:?} dão o mesmo código {v:#X}");
        }
        assert!(Key(v).script_name().is_some(), "{code:?} → {v:#X} sem nome");
    }
    let named: Vec<u32> = Key::all_named().map(|k| k.0).collect();
    let mut sorted = named.clone();
    sorted.sort_unstable();
    assert_eq!(image.keys().copied().collect::<Vec<_>>(), sorted);
    assert_eq!(
        MAPPED.len(),
        26 + 10 + 12 + 50,
        "a população do teclado de notebook"
    );
}

/// ⛔ Numpad 7 ≠ 7 ≠ Home e Numpad Enter ≠ Enter (antes caía no `0x0D`); os dois Enter confirmam.
#[test]
fn the_numpad_has_its_own_codes() {
    let key = |c| Key(winit_to_input_keycode(c).expect("mapeada"));
    assert_eq!(key(K::Numpad7).label(), "Numpad 7");
    assert_eq!(key(K::Digit7).label(), "7");
    assert_eq!(key(K::Home).label(), "Home");
    assert_eq!(key(K::NumpadEnter), Key::NUMPAD_ENTER);
    assert_eq!(key(K::Enter), Key::ENTER);
    assert!(key(K::NumpadEnter).is_enter_like() && key(K::Enter).is_enter_like());
    assert_eq!(key(K::Slash).script_name().as_deref(), Some("/"));
    assert_eq!(key(K::NumpadDivide).script_name().as_deref(), Some("kp/"));
}

/// ⛔ As do sistema nunca chegam (medido: o desktop não as entrega); tampouco as que o notebook não tem.
#[test]
fn system_keys_and_unknown_keys_are_none() {
    for code in [
        K::SuperLeft,
        K::SuperRight,
        K::Fn,
        K::FnLock,
        K::PrintScreen,
        K::Meta,
        K::ContextMenu,
        K::IntlRo,
        K::ScrollLock,
        K::Pause,
        K::F13,
    ] {
        assert_eq!(winit_to_input_keycode(code), None, "{code:?}");
    }
    for k in Key::system_keys() {
        assert!(
            MAPPED
                .iter()
                .all(|&c| winit_to_input_keycode(c) != Some(k.0)),
            "a do sistema {} é ligável",
            k.label()
        );
    }
}
