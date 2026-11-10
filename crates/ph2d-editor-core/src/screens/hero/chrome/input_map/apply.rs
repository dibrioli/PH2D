//! **A METADE DE DESPACHO do Input Map** — o abridor da aba e a ligação que a escuta apanhou.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;
use ph2d_input::Binding;

/// A metade de DESPACHO — ligada ao `dispatch_all` pelo `ph2d-chrome-sync`.
pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_INPUT_MAP {
        // A aba central (plano InputMap §3); quem a abre é a família.
        hero.store.request_input_map_tab();
        return true;
    }
    // O `Esc` durante a escuta: a lei já desarmou; aqui só se consome o evento.
    if id == ids::INPUT_MAP_LISTEN_CANCELLED {
        return true;
    }
    if id == ids::INPUT_MAP_BIND_CAPTURED {
        let captured = hero.store.take_captured_binding();
        let armed = hero.store.input_map_listening();
        let troca = hero.store.input_map_listen_replaces();
        hero.store.stop_listening();
        if let (Some(b), Some(aid)) = (captured, armed)
            && let Some(a) = hero.input_map.get_mut(aid)
        {
            // A escuta do Trigger TROCA as teclas de teclado; as de comando ficam (dono 09/11).
            if troca && matches!(b, Binding::Key(_)) {
                a.bindings.retain(|o| !matches!(o, Binding::Key(_)));
            }
            // A mesma ligação duas vezes seriam duas linhas iguais, indistinguíveis ao apagar.
            if !a.bindings.contains(&b) {
                a.bindings.push(b);
            }
        }
        return true;
    }
    false
}
