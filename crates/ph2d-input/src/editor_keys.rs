//! **As teclas físicas que o Input Map NUNCA liga** — a lei num sítio só, aqui na folha para que a escuta partilhada
//! do editor (`ph2d-editor-core`) e as vistas da aba (`ph2d-input-map-editor::raw_keys`) a leiam sem ciclo de
//! dependência (plano `docs/InputMap/01_plano.md` §3): Ctrl e Alt sozinhos, qualquer tecla com um deles em baixo e o
//! F5 (Play/Stop) são do editor e passam (o «em baixo» vem do store por [`held_from_flags`]: Alt+Q é tão do editor
//! quanto Ctrl+Q); as do SISTEMA nenhum jogo recebe. O Shift NÃO entra: sozinho é uma ligação
//! legítima (correr) e com uma letra é a maiúscula.

use crate::Key;

fn named(name: &str) -> Vec<Key> {
    Key::named_in_script(name).unwrap_or_default()
}

/// Ctrl ou Alt, de qualquer lado.
#[must_use]
pub fn is_modifier(key: Key) -> bool {
    ["ctrl", "alt"].iter().any(|n| named(n).contains(&key))
}

/// O F5: o transporte (Play/Stop) do editor — no Play ele nunca chega ao jogo.
#[must_use]
pub fn is_transport(key: Key) -> bool {
    named("f5").contains(&key)
}

/// `true` = o premir é um acorde do editor: a tecla é um modificador, ou há um em `held` (as teclas em baixo; pode
/// incluir a própria `key`).
#[must_use]
pub fn is_chord_or_modifier(key: Key, held: impl IntoIterator<Item = Key>) -> bool {
    is_modifier(key) || held.into_iter().any(is_modifier)
}

/// As teclas em baixo a partir das BANDEIRAS de modificador (o que a shell guarda no store em cada
/// `ModifiersChanged`): Ctrl e Alt dos dois lados, para [`never_binds`] reconhecer o acorde.
#[must_use]
pub fn held_from_flags(ctrl: bool, alt: bool) -> Vec<Key> {
    let mut held = Vec::new();
    if ctrl {
        held.extend(named("ctrl"));
    }
    if alt {
        held.extend(named("alt"));
    }
    held
}

/// `true` = nunca vira ligação: o acorde ([`is_chord_or_modifier`]), o F5, ou uma tecla do SISTEMA
/// (Windows, Fn, Print Screen: [`Key::system_owned`], plano §13.1).
#[must_use]
pub fn never_binds(key: Key, held: impl IntoIterator<Item = Key>) -> bool {
    key.system_owned().is_some() || is_transport(key) || is_chord_or_modifier(key, held)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alt_em_baixo_e_acorde_como_o_ctrl() {
        let q = Key(u32::from('Q'));
        assert!(!never_binds(q, held_from_flags(false, false)));
        assert!(never_binds(q, held_from_flags(true, false)));
        assert!(never_binds(q, held_from_flags(false, true)));
        for n in ["left_alt", "right_alt", "left_ctrl", "right_ctrl"] {
            assert!(
                never_binds(named(n)[0], held_from_flags(false, false)),
                "{n}"
            );
        }
    }
}
