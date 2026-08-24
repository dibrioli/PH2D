//! **O TECLADO** — o dispositivo que faltava a esta crate.
//!
//! Ela nasceu na M8 com **gamepad e caneta** (PR #13) e nunca teve teclado, porque o consumidor
//! dela era o `ph2d.input` do Luau e o adaptador do `gilrs`. O Input Map
//! (`docs/Vector Module/30_plano_input_map.md`) precisa dele: a esmagadora maioria das ligações que
//! um artista autora é uma tecla.
//!
//! ⚠️ **`Key` é um `u32`, e NÃO um enum novo.** A shell já normaliza o teclado nativo (winit,
//! AppKit, win32) para este espaço antes de despachar — é o que o
//! `ph2d_editor_core::interaction::dispatch::keymap` consome, e é por isso que **nem essa crate nem
//! esta** dependem de `winit`. *O vocabulário de teclas do app já existe; um segundo seriam duas
//! respostas para a mesma pergunta.*
//!
//! A forma é a do irmão [`crate::gamepad::GamepadState`], de propósito: `begin_frame` fotografa o
//! quadro anterior, e daí saem `pressed` (a borda), `held` e `released`.

use serde::{Deserialize, Serialize};

/// Uma tecla, no espaço de keycode normalizado do app.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Key(pub u32);

impl Key {
    /// **O NOME QUE O ARTISTA LÊ** — `"Left Arrow"`, `"A"`, `"Space"`.
    ///
    /// ⛔⛔ **Ele existe porque a primeira versão desta janela mostrava `Key 0xF702`**, e o Enio
    /// respondeu o óbvio: *"Quem vai usar são artistas e não IA"*. Um código hexadecimal é a
    /// representação interna a vazar para a cara do produto — e o artista não tem como saber que
    /// `0xF702` é a seta para a esquerda, nem tem por que saber.
    ///
    /// ⚠️ **Devolve `String` e não `&'static str`** por causa da última linha: uma tecla que este
    /// mapa não conhece tem de dizer **alguma coisa** em vez de sumir da lista, e o que ela pode
    /// dizer honestamente é o número. *Uma ligação invisível é pior que uma ligação feia.*
    #[must_use]
    pub fn label(self) -> String {
        let named = match self.0 {
            0x09 => "Tab",
            0x0D => "Enter",
            0x20 => "Space",
            0x1B => "Esc",
            0x08 => "Backspace",
            0xF728 => "Delete",
            0xF700 => "Up Arrow",
            0xF701 => "Down Arrow",
            0xF702 => "Left Arrow",
            0xF703 => "Right Arrow",
            0xF710 => "Left Shift",
            0xF711 => "Right Shift",
            0xF712 => "Left Ctrl",
            0xF713 => "Right Ctrl",
            0xF714 => "Left Alt",
            0xF715 => "Right Alt",
            // Letras e dígitos vivem no ASCII, então o próprio código É o nome.
            c @ (0x30..=0x39 | 0x41..=0x5A) => {
                return char::from_u32(c).map_or_else(|| format!("Key {c:#X}"), String::from);
            }
            // F1..F12, contíguas a partir de `0xF704`.
            c @ 0xF704..=0xF70F => return format!("F{}", c - 0xF704 + 1),
            c => return format!("Key {c:#X}"),
        };
        named.to_string()
    }
}

/// Retrato do teclado: o que está em baixo agora, e o que estava no quadro anterior.
///
/// ⚠️ **Listas ordenadas e sem repetidos**, nunca um `HashMap`. Não é arrumação: a resolução em
/// acções alimenta a fita determinística da física, e a ordem de iteração de um `HashMap` não é uma
/// promessa — é a mesma espinha de determinismo que faz o módulo de física proibir `HashMap` por
/// lint estrutural.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyboardState {
    down: Vec<Key>,
    was_down: Vec<Key>,
}

impl KeyboardState {
    /// Teclado em repouso.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Fotografa o quadro anterior — o que paga [`KeyboardState::pressed`] e
    /// [`KeyboardState::released`].
    pub fn begin_frame(&mut self) {
        self.was_down.clear();
        self.was_down.extend_from_slice(&self.down);
    }

    /// A tecla desceu.
    pub fn handle_key_down(&mut self, k: Key) {
        if let Err(at) = self.down.binary_search(&k) {
            self.down.insert(at, k);
        }
    }

    /// A tecla subiu.
    pub fn handle_key_up(&mut self, k: Key) {
        if let Ok(at) = self.down.binary_search(&k) {
            self.down.remove(at);
        }
    }

    /// **Larga tudo** — o que a shell chama quando a janela perde o foco.
    ///
    /// ⚠️ É a metade que **esta crate** pode resolver do modo de falha que o `player_input.rs` da
    /// shell nomeia: *um `Up` perdido (troca de janela, foco roubado) deixaria o personagem a andar
    /// sozinho para sempre*. Sem esta porta, a única saída seria a shell adivinhar tecla a tecla.
    pub fn release_all(&mut self) {
        self.down.clear();
    }

    /// A borda: passou de solta a premida **neste** quadro.
    #[must_use]
    pub fn pressed(&self, k: Key) -> bool {
        self.down.binary_search(&k).is_ok() && self.was_down.binary_search(&k).is_err()
    }

    /// Está em baixo agora.
    #[must_use]
    pub fn held(&self, k: Key) -> bool {
        self.down.binary_search(&k).is_ok()
    }

    /// A borda contrária.
    #[must_use]
    pub fn released(&self, k: Key) -> bool {
        self.down.binary_search(&k).is_err() && self.was_down.binary_search(&k).is_ok()
    }

    /// As teclas em baixo, em ordem estável.
    pub fn iter_held(&self) -> impl Iterator<Item = Key> + '_ {
        self.down.iter().copied()
    }
}

#[cfg(test)]
#[path = "keyboard_tests.rs"]
mod tests;
