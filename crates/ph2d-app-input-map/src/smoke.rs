//! ⭐ **As cenas da aba Input Map** (`PH2D_GAME_SMOKE`, plano §8). O roteiro do jogo (`ph2d_app_game::smoke`) monta o
//! nível e diz QUAL cena pelo código (`"A1"`…`"B4"`: uma família não depende da outra); a shell chama [`arma`],
//! que põe o mapa do dono, pede a aba (`request_input_map_tab`, o caminho do Settings ▸ Input Map…) e põe a aba no
//! estado da cena. A aba é instalada no arranque (`install`), antes de a cena montar.

use crate::InputMapTab;
use ph2d_editor_core::HeroScreen;
use ph2d_input::{Binding, GamepadAxis, InputMap, Key};
use ph2d_input_map_editor::View;
use ph2d_input_map_editor::view_keyboard::state::Sub;

/// As cenas pela ordem do roteiro do jogo (A · List, depois B · Keyboard).
pub const CENAS: [&str; 8] = ["A1", "A2", "A3", "A4", "B1", "B2", "B3", "B4"];

/// A zona da manche do move_left nas cenas com comando (maquete A4 e B3).
pub const DEAD: f32 = 0.15;
pub const PRESS: f32 = 0.5;

/// O mapa do dono: as 8 de fábrica + fire `J`, reload sem tecla, laser `H`; `manche` liga a manche esquerda
/// (←) ao move_left com Dead 0.15 e Press 0.50.
#[must_use]
pub fn owner_map(manche: bool) -> InputMap {
    let mut map = InputMap::with_player_defaults();
    for (name, key) in [("fire", Some('J')), ("reload", None), ("laser", Some('H'))] {
        let a = map.create(name);
        if let (Some(k), Some(act)) = (key, map.get_mut(a)) {
            act.bindings.push(Binding::Key(Key(u32::from(k))));
        }
    }
    let ml = map.id(ph2d_input::PLAYER_MOVE_LEFT);
    if let Some(a) = ml.and_then(|id| map.get_mut(id)).filter(|_| manche) {
        a.bindings.push(Binding::PadAxis {
            axis: GamepadAxis::LeftStickX,
            positive: false,
        });
        a.set_zone(DEAD, PRESS);
    }
    map
}

/// A tecla ↑ (a da busca A3 e do ponteiro B2).
fn up_arrow() -> Option<Key> {
    Key::named_in_script("up").and_then(|v| v.first().copied())
}

/// Põe o mapa, abre a aba e arma a cena `cena`. `false` = código desconhecido ou a aba não está instalada.
pub fn arma(hero: &mut HeroScreen, cena: &str) -> bool {
    if !CENAS.contains(&cena) {
        return false;
    }
    let map = owner_map(matches!(cena, "A4" | "B3"));
    let id = |n: &str| map.id(n);
    let (jump, move_left, reload) = (id("jump"), id(ph2d_input::PLAYER_MOVE_LEFT), id("reload"));
    hero.input_map = map;
    hero.store.request_input_map_tab();
    let Some(tab) = hero.documents.surfaces_mut().find_mut::<InputMapTab>() else {
        return false;
    };
    let doc = &mut tab.doc;
    let lista = cena.starts_with('A');
    doc.smoke_view(if lista { View::List } else { View::Keyboard });
    match cena {
        "A2" => reload.into_iter().for_each(|r| doc.smoke_listen(r)),
        "A3" => doc.smoke_search_key(up_arrow()),
        "B1" => doc.smoke_open(jump),
        "B2" => {
            doc.smoke_open(jump);
            doc.smoke_hover_key(up_arrow());
        }
        "B3" => {
            doc.smoke_open(move_left);
            doc.smoke_sub(Sub::Gamepad);
        }
        "B4" => doc.smoke_open(reload),
        _ => {}
    }
    true
}
