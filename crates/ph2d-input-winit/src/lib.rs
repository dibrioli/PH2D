//! **A tecla FÍSICA do winit no espaço `u32` do app** — o normalizador TOTAL que o Input Map usa
//! (plano `docs/Vector Module/30_plano_input_map.md` W5; teclado de notebook: `docs/InputMap/01_plano.md`
//! §13.1). Saiu da shell (`shells/desktop/src/keymap.rs`) na W1 da onda 2: as linhas novas do numérico
//! e da pontuação pagam-se aqui, não na catraca `the_shell_only_shrinks`.
//!
//! ⚠️ **Por que existe um segundo normalizador, e não um alargamento do do editor**
//! (`winit_to_editor_keycode`, que fica na shell): esse devolve `None` de propósito para as teclas
//! que o editor **não** consome, e é esse `None` que as deixa *cair* para os consumidores de baixo.
//! Este mapeia **tudo o que um artista pode querer ligar**, no **MESMO espaço `u32`**; o gate da shell
//! `the_two_normalizers_never_disagree` torna a divergência impossível de aterrar em silêncio.
//!
//! ⛔ **Não é um enum novo.** Letras e dígitos vão para o ASCII maiúsculo (`A` = `0x41`), onde já estão
//! as `KEY_KEY_*` do editor. O resto segue a tabela `NAMED` de `ph2d_input::keyboard` — os testes
//! desta crate conferem que a imagem deste mapa é EXACTAMENTE essa tabela.

use winit::keyboard::KeyCode;

/// O primeiro código da faixa de função no espaço NSEvent que o editor já usa para as setas.
const FN_BASE: u32 = 0xF704;
/// Numpad 0 (`ph2d_input`: `kp0`); Numpad 1..9 seguem-no.
const KP0: u32 = 0xF740;

/// Traduz uma tecla física do winit; `None` = tecla que nenhum jogo liga. ⛔ Super (Windows), Fn e
/// Print Screen ficam `None` de propósito: medido a 10/11, o desktop nunca as entrega ao app (o KDE
/// prende Meta ao lançador e Print ao Spectacle; o Fn resolve-se dentro do teclado).
#[must_use]
pub fn winit_to_input_keycode(code: KeyCode) -> Option<u32> {
    use KeyCode as K;
    let v = match code {
        K::KeyA => 0x41,
        K::KeyB => 0x42,
        K::KeyC => 0x43,
        K::KeyD => 0x44,
        K::KeyE => 0x45,
        K::KeyF => 0x46,
        K::KeyG => 0x47,
        K::KeyH => 0x48,
        K::KeyI => 0x49,
        K::KeyJ => 0x4A,
        K::KeyK => 0x4B,
        K::KeyL => 0x4C,
        K::KeyM => 0x4D,
        K::KeyN => 0x4E,
        K::KeyO => 0x4F,
        K::KeyP => 0x50,
        K::KeyQ => 0x51,
        K::KeyR => 0x52,
        K::KeyS => 0x53,
        K::KeyT => 0x54,
        K::KeyU => 0x55,
        K::KeyV => 0x56,
        K::KeyW => 0x57,
        K::KeyX => 0x58,
        K::KeyY => 0x59,
        K::KeyZ => 0x5A,
        K::Digit0 => 0x30,
        K::Digit1 => 0x31,
        K::Digit2 => 0x32,
        K::Digit3 => 0x33,
        K::Digit4 => 0x34,
        K::Digit5 => 0x35,
        K::Digit6 => 0x36,
        K::Digit7 => 0x37,
        K::Digit8 => 0x38,
        K::Digit9 => 0x39,
        // Os controlos, nos MESMOS valores que o editor já usa.
        K::Tab => 0x09,
        K::Enter => 0x0D,
        K::Space => 0x20,
        K::Escape => 0x1B,
        K::Backspace => 0x08,
        K::Delete => 0xF728,
        K::ArrowUp => 0xF700,
        K::ArrowDown => 0xF701,
        K::ArrowLeft => 0xF702,
        K::ArrowRight => 0xF703,
        // F1..F12, contíguas a partir da mesma faixa das setas (o `F2` do editor é `0xF705`).
        K::F1 => FN_BASE,
        K::F2 => FN_BASE + 1,
        K::F3 => FN_BASE + 2,
        K::F4 => FN_BASE + 3,
        K::F5 => FN_BASE + 4,
        K::F6 => FN_BASE + 5,
        K::F7 => FN_BASE + 6,
        K::F8 => FN_BASE + 7,
        K::F9 => FN_BASE + 8,
        K::F10 => FN_BASE + 9,
        K::F11 => FN_BASE + 10,
        K::F12 => FN_BASE + 11,
        // Os modificadores: um jogo liga-lhes acções (agachar no Ctrl, correr no Shift).
        K::ShiftLeft => 0xF710,
        K::ShiftRight => 0xF711,
        K::ControlLeft => 0xF712,
        K::ControlRight => 0xF713,
        K::AltLeft => 0xF714,
        K::AltRight => 0xF715,
        // ⛔ As cinco que a auditoria de 2026-08-24 achou INLIGÁVEIS (atalho de editor sem normalizador).
        K::BracketLeft => 0x5B,
        K::BracketRight => 0x5D,
        K::Comma => 0x2C,
        K::Period => 0x2E,
        K::Home => 0xF729,
        // Teclado de notebook (§13.1): a pontuação no ASCII do carácter US.
        K::Backquote => 0x60,
        K::Minus => 0x2D,
        K::Equal => 0x3D,
        K::Backslash => 0x5C,
        K::Semicolon => 0x3B,
        K::Quote => 0x27,
        K::Slash => 0x2F,
        // Navegação e travas, na faixa NSEvent.
        K::Insert => 0xF727,
        K::End => 0xF72B,
        K::PageUp => 0xF72C,
        K::PageDown => 0xF72D,
        K::CapsLock => 0xF716,
        K::NumLock => 0xF717,
        // O numérico: tecla FÍSICA, igual com o Num Lock aceso ou apagado (Numpad 7 ≠ 7 ≠ Home).
        K::Numpad0 => KP0,
        K::Numpad1 => KP0 + 1,
        K::Numpad2 => KP0 + 2,
        K::Numpad3 => KP0 + 3,
        K::Numpad4 => KP0 + 4,
        K::Numpad5 => KP0 + 5,
        K::Numpad6 => KP0 + 6,
        K::Numpad7 => KP0 + 7,
        K::Numpad8 => KP0 + 8,
        K::Numpad9 => KP0 + 9,
        K::NumpadDecimal => 0xF74A,
        K::NumpadDivide => 0xF74B,
        K::NumpadMultiply => 0xF74C,
        K::NumpadSubtract => 0xF74D,
        K::NumpadAdd => 0xF74E,
        // ⛔ Código PRÓPRIO: já não cai no Enter. Quem CONFIRMA aceita os dois (`Key::is_enter_like`).
        K::NumpadEnter => 0xF74F,
        _ => return None,
    };
    Some(v)
}

#[cfg(test)]
mod tests;
