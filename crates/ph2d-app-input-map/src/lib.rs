//! **A aba Input Map** (plano `docs/InputMap/01_plano.md` §3): uma família de documentos com UMA aba; a shell chama
//! [`install`] com a vista das prefs e [`frame`] uma vez por quadro (abre a aba pedida pelo Settings ▸ Input Map…).

pub use ph2d_input_map_editor::{
    InputMapDoc, InputMapEdit, Snapshot, View, census, ids, interact, palette, view_keyboard,
    view_list, view_try,
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
    let shown = ph2d_input_map_editor::active_is::<InputMapTab>(hero);
    let Some(tab) = hero.documents.surfaces_mut().find_mut::<InputMapTab>() else {
        return;
    };
    if open {
        tab.open();
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
    // A tecla (ou o comando) apanhada vira ligação no braço `INPUT_MAP_BIND_CAPTURED` do editor-core.
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
        tab.doc.set_snapshot(snap);
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

/// ⭐ **A tecla física para a aba de família activa** que tem o teclado (`DocumentSurface::raw_key`), depois da escuta
/// e dos modais, antes de todo atalho. `code` = o `ph2d_input::Key` normalizado; `true` = consumida.
pub fn surface_raw_key(
    hero: &mut HeroScreen,
    code: u32,
    pressed: bool,
    text: Option<&str>,
) -> bool {
    if !ph2d_editor_core::screens::hero::surface_input::owns_the_keyboard(Some(hero)) {
        return false;
    }
    let writes = text.is_some_and(|t| t.chars().any(|c| !c.is_control()));
    hero.documents
        .active_surface_mut()
        .is_some_and(|(f, id)| f.raw_key(id, ph2d_input::Key(code), pressed, writes))
}
