//! Gates do BULLET — *que linha deste menu está acesa?*
//!
//! ⚠️ **É o primeiro gate sobre o `id_is_currently_selected`, e a razão de nascer com a submenu de
//! Motion é que aqui o bullet é o ÚNICO readout do estado.** Um filtro de imagem errado vê-se na
//! arte; um carácter de UI errado vê-se... na velocidade com que um botão acende, que é
//! precisamente o que o artista não consegue medir a olho. Se a linha acesa mentir, ele não tem
//! como saber em que modo está.
//!
//! O gate chama a função directamente em vez de pintar e ler a cena: o harness de chrome deste repo
//! lê retângulos de hit, nunca pixels, e uma segunda máquina de leitura de cena para um ponto seria
//! mais superfície do que o facto que ela prova.

use super::context_menu_overlay::id_is_currently_selected;
use super::ids;
use crate::interaction::WidgetStore;
use crate::motion::{UiCharacter, UiMotion};
use crate::project::ProjectSettings;
use ph2d_tokens::Theme;

fn lit(id: ph2d_a11y::NodeId, motion: &UiMotion) -> bool {
    let store = WidgetStore::default();
    let project = ProjectSettings::default();
    id_is_currently_selected(id, Theme::Forge, &store, &project, motion)
}

/// O carácter é um RÁDIO: exactamente UMA das duas linhas acende, e ela segue o estado.
///
/// **Mutação que deve sangrar:** trocar os dois braços do `match motion.character()`.
#[test]
fn exactly_one_character_row_is_lit_and_it_is_the_live_one() {
    let mut motion = UiMotion::default();
    for character in [UiCharacter::Discrete, UiCharacter::Expressive] {
        motion.set_character(character);
        let expressive = lit(ids::CTX_MENU_MOTION_EXPRESSIVE, &motion);
        let discrete = lit(ids::CTX_MENU_MOTION_DISCRETE, &motion);
        assert_ne!(
            expressive, discrete,
            "um rádio acende uma linha, nunca duas nem zero"
        );
        assert_eq!(
            expressive,
            character == UiCharacter::Expressive,
            "a linha acesa tem de ser a do carácter VIVO ({character:?})"
        );
    }
}

/// O reduced motion é um TOGGLE: acende quando ligado e **apaga quando desligado**.
///
/// **Mutação que deve sangrar:** `if id == CTX_MENU_MOTION_REDUCED { return true }` sem a segunda
/// metade da condição — a row passaria a dizer *ligado* para sempre, sobre um interruptor que o
/// artista consegue desligar.
#[test]
fn the_reduced_row_is_lit_only_while_it_is_on() {
    let mut motion = UiMotion::default();
    assert!(!lit(ids::CTX_MENU_MOTION_REDUCED, &motion));
    motion.set_reduced_motion(true);
    assert!(lit(ids::CTX_MENU_MOTION_REDUCED, &motion));
    motion.set_reduced_motion(false);
    assert!(!lit(ids::CTX_MENU_MOTION_REDUCED, &motion));
}

/// **Os dois eixos acendem ao mesmo tempo** — a prova, no readout, de que a submenu não é um
/// selector de três posições disfarçado.
#[test]
fn expressive_and_reduced_light_together() {
    let mut motion = UiMotion::default();
    motion.set_character(UiCharacter::Expressive);
    motion.set_reduced_motion(true);
    assert!(lit(ids::CTX_MENU_MOTION_EXPRESSIVE, &motion));
    assert!(lit(ids::CTX_MENU_MOTION_REDUCED, &motion));
    assert!(!lit(ids::CTX_MENU_MOTION_DISCRETE, &motion));
}
