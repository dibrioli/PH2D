//! **AS TECLAS DO JOGADOR** — o que o jogador trocou na tela de controles (plano InputMap §2).
//!
//! A chave é o NOME da acção (o contrato de quem lê), e o ficheiro é texto:
//! `jump = key:up, key:z, pad:south, axis:left_stick_x-`.

use std::collections::BTreeMap;

use crate::action::Binding;
use crate::gamepad::{GamepadAxis, GamepadButton};
use crate::keyboard::Key;
use crate::map::InputMap;

/// A primeira linha do ficheiro do jogador.
pub const PLAYER_KEYS_HEADER: &str = "# PH2D player keys";

/// As ligações que o jogador pôs por cima das do jogo, por nome de acção.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerKeys {
    by_action: BTreeMap<String, Vec<Binding>>,
}

impl PlayerKeys {
    /// Nada trocado.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// O jogador liga `action` a `bindings` (zero é válido: ele tirou todas).
    pub fn set(&mut self, action: impl Into<String>, bindings: Vec<Binding>) {
        self.by_action.insert(action.into(), bindings);
    }

    /// `action` volta às teclas do jogo.
    pub fn clear(&mut self, action: &str) {
        self.by_action.remove(action);
    }

    /// Todas voltam às teclas do jogo.
    pub fn clear_all(&mut self) {
        self.by_action.clear();
    }

    /// As ligações do jogador para `action`, se ele a trocou.
    #[must_use]
    pub fn get(&self, action: &str) -> Option<&[Binding]> {
        self.by_action.get(action).map(Vec::as_slice)
    }

    /// Nada trocado.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_action.is_empty()
    }

    /// O mapa do jogo com as trocas do jogador; nomes que o mapa não tem são ignorados.
    #[must_use]
    pub fn apply_over(&self, map: &InputMap) -> InputMap {
        let mut out = map.clone();
        let ids: Vec<_> = map.actions().iter().map(|a| (a.id, a.name.clone())).collect();
        for (id, name) in ids {
            if let (Some(b), Some(a)) = (self.by_action.get(&name), out.get_mut(id)) {
                a.bindings.clone_from(b);
            }
        }
        out
    }

    /// Quantas acções do mapa têm ligações diferentes das do jogo.
    #[must_use]
    pub fn changed(&self, map: &InputMap) -> usize {
        map.actions()
            .iter()
            .filter(|a| self.by_action.get(&a.name).is_some_and(|b| *b != a.bindings))
            .count()
    }

    /// O ficheiro do jogador.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut s = String::from(PLAYER_KEYS_HEADER);
        s.push('\n');
        for (name, bindings) in &self.by_action {
            let list: Vec<String> = bindings.iter().map(|b| binding_text(*b)).collect();
            s.push_str(name);
            s.push_str(" =");
            if !list.is_empty() {
                s.push(' ');
                s.push_str(&list.join(", "));
            }
            s.push('\n');
        }
        s
    }

    /// Lê o ficheiro do jogador; linha inválida salta-se.
    #[must_use]
    pub fn from_text(text: &str) -> Self {
        let mut keys = Self::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((name, rhs)) = line.split_once('=') else {
                continue;
            };
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            if let Some(bindings) = parse_bindings(rhs) {
                keys.set(name, bindings);
            }
        }
        keys
    }
}

/// `key:<nome>` · `pad:<botão>` · `axis:<eixo>+|-`. Uma tecla sem nome escreve-se `key:0x<hex>`.
#[must_use]
pub fn binding_text(b: Binding) -> String {
    match b {
        Binding::Key(k) => match k.script_name() {
            Some(n) => format!("key:{n}"),
            None => format!("key:{:#x}", k.0),
        },
        Binding::PadButton(p) => format!("pad:{}", p.as_lua_key()),
        Binding::PadAxis { axis, positive } => {
            format!("axis:{}{}", axis.as_lua_key(), if positive { '+' } else { '-' })
        }
    }
}

/// A lista depois do `=`; `None` se algum item não se lê.
fn parse_bindings(rhs: &str) -> Option<Vec<Binding>> {
    let mut out = Vec::new();
    let mut rest = rhs.trim_start();
    while !rest.is_empty() {
        let (kind, after) = rest.split_once(':')?;
        let after_name;
        match kind.trim() {
            "key" => {
                let (name, tail) = take_key_name(after)?;
                out.extend(parse_key(name)?.into_iter().map(Binding::Key));
                after_name = tail;
            }
            "pad" => {
                let (name, tail) = take_word(after);
                let p = GamepadButton::ALL.into_iter().find(|p| p.as_lua_key() == name)?;
                out.push(Binding::PadButton(p));
                after_name = tail;
            }
            "axis" => {
                let (name, tail) = take_word(after);
                let axis = GamepadAxis::ALL.into_iter().find(|a| a.as_lua_key() == name)?;
                let positive = match tail.chars().next()? {
                    '+' => true,
                    '-' => false,
                    _ => return None,
                };
                out.push(Binding::PadAxis { axis, positive });
                after_name = &tail[1..];
            }
            _ => return None,
        }
        let tail = after_name.trim_start();
        rest = match tail.strip_prefix(',') {
            Some(t) => t.trim_start(),
            None if tail.is_empty() => tail,
            None => return None,
        };
    }
    Some(out)
}

/// `[a-z0-9_]+` do início.
fn take_word(s: &str) -> (&str, &str) {
    let end = s
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    s.split_at(end)
}

/// O nome de uma tecla: uma palavra, ou um sinal só (`[`, `]`, `,`, `.`).
fn take_key_name(s: &str) -> Option<(&str, &str)> {
    let (w, tail) = take_word(s);
    if !w.is_empty() {
        return Some((w, tail));
    }
    let c = s.chars().next()?;
    Some(s.split_at(c.len_utf8()))
}

fn parse_key(name: &str) -> Option<Vec<Key>> {
    if let Some(hex) = name.strip_prefix("0x") {
        return u32::from_str_radix(hex, 16).ok().map(|c| vec![Key(c)]);
    }
    Key::named_in_script(name)
}

#[cfg(test)]
#[path = "player_keys_tests.rs"]
mod tests;
