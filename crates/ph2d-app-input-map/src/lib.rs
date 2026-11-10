//! **A aba Input Map** (plano `docs/InputMap/01_plano.md` §3): uma família de documentos ([`DocumentSurface`]) com
//! UMA aba. A shell instala-a com a vista das prefs ([`install`]) e chama [`frame`] uma vez por quadro: o
//! Settings ▸ Input Map… chega como o pedido do `WidgetStore`, e o `frame` abre (ou traz à frente) a aba.

pub use ph2d_input_map_editor::{InputMapDoc, InputMapEdit, Snapshot, View};

mod apply;
mod surface;

pub use surface::{InputMapTab, TAB_ID};

use ph2d_editor_core::HeroScreen;

/// A declaração da família à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "input-map",
    routers: &[],
};

/// Instala a família; `view` é a das prefs (`None` = a 1.ª vez: Keyboard).
pub fn install(hero: &mut HeroScreen, view: Option<View>) {
    hero.documents
        .install_surface(Box::new(InputMapTab::new(view.unwrap_or_default())));
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
    for e in edits {
        apply::apply(&mut hero.input_map, e);
    }
    let snap = Snapshot {
        map: hero.input_map.clone(),
        gamepad_connected,
        play_running: ph2d_ecs::run_is_on_copy(world),
    };
    if let Some(tab) = hero.documents.surfaces_mut().find_mut::<InputMapTab>() {
        tab.doc.set_snapshot(snap);
    }
}

/// ⭐ **A tecla física para a aba de família activa** que tem o teclado ([`DocumentSurface::raw_key`]): a shell
/// chama-a depois da escuta e dos modais, antes de todo atalho. `code` é o `ph2d_input::Key` normalizado;
/// `text` é o texto do evento. `true` = consumida.
///
/// [`DocumentSurface::raw_key`]: ph2d_editor_core::doc_surface::DocumentSurface::raw_key
pub fn surface_raw_key(hero: &mut HeroScreen, code: u32, pressed: bool, text: Option<&str>) -> bool {
    if !ph2d_editor_core::screens::hero::surface_input::owns_the_keyboard(Some(hero)) {
        return false;
    }
    let writes = text.is_some_and(|t| t.chars().any(|c| !c.is_control()));
    hero.documents
        .active_surface_mut()
        .is_some_and(|(f, id)| f.raw_key(id, ph2d_input::Key(code), pressed, writes))
}
