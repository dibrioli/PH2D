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

use super::menu_row_mark::{contributed_row_is_current, id_is_currently_selected};
use crate::ids;
use crate::interaction::{InteractiveState, WidgetStore};
use crate::motion::{UiCharacter, UiMotion};
use crate::project::ProjectSettings;
use crate::widget::{ButtonState, ToolRailEntry};
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

/// ⭐⭐⭐ **A linha que um MÓDULO contribui acende pelo estado que ELE publica** (report do dono,
/// 2026-09-13: *«coloque a marca de seleção no render selecionado»*).
///
/// ⛔⛔ **A metade que se esquece é a CERCA, e ela é o 2.º `assert`:** o `dispatch::pointer_down`
/// escreve `Pressed` num botão enquanto o dedo está em baixo, então um `Pressed` **sozinho** não
/// significa *«é este o estado»* — significa *«é aqui que estou a carregar»*. Sem a pergunta
/// *«esta linha veio do módulo?»* a marca mudaria de sentido no meio de um clique.
///
/// **Mutações que devem sangrar:** apagar o `contrib.iter().any(...)` (a linha alheia acende sob o
/// dedo) · apagar o `matches!(..Pressed)` (as nove linhas do pulldown acendem todas) · devolver
/// `false` sempre (o defeito que o dono reportou).
#[test]
fn a_contributed_row_is_marked_by_the_state_its_module_publishes() {
    // Dois ids do MESMO módulo, os dois `Pressed`: um está no menu, o outro não.
    let (na_lista, fora_da_lista, apagada) = (
        ph2d_a11y::NodeId(901),
        ph2d_a11y::NodeId(902),
        ph2d_a11y::NodeId(903),
    );
    let mut store = WidgetStore::default();
    for id in [na_lista, fora_da_lista] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Pressed,
            },
        );
    }
    store.register(
        apagada,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    let contrib = vec![
        ToolRailEntry::compound(na_lista, "Render", "Render", ""),
        ToolRailEntry::compound(apagada, "Matcap", "Matcap", ""),
    ];

    assert!(
        contributed_row_is_current(na_lista, &contrib, &store),
        "a linha que o módulo publicou ACESA não recebeu a marca — é o report do dono"
    );
    assert!(
        !contributed_row_is_current(fora_da_lista, &contrib, &store),
        "uma linha que NÃO veio deste menu acendeu: o `Pressed` do dedo em baixo passou a ler-se \
         como estado"
    );
    assert!(
        !contributed_row_is_current(apagada, &contrib, &store),
        "a linha apagada do módulo recebeu marca — o menu passa a dizer que está em dois estados"
    );
    assert!(
        !contributed_row_is_current(na_lista, &[], &store),
        "controlo: sem contribuição nenhuma não há linha contribuída para marcar"
    );
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
