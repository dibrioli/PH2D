//! ⭐ **A cena 10** (`PH2D_OBJECT_MODE_SMOKE=10`, D10 F4 — 08/10): a paleta que ENSINA. Depois da
//! cena 9 (layout *Vetor*, um rectângulo e uma elipse seleccionados, a Caneta na mão), a PALETA
//! escolhe *Stroke options* pela porta dela (`route_global_pick`) — o painel do traço abre por baixo
//! da barra e o chip «Stroke ▾» pisca onde o comando mora, até `FLASH_S` para a foto o apanhar;
//! depois o app é do dono. (Até 15/10 o comando era o *Snap ▸ ao caminho*; a F16 devolveu o encaixe
//! ao painel e a fila ② virou a barra de controle.)

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

/// Um passo da cena 10; devolve o próximo estágio, se mudou. `30` = a paleta escolhe · `32` = o
/// pulldown pisca · `9` = feito.
pub fn step(stage: u8, hero: &mut HeroScreen) -> Option<u8> {
    match stage {
        30 => {
            global_palette::route_global_pick(hero, ph2d_panel_vector::ids::VECTOR_BAR_STROKE);
            since(true);
            Some(32)
        }
        32 => {
            if since(false) >= FLASH_S {
                return Some(9);
            }
            if hero.store.flashing(false).is_none() {
                hero.store.flash(ph2d_panel_vector::ids::VECTOR_BAR_STROKE);
            }
            None
        }
        _ => None,
    }
}
