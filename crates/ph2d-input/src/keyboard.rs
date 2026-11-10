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

/// ⭐ **A TABELA ÚNICA das teclas com nome** — o código normalizado, o rótulo que o artista lê, o
/// nome que um script escreve, a legenda da tecla (`cap`, a da foto do teclado do dono) e a legenda
/// curta dos teclados compactos (`cap_short`). Letras, dígitos e F1..F12 derivam do código (abaixo).
/// Códigos e nomes: `docs/InputMap/01_plano.md` §13.1 (tecla FÍSICA: o numérico liga igual com o
/// Num Lock aceso ou apagado).
const NAMED: &[(u32, &str, &str, &str, &str)] = &[
    (0x09, "Tab", "tab", "Tab", "Tab"),
    (0x0D, "Enter", "enter", "Enter", "Enter"),
    (0x20, "Space", "space", "Space", "Space"),
    (0x1B, "Esc", "escape", "Esc", "Esc"),
    (0x08, "Backspace", "backspace", "Backspace", "Bksp"),
    (0xF728, "Delete", "delete", "Delete", "Del"),
    (0xF700, "Up Arrow", "up", "Up Arrow", "Up"),
    (0xF701, "Down Arrow", "down", "Down Arrow", "Down"),
    (0xF702, "Left Arrow", "left", "Left Arrow", "Left"),
    (0xF703, "Right Arrow", "right", "Right Arrow", "Right"),
    (0xF710, "Left Shift", "left_shift", "Shift", "Shift"),
    (0xF711, "Right Shift", "right_shift", "Shift", "Shift"),
    (0xF712, "Left Ctrl", "left_ctrl", "Ctrl", "Ctrl"),
    (0xF713, "Right Ctrl", "right_ctrl", "Ctrl", "Ctrl"),
    (0xF714, "Left Alt", "left_alt", "Alt", "Alt"),
    (0xF715, "Right Alt", "right_alt", "Alt", "Alt"),
    (0xF729, "Home", "home", "Home", "Home"),
    (0x5B, "[", "[", "[", "["),
    (0x5D, "]", "]", "]", "]"),
    (0x2C, ",", ",", ",", ","),
    (0x2E, ".", ".", ".", "."),
    // Pontuação: o ASCII do próprio carácter.
    (0x60, "`", "`", "`", "`"),
    (0x2D, "-", "-", "-", "-"),
    (0x3D, "=", "=", "=", "="),
    (0x5C, "\\", "\\", "\\", "\\"),
    (0x3B, ";", ";", ";", ";"),
    (0x27, "'", "'", "'", "'"),
    (0x2F, "/", "/", "/", "/"),
    // Navegação, na faixa NSEvent das setas.
    (0xF727, "Insert", "insert", "Insert", "Ins"),
    (0xF72B, "End", "end", "End", "End"),
    (0xF72C, "Page Up", "pageup", "PgUp", "PgUp"),
    (0xF72D, "Page Down", "pagedown", "PgDn", "PgDn"),
    // Travas.
    (0xF716, "Caps Lock", "capslock", "CapsLk", "Caps"),
    (0xF717, "Num Lock", "numlock", "Num Lock", "Num"),
    // O numérico: Numpad 7 ≠ 7 ≠ Home, Numpad Enter ≠ Enter.
    (KP0, "Numpad 0", "kp0", "0", "0"),
    (KP0 + 1, "Numpad 1", "kp1", "1", "1"),
    (KP0 + 2, "Numpad 2", "kp2", "2", "2"),
    (KP0 + 3, "Numpad 3", "kp3", "3", "3"),
    (KP0 + 4, "Numpad 4", "kp4", "4", "4"),
    (KP0 + 5, "Numpad 5", "kp5", "5", "5"),
    (KP0 + 6, "Numpad 6", "kp6", "6", "6"),
    (KP0 + 7, "Numpad 7", "kp7", "7", "7"),
    (KP0 + 8, "Numpad 8", "kp8", "8", "8"),
    (KP0 + 9, "Numpad 9", "kp9", "9", "9"),
    (0xF74A, "Numpad .", "kp.", ".", "."),
    (0xF74B, "Numpad /", "kp/", "/", "/"),
    (0xF74C, "Numpad *", "kp*", "*", "*"),
    (0xF74D, "Numpad -", "kp-", "-", "-"),
    (0xF74E, "Numpad +", "kp+", "+", "+"),
    (NUMPAD_ENTER, "Numpad Enter", "kpenter", "Enter", "Enter"),
];

/// Numpad 0; Numpad 1..9 seguem-no.
const KP0: u32 = 0xF740;
/// O Enter do numérico, com código PRÓPRIO (já não cai no `0x0D`).
const NUMPAD_ENTER: u32 = 0xF74F;

/// ⛔ **As teclas do SISTEMA** — desenhadas no teclado, nunca ligadas: `(código, rótulo, cap,
/// cap_short, chave i18n do porquê)`. Sem nome de script: um script nunca as vê. Medido a 10/11
/// (plano §13.1): o Fn resolve-se dentro do teclado, o KDE prende o Print ao Spectacle e o Meta ao
/// lançador.
const SYSTEM: &[(u32, &str, &str, &str, &str)] = &[
    (
        0xF718,
        "Windows key",
        "Win",
        "Win",
        "input_map_keyboard.system.windows",
    ),
    (
        0xF719,
        "Windows key",
        "Win",
        "Win",
        "input_map_keyboard.system.windows",
    ),
    (0xF71A, "Fn", "Fn", "Fn", "input_map_keyboard.system.fn"),
    (
        0xF72E,
        "Print Screen",
        "PrtSc",
        "PrtSc",
        "input_map_keyboard.system.print_screen",
    ),
];

const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &str = "0123456789";
const F_CAPS: [&str; 12] = [
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
];

fn named_row(
    c: u32,
) -> Option<&'static (u32, &'static str, &'static str, &'static str, &'static str)> {
    NAMED.iter().find(|r| r.0 == c)
}

fn system_row(
    c: u32,
) -> Option<&'static (u32, &'static str, &'static str, &'static str, &'static str)> {
    SYSTEM.iter().find(|r| r.0 == c)
}

/// F1..F12, contíguas a partir daqui.
pub(crate) const F1: u32 = 0xF704;

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
            .chain(NAMED.iter().map(|r| r.0))
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
            c => named_row(c)
                .or_else(|| system_row(c))
                .map_or_else(|| format!("Key {c:#X}"), |r| r.1.to_owned()),
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
            c => named_row(c).map(|r| r.2.to_owned()),
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

    /// O Enter da fileira principal.
    pub const ENTER: Key = Key(0x0D);
    /// O Enter do numérico — tecla própria, ligável à parte.
    pub const NUMPAD_ENTER: Key = Key(NUMPAD_ENTER);

    /// **Confirma como um Enter?** Os dois Enter (fileira e numérico): quem usa o Enter para
    /// CONFIRMAR (um campo, a tela do jogador) aceita os dois; uma LIGAÇÃO de jogo distingue-os.
    #[must_use]
    pub fn is_enter_like(self) -> bool {
        self == Self::ENTER || self == Self::NUMPAD_ENTER
    }

    /// As teclas do sistema (Windows, Fn, Print Screen): desenhadas, nunca ligadas, sem nome de script.
    pub fn system_keys() -> impl Iterator<Item = Key> {
        SYSTEM.iter().map(|r| Key(r.0))
    }

    /// `Some(chave i18n do porquê)` = tecla do SISTEMA, que nenhum jogo recebe
    /// (`input_map_keyboard.system.{fn,windows,print_screen}`); `None` = uma tecla do jogo.
    #[must_use]
    pub fn system_owned(self) -> Option<&'static str> {
        system_row(self.0).map(|r| r.4)
    }

    /// **A legenda impressa na tecla** (a foto do teclado do dono: `Backspace`, `CapsLk`, `PrtSc`,
    /// `PgUp`, o `7` do numérico…). `""` = código sem tecla conhecida (o pintor usa o [`Key::label`]).
    #[must_use]
    pub fn cap(self) -> &'static str {
        self.caps().0
    }

    /// A legenda curta dos teclados compactos (`Bksp`, `Caps`, `Num`, `Ins`, `Del`); igual ao
    /// [`Key::cap`] quando não há uma mais curta. O pintor escolhe a que cabe, medida, por tecla.
    #[must_use]
    pub fn cap_short(self) -> &'static str {
        self.caps().1
    }

    /// A legenda em DUAS linhas, como muitos notebooks a imprimem (`Prt` / `Sc`); `None` = uma linha só.
    #[must_use]
    pub fn cap_lines(self) -> Option<(&'static str, &'static str)> {
        match self.0 {
            0xF72E => Some(("Prt", "Sc")),
            0xF717 => Some(("Num", "Lock")),
            _ => None,
        }
    }

    fn caps(self) -> (&'static str, &'static str) {
        let at = |s: &'static str, i: u32| {
            let i = i as usize;
            let one = &s[i..=i];
            (one, one)
        };
        match self.0 {
            c @ 0x41..=0x5A => at(LETTERS, c - 0x41),
            c @ 0x30..=0x39 => at(DIGITS, c - 0x30),
            c if (F1..F1 + 12).contains(&c) => {
                let f = F_CAPS[(c - F1) as usize];
                (f, f)
            }
            c => named_row(c)
                .map(|r| (r.3, r.4))
                .or_else(|| system_row(c).map(|r| (r.2, r.3)))
                .unwrap_or(("", "")),
        }
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
