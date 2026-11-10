//! **A aba Input Map** (plano `docs/InputMap/01_plano.md` §3): uma família de documentos ([`DocumentSurface`]) com
//! UMA aba. A shell instala-a com a vista das prefs ([`install`]) e chama [`frame`] uma vez por quadro: o
//! Settings ▸ Input Map… chega como o pedido do `WidgetStore`, e o `frame` abre (ou traz à frente) a aba.

pub use ph2d_input_map_editor::{
    InputMapDoc, InputMapEdit, Snapshot, View, ids, view_keyboard, view_list,
};

mod apply;
pub mod listen;
pub mod readers;
pub mod smoke;
mod surface;

pub use surface::{InputMapTab, TAB_ID};

use ph2d_editor_core::HeroScreen;

/// A declaração da família à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "input-map",
    routers: &[],
};

/// Instala a família; `view` é a das prefs (a de omissão, a 1.ª vez, é o Keyboard).
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
pub fn frame(world: &ph2d_ecs::World, hero: &mut HeroScreen, gamepad_connected: bool) {
    let open = hero.store.take_input_map_tab_request();
    let Some(tab) = hero.documents.surfaces_mut().find_mut::<InputMapTab>() else {
        return;
    };
    if open {
        tab.open();
    }
    let edits = tab.doc.take_pending();
    let listen = tab.doc.take_listen_request();
    for e in edits {
        apply::apply(&mut hero.input_map, e);
    }
    match listen {
        Some(Some(id)) => hero.store.listen_for_binding(id),
        Some(None) => hero.store.stop_listening(),
        None => {}
    }
    // A tecla (ou o comando) apanhada vira ligação no braço `INPUT_MAP_BIND_CAPTURED` do editor-core.
    let listening = hero
        .store
        .input_map_listening()
        .filter(|_| !hero.store.input_map_listen_replaces());
    let snap = Snapshot {
        map: hero.input_map.clone(),
        gamepad_connected,
        play_running: ph2d_ecs::run_is_on_copy(world),
        readers: readers::of_world(world),
        listening,
    };
    if let Some(tab) = hero.documents.surfaces_mut().find_mut::<InputMapTab>() {
        tab.doc.set_snapshot(snap);
    }
}

/// O undo do projecto pergunta: há um arrasto de número vivo na aba?
#[must_use]
pub fn gesture_in_progress(hero: &HeroScreen) -> bool {
    hero.documents
        .surfaces()
        .find::<InputMapTab>()
        .is_some_and(|t| t.doc.gesture_in_progress())
}

/// ⭐ **A tecla física para a aba de família activa** que tem o teclado ([`DocumentSurface::raw_key`]): a shell
/// chama-a depois da escuta e dos modais, antes de todo atalho. `code` é o `ph2d_input::Key` normalizado;
/// `text` é o texto do evento. `true` = consumida.
///
/// [`DocumentSurface::raw_key`]: ph2d_editor_core::doc_surface::DocumentSurface::raw_key
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
