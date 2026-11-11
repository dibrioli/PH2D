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

/// ⭐ **AltGr não é o Alt do editor** (onda 4): das teclas FÍSICAS em baixo, as que fazem acorde — Ctrl dos dois
/// lados, o Alt esquerdo, e o Alt direito só com a bandeira Alt da plataforma (`alt`, o `ModifiersState::ALT` que
/// todo atalho de Alt do editor lê). Medido no winit 0.30.13: o estado por lado (`lalt_state`) só existe no macOS;
/// no Windows `get_agnostic_mods` apaga ALT e CONTROL com o AltGr em baixo; no xkb (`br(abnt2)`, `us(intl)`) o
/// `<RALT>` é `ISO_Level3_Shift` no `Mod5` e o `Mod1` (Alt) é só o `<LALT>`; no `us` simples o `Alt_R` é `Mod1`.
#[must_use]
pub fn chord_keys(held: impl IntoIterator<Item = Key>, alt: bool) -> Vec<Key> {
    let right_alt = named("right_alt");
    held.into_iter()
        .filter(|k| is_modifier(*k) && (alt || !right_alt.contains(k)))
        .collect()
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

    /// A tabela do AltGr: (teclas físicas em baixo, bandeira Alt da plataforma, tecla, recusada?).
    #[test]
    fn altgr_nao_faz_acorde_e_o_alt_esquerdo_faz() {
        let (f, k) = (Key(u32::from('F')), Key(u32::from('K')));
        let [la, ra, lc] = ["left_alt", "right_alt", "left_ctrl"].map(|n| named(n)[0]);
        let casos = [
            (
                vec![la],
                true,
                f,
                true,
                "Alt esquerdo + F (Alt+F do editor de scripts)",
            ),
            (
                vec![la],
                true,
                k,
                true,
                "Alt esquerdo + K: o acorde segue recusado",
            ),
            (
                vec![ra],
                false,
                f,
                false,
                "AltGr + F: a plataforma não ergue Alt, liga o F",
            ),
            (
                vec![ra],
                true,
                f,
                true,
                "Alt direito que É Alt (us): recusado",
            ),
            (vec![la, ra], true, f, true, "os dois Alt"),
            (
                vec![lc, ra],
                false,
                f,
                true,
                "Ctrl + AltGr: o Ctrl faz acorde",
            ),
            (vec![], false, f, false, "nada em baixo"),
        ];
        for (held, alt, key, recusada, caso) in casos {
            assert_eq!(never_binds(key, chord_keys(held, alt)), recusada, "{caso}");
        }
        // Controlo: o Alt direito físico sem o filtro é acorde — a distinção é o filtro.
        assert!(never_binds(f, [ra]));
        assert!(
            never_binds(ra, chord_keys([], false)),
            "AltGr sozinho não liga"
        );
    }
}
