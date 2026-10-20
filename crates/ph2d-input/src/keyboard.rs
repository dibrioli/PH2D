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

/// ⭐ **A TABELA ÚNICA das teclas com nome** — o código normalizado, o rótulo que o artista lê e o
/// nome que um script escreve. Letras, dígitos e F1..F12 derivam do código (abaixo).
const NAMED: &[(u32, &str, &str)] = &[
    (0x09, "Tab", "tab"),
    (0x0D, "Enter", "enter"),
    (0x20, "Space", "space"),
    (0x1B, "Esc", "escape"),
    (0x08, "Backspace", "backspace"),
    (0xF728, "Delete", "delete"),
    (0xF700, "Up Arrow", "up"),
    (0xF701, "Down Arrow", "down"),
    (0xF702, "Left Arrow", "left"),
    (0xF703, "Right Arrow", "right"),
    (0xF710, "Left Shift", "left_shift"),
    (0xF711, "Right Shift", "right_shift"),
    (0xF712, "Left Ctrl", "left_ctrl"),
    (0xF713, "Right Ctrl", "right_ctrl"),
    (0xF714, "Left Alt", "left_alt"),
    (0xF715, "Right Alt", "right_alt"),
    (0xF729, "Home", "home"),
    (0x5B, "[", "["),
    (0x5D, "]", "]"),
    (0x2C, ",", ","),
    (0x2E, ".", "."),
];

/// F1..F12, contíguas a partir daqui.
const F1: u32 = 0xF704;

/// Os nomes de script que valem os DOIS lados (o `KEY_SHIFT` do Godot).
pub const SCRIPT_ALIASES: &[(&str, [Key; 2])] = &[
    ("shift", [Key(0xF710), Key(0xF711)]),
    ("ctrl", [Key(0xF712), Key(0xF713)]),
    ("alt", [Key(0xF714), Key(0xF715)]),
];

impl Key {
    /// Toda tecla com nome, pela ordem da tabela: letras, dígitos, F1..F12 e a [`NAMED`].
    pub fn all_named() -> impl Iterator<Item = Key> {
        (0x41..=0x5A)
            .chain(0x30..=0x39)
            .chain(F1..F1 + 12)
            .chain(NAMED.iter().map(|(c, _, _)| *c))
            .map(Key)
    }

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
        match self.0 {
            // Letras e dígitos vivem no ASCII, então o próprio código É o nome.
            c @ (0x30..=0x39 | 0x41..=0x5A) => {
                char::from_u32(c).map_or_else(|| format!("Key {c:#X}"), String::from)
            }
            c if (F1..F1 + 12).contains(&c) => format!("F{}", c - F1 + 1),
            c => NAMED
                .iter()
                .find(|(k, _, _)| *k == c)
                .map_or_else(|| format!("Key {c:#X}"), |(_, l, _)| (*l).to_owned()),
        }
    }

    /// **O nome que um script escreve** (`ph2d.key_down("left")`) — minúsculas; `None` para um
    /// código sem nome.
    #[must_use]
    pub fn script_name(self) -> Option<String> {
        match self.0 {
            c @ (0x30..=0x39 | 0x41..=0x5A) => {
                char::from_u32(c).map(|ch| ch.to_ascii_lowercase().to_string())
            }
            c if (F1..F1 + 12).contains(&c) => Some(format!("f{}", c - F1 + 1)),
            c => NAMED
                .iter()
                .find(|(k, _, _)| *k == c)
                .map(|(_, _, n)| (*n).to_owned()),
        }
    }

    /// As teclas que um nome de script designa: uma, ou as duas de um [`SCRIPT_ALIASES`]. `None` =
    /// nome desconhecido (o script tem de o dizer, nunca ler «solta»).
    #[must_use]
    pub fn named_in_script(name: &str) -> Option<Vec<Key>> {
        if let Some((_, both)) = SCRIPT_ALIASES.iter().find(|(n, _)| *n == name) {
            return Some(both.to_vec());
        }
        Self::all_named()
            .find(|k| k.script_name().as_deref() == Some(name))
            .map(|k| vec![k])
    }

    /// Todo nome de script, sem repetidos: as teclas e depois os [`SCRIPT_ALIASES`].
    #[must_use]
    pub fn script_names() -> Vec<String> {
        Self::all_named()
            .filter_map(Key::script_name)
            .chain(SCRIPT_ALIASES.iter().map(|(n, _)| (*n).to_owned()))
            .collect()
    }
}

/// Retrato do teclado: o que está em baixo agora e as bordas que aconteceram desde o `begin_frame`.
///
/// ⚠️ **Listas ordenadas e sem repetidos**, nunca um `HashMap`. Não é arrumação: a resolução em
/// acções alimenta a fita determinística da física, e a ordem de iteração de um `HashMap` não é uma
/// promessa — é a mesma espinha de determinismo que faz o módulo de física proibir `HashMap` por
/// lint estrutural.
///
/// ⚠️ **As bordas são EVENTOS, não a diferença de dois retratos**: um toque que carrega e larga
/// dentro do mesmo quadro nunca aparece no `down`, e a diferença perdia-o (o Godot dá-lhe
/// `just_pressed` e `just_released` — `docs/CodeEditor/05_api_que_funciona.md` §1).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyboardState {
    down: Vec<Key>,
    went_down: Vec<Key>,
    went_up: Vec<Key>,
}

fn mark(list: &mut Vec<Key>, k: Key) {
    if let Err(at) = list.binary_search(&k) {
        list.insert(at, k);
    }
}

impl KeyboardState {
    /// Teclado em repouso.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Abre um quadro: esquece as bordas do anterior.
    pub fn begin_frame(&mut self) {
        self.went_down.clear();
        self.went_up.clear();
    }

    /// A tecla desceu. A repetição do SO (já em baixo) não é borda.
    pub fn handle_key_down(&mut self, k: Key) {
        if let Err(at) = self.down.binary_search(&k) {
            self.down.insert(at, k);
            mark(&mut self.went_down, k);
        }
    }

    /// A tecla subiu.
    pub fn handle_key_up(&mut self, k: Key) {
        if let Ok(at) = self.down.binary_search(&k) {
            self.down.remove(at);
            mark(&mut self.went_up, k);
        }
    }

    /// **Larga tudo** — o que a shell chama quando a janela perde o foco. Cada tecla largada é
    /// uma borda de subida.
    ///
    /// ⚠️ É a metade que **esta crate** pode resolver do modo de falha que o `player_input.rs` da
    /// shell nomeia: *um `Up` perdido (troca de janela, foco roubado) deixaria o personagem a andar
    /// sozinho para sempre*. Sem esta porta, a única saída seria a shell adivinhar tecla a tecla.
    pub fn release_all(&mut self) {
        for k in std::mem::take(&mut self.down) {
            mark(&mut self.went_up, k);
        }
    }

    /// A borda: desceu **neste** quadro (também num toque que já subiu).
    #[must_use]
    pub fn pressed(&self, k: Key) -> bool {
        self.went_down.binary_search(&k).is_ok()
    }

    /// Está em baixo agora.
    #[must_use]
    pub fn held(&self, k: Key) -> bool {
        self.down.binary_search(&k).is_ok()
    }

    /// A borda contrária.
    #[must_use]
    pub fn released(&self, k: Key) -> bool {
        self.went_up.binary_search(&k).is_ok()
    }

    /// As teclas em baixo, em ordem estável.
    pub fn iter_held(&self) -> impl Iterator<Item = Key> + '_ {
        self.down.iter().copied()
    }

    /// As que desceram neste quadro, em ordem estável.
    pub fn iter_pressed(&self) -> impl Iterator<Item = Key> + '_ {
        self.went_down.iter().copied()
    }

    /// As que subiram neste quadro, em ordem estável.
    pub fn iter_released(&self) -> impl Iterator<Item = Key> + '_ {
        self.went_up.iter().copied()
    }
}

#[cfg(test)]
#[path = "keyboard_tests.rs"]
mod tests;
