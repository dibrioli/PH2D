//! A prova «0 textos cortados, 0 números mortos» na vista B (plano §3): o mapa de fábrica e o de 11 acções, cada
//! acção aberta por vez, nas duas sub-vistas, a 1600×1000 e 1280×720 — e só a aberta com manche mostra 2 números.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_editor_core::zones::Rect;

const SIZES: [(f32, f32); 2] = [(1600.0, 1000.0), (1280.0, 720.0)]; // LITERAL-PX-OK: as janelas do plano §8

#[test]
fn o_teclado_nao_corta_texto_nem_mostra_numero_morto() {
    for (label, map) in [
        ("fábrica", ph2d_input::InputMap::with_player_defaults()),
        ("onze", onze()),
    ] {
        for (w, h) in SIZES {
            let area = Rect::new(0.0, 0.0, w, h);
            let mut m = Mesa::aberta(View::Keyboard);
            m.hero.input_map = map.clone();
            m.frame();
            for sub in [Sub::Keyboard, Sub::Gamepad] {
                m.click(m.at(KbHit::Sub(sub).id()));
                for a in map.actions() {
                    m.click(m.at(KbHit::Row(a.id).id()));
                    m.paint_in(area);
                    let c = &m.tab().doc.census;
                    let at = format!("{label} {w}×{h} {sub:?} {}", a.name);
                    assert_eq!(c.elided(), Vec::<&str>::new(), "{at}: texto cortado");
                    for b in map.actions() {
                        let want = if b.id == a.id && b.is_analog() { 2 } else { 0 };
                        assert_eq!(c.numbers_of(b.id), want, "{at}: números de {}", b.name);
                    }
                }
            }
        }
    }
}
