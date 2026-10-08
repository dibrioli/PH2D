//! ⭐ **A cena 10** (`PH2D_OBJECT_MODE_SMOKE=10`, D10 F4 — 08/10): o ALCANCE das barras. Depois da
//! cena 9 (layout *Vetor*, um rectângulo e uma elipse seleccionados, a Caneta na mão):
//!
//! 1. a RODA abre ao meio da área, com o leque de **Draw** aberto e o Lápis aceso — o mesmo estado
//!    que segurar `J` e apontar dá (o modelo é o `build_radial_model`, o ponteiro o `radial_point`);
//! 2. ao fim de `RADIAL_S`, a roda fecha SEM escolher, e a PALETA escolhe *Snap ▸ ao caminho*
//!    pela porta dela (`route_global_pick`) — o pulldown pisca onde o comando mora, e volta a
//!    piscar até `FLASH_S` para a foto o apanhar;
//! 3. depois a cena pára e o app é do dono.

use ph2d_editor_core::screens::hero::{HeroScreen, global_palette, radial};
use std::cell::Cell;
use std::time::Instant;

/// Quanto a roda fica aberta (s) — o bastante para a foto, e para o dono ler os sectores.
const RADIAL_S: f32 = 6.0;
/// Até quando o pulldown volta a piscar (s, contados do fecho da roda).
const FLASH_S: f32 = 8.0;

thread_local! {
    static T0: Cell<Option<Instant>> = const { Cell::new(None) };
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

fn since(reset: bool) -> f32 {
    T0.with(|c| {
        if reset || c.get().is_none() {
            c.set(Some(Instant::now()));
        }
        c.get().map_or(0.0, |t| t.elapsed().as_secs_f32())
    })
}

/// Um passo da cena 10; devolve o próximo estágio, se mudou. `30` = abrir a roda · `31` = roda
/// aberta · `32` = a paleta escolheu e o pulldown pisca · `9` = feito.
pub fn step(stage: u8, hero: &mut HeroScreen) -> Option<u8> {
    match stage {
        30 => {
            let area = hero.last_content;
            let c = [area.x + area.w * 0.5, area.y + area.h * 0.55];
            let model = radial::build_radial_model(hero);
            let draw = model.iter().position(|i| {
                i.label == ph2d_i18n::tr(ph2d_editor_core::interaction::BarFamily::Draw.name_key())
            })?;
            let n = model.len();
            hero.store.open_radial(c, model.clone()).then_some(())?;
            let o = ph2d_editor_core::widget::radial_item_offset(draw, n);
            hero.store.radial_point([c[0] + o[0], c[1] + o[1]]);
            let open = hero.store.radial().cloned()?;
            let fc = ph2d_editor_core::widget::radial_fan_center(&open, draw);
            let k = model[draw].children.len();
            let f = ph2d_editor_core::widget::radial_fan_offset(
                1.min(k.saturating_sub(1)),
                k,
                ph2d_editor_core::widget::radial_outward(draw, n),
            );
            hero.store.radial_point([fc[0] + f[0], fc[1] + f[1]]);
            since(true);
            Some(31)
        }
        31 if since(false) >= RADIAL_S => {
            // Fecha SEM escolher (a zona morta), e a paleta escolhe pela porta dela.
            hero.store.radial_point(hero.store.radial()?.center);
            let _ = hero.store.close_radial();
            global_palette::route_global_pick(hero, snap_row(hero)?);
            since(true);
            Some(32)
        }
        32 => {
            let t = since(false);
            if t >= FLASH_S {
                return Some(9);
            }
            // De novo a cada três pulsos, para a foto apanhar o anel aceso.
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
