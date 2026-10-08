//! ⭐ **A cena 10** (`PH2D_OBJECT_MODE_SMOKE=10`, D10 F4 — 08/10): a paleta que ENSINA. Depois da
//! cena 9 (layout *Vetor*, um rectângulo e uma elipse seleccionados, a Caneta na mão), a PALETA
//! escolhe *Snap ▸ ao caminho* pela porta dela (`route_global_pick`) — o pulldown pisca onde o
//! comando mora, e volta a piscar até `FLASH_S` para a foto o apanhar; depois o app é do dono.

use ph2d_editor_core::screens::hero::{HeroScreen, global_palette};
use std::cell::Cell;
use std::time::Instant;

/// Até quando o pulldown volta a piscar (s, contados da escolha).
const FLASH_S: f32 = 8.0;

thread_local! {
    static T0: Cell<Option<Instant>> = const { Cell::new(None) };
}

fn since(reset: bool) -> f32 {
    T0.with(|c| {
        if reset || c.get().is_none() {
            c.set(Some(Instant::now()));
        }
        c.get().map_or(0.0, |t| t.elapsed().as_secs_f32())
    })
}

/// *Snap ▸ ao caminho* — pelo NOME do pulldown, nunca pela posição: o 1.º da área é o seletor de
/// MODO, e a 1.ª linha dele (Object Mode) tirava o vetor da mão (a 1.ª foto, 08/10).
fn snap_row(hero: &HeroScreen) -> Option<ph2d_editor_core::NodeId> {
    let snap = ph2d_i18n::tr("panel.vector.bar.snap");
    hero.store
        .area_menus()
        .iter()
        .find(|m| m.label == snap)?
        .rows
        .iter()
        .filter_map(ph2d_editor_core::widget::ToolRailEntry::node_id)
        .nth(1)
}

/// Um passo da cena 10; devolve o próximo estágio, se mudou. `30` = a paleta escolhe · `32` = o
/// pulldown pisca · `9` = feito.
pub fn step(stage: u8, hero: &mut HeroScreen) -> Option<u8> {
    match stage {
        30 => {
            global_palette::route_global_pick(hero, snap_row(hero)?);
            since(true);
            Some(32)
        }
        32 => {
            if since(false) >= FLASH_S {
                return Some(9);
            }
            // ⚠️ Pelo id que a linha tem AGORA: o interruptor trocou-o ao ser executado.
            if hero.store.flashing(false).is_none()
                && let Some(id) = snap_row(hero)
            {
                hero.store.flash(id);
            }
            None
        }
        _ => None,
    }
}
