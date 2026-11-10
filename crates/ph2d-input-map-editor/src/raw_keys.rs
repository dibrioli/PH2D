//! **As teclas físicas que a aba NUNCA liga** — um filtro puro, o mesmo para as três vistas (plano
//! `docs/InputMap/01_plano.md` §3): Ctrl e Alt sozinhos, qualquer tecla com um deles em baixo (o Ctrl+Z, o Ctrl+S,
//! o Alt+…) e o F5 (Play/Stop) são do editor e passam; ligá-los seria roubar o atalho. O Super não chega como
//! `Key` (só como modificador do host), por isso não há o que filtrar aqui. O Shift NÃO entra: sozinho é uma
//! ligação legítima (correr) e com uma letra é a maiúscula.

//!
//! A lei mora em [`ph2d_input::editor_keys`], a folha que a escuta partilhada do editor também lê; aqui só se
//! reexporta, para as vistas continuarem a chamar `raw_keys::never_binds`.

use ph2d_input::Key;
pub use ph2d_input::editor_keys::{is_chord_or_modifier, is_modifier, is_transport, never_binds};

/// Os modificadores em baixo, para quem não tem um `KeyboardState` (a vista B): só Ctrl/Alt, ordenados.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeldModifiers(Vec<Key>);

impl HeldModifiers {
    /// Regista o premir/soltar de `key` (as que não são modificador não ficam).
    pub fn update(&mut self, key: Key, pressed: bool) {
        if !is_modifier(key) {
            return;
        }
        match (self.0.binary_search(&key), pressed) {
            (Err(at), true) => self.0.insert(at, key),
            (Ok(at), false) => {
                self.0.remove(at);
            }
            _ => {}
        }
    }

    /// Os modificadores em baixo.
    pub fn iter(&self) -> impl Iterator<Item = Key> + '_ {
        self.0.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔ As do sistema nunca ligam, nem sozinhas; o numérico e a pontuação ligam (são do jogo).
    #[test]
    fn system_keys_never_bind_and_notebook_keys_do() {
        let sys: Vec<Key> = Key::system_keys().collect();
        assert_eq!(sys.len(), 4);
        for k in sys {
            assert!(never_binds(k, []), "{}", k.label());
        }
        for n in ["kp7", "kpenter", "numlock", "capslock", "insert", "/", "`"] {
            let k = Key::named_in_script(n).expect(n)[0];
            assert!(!never_binds(k, []), "{n}");
        }
    }
}
