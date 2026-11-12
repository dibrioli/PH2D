//! **A aba Input Map** (plano `docs/InputMap/01_plano.md` §3): uma família de documentos com UMA aba; a shell chama
//! [`install`] com a vista das prefs e [`frame`] uma vez por quadro (abre a aba pedida pelo Settings ▸ Input Map…).

pub use ph2d_input_map_editor::{
    InputMapDoc, InputMapEdit, Snapshot, View, census, ids, interact, palette,
    surface_key::surface_raw_key, view_keyboard, view_list, view_try,
};

pub mod listen;
pub mod readers;
mod surface;

pub use ph2d_player_controls;
pub use surface::{InputMapTab, TAB_ID};

use ph2d_editor_core::HeroScreen;

/// O que está vivo para o [`frame`]: o `InputState` e a sessão do jogador (o mapa que o Play lê).
pub type Live<'a> = (
    &'a ph2d_input::InputState,
    &'a ph2d_player_controls::PlaySession,
);

pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "input-map",
    routers: &[],
};

pub fn install(hero: &mut HeroScreen, view: View) {
    hero.documents
        .install_surface(Box::new(InputMapTab::new(view)));
}

/// A vista escolhida — a que vai para `input_map_view=` nas prefs (a shell deriva-a, como as outras).
#[must_use]
pub fn view(hero: &HeroScreen) -> View {
    hero.documents
        .surfaces()
        .find::<InputMapTab>()
        .map_or_else(View::default, |t| t.doc.view())
}

/// **O quadro da família**: abre a aba pedida, aplica os pedidos dela no mapa e publica o snapshot.
pub fn frame(
    world: &ph2d_ecs::World,
    hero: &mut HeroScreen,
    gamepad_connected: bool,
    (live, jogador): Live<'_>,
) {
    let open = hero.store.take_input_map_tab_request();
    let focus = hero.store.take_input_map_action();
    let shown = ph2d_input_map_editor::active_is::<InputMapTab>(hero);
    let Some(tab) = hero.documents.surfaces_mut().find_mut::<InputMapTab>() else {
        return;
    };
    if open {
        tab.open();
    }
    // A busca escolheu uma ação: a vista List abre-a, destacada e à vista.
    if let Some(a) = focus {
        tab.doc.show_action(a);
    }
    let swapped = tab.doc.begin_frame(&hero.input_map);
    let edits = tab.doc.take_pending();
    let listen = tab.doc.take_listen_request();
    let file = tab.doc.player_file();
    for e in edits {
        ph2d_input_map_editor::edit::apply(&mut hero.input_map, e, file.as_deref());
    }
    let keep = !swapped && tab.is_open() && (shown || open);
    tab.doc
        .end_frame(&hero.input_map, &mut hero.store, listen, keep);
    let front = tab.is_open() && (shown || open);
    let listening = hero
        .store
        .input_map_listening()
        .filter(|_| !hero.store.input_map_listen_replaces());
    let play_running = ph2d_ecs::run_is_on_copy(world);
    let snap = Snapshot {
        map: hero.input_map.clone(),
        gamepad_connected,
        play_running,
        play_map: play_running.then(|| jogador.map(&hero.input_map).clone()),
        readers: readers::of_world(world),
        listening,
        game_id: world
            .get_resource::<ph2d_player_controls::GameId>()
            .map_or(0, |g| g.0),
    };
    if let Some(tab) = hero.documents.surfaces_mut().find_mut::<InputMapTab>() {
        ph2d_input_map_editor::inspect::set_snapshot(&mut tab.doc, snap, front);
        tab.doc.tick_try(live);
        tab.doc.tick_motion(&mut hero.motion);
    }
}

#[must_use]
pub fn gesture_in_progress(hero: &HeroScreen) -> bool {
    hero.documents
        .surfaces()
        .find::<InputMapTab>()
        .is_some_and(|t| t.doc.gesture_in_progress())
}
