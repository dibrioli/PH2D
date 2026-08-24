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
