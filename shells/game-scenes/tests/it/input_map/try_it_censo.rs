//! A prova «0 textos cortados, 0 números mortos» na vista C (plano §3): o mapa de fábrica e o de 11 acções (o
//! move_left numa manche), a 1600×1000 e 1280×720, com a confirmação aberta e fechada — só a acção da manche mostra
//! os 2 números.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_try::state::TryHit;
use ph2d_editor_core::zones::Rect;
use ph2d_input::recipes::Recipe;

const SIZES: [(f32, f32); 2] = [(1600.0, 1000.0), (1280.0, 720.0)]; // LITERAL-PX-OK: as janelas do plano §8

#[test]
fn o_try_it_nao_corta_texto_nem_mostra_numero_morto() {
    for (label, map) in [
        ("fábrica", ph2d_input::InputMap::with_player_defaults()),
        ("onze", onze()),
    ] {
        for (w, h) in SIZES {
            let area = Rect::new(0.0, 0.0, w, h);
            let mut m = Mesa::aberta(View::TryIt);
            m.hero.input_map = map.clone();
            m.frame();
            for confirm in [false, true] {
                if confirm {
                    m.click(m.at(TryHit::Recipe(Recipe::Shooter).id()));
                }
                m.paint_in(area);
                let c = &m.tab().doc.census;
                let at = format!("{label} {w}×{h} confirm={confirm}");
                assert_eq!(c.elided(), Vec::<&str>::new(), "{at}: texto cortado");
                let stick = map.actions().iter().find(|a| a.is_analog()).map(|a| a.id);
                for b in map.actions() {
                    let want = if Some(b.id) == stick { 2 } else { 0 };
                    assert_eq!(c.numbers_of(b.id), want, "{at}: números de {}", b.name);
                }
            }
        }
    }
}
