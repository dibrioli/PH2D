//! A prova «0 textos cortados, 0 números mortos» na vista B (plano §3): o mapa de fábrica e o de 11 acções, cada
//! acção aberta por vez, nas duas sub-vistas, a 1600×1000 e 1280×720 — e só a aberta com manche mostra 2 números.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_editor_core::zones::Rect;

/// A área que a aba RECEBE em cada janela do plano §8 (medida nas fotos b1_1600 e b4_1280): dar à aba a janela
/// inteira deixava o censo verde enquanto a 1280 o app cortava a frase do cartão aberto e o aviso.
pub(super) const AREAS: [Rect; 2] = [
    Rect::new(308.0, 28.0, 988.0, 890.0), // LITERAL-PX-OK: a área central na foto da janela 1600×1000
    Rect::new(297.0, 28.0, 690.0, 604.0), // LITERAL-PX-OK: a área central na foto da janela 1280×720
];

#[test]
fn o_teclado_nao_corta_texto_nem_mostra_numero_morto() {
    super::em_cada_modo_de_texto(|| {
        for (label, map) in [
            ("fábrica", ph2d_input::InputMap::with_player_defaults()),
            ("onze", onze()),
        ] {
            for area in AREAS {
                let (w, h) = (area.w, area.h);
                let mut m = Mesa::aberta(View::Keyboard);
                m.hero.input_map = map.clone();
                m.frame();
                for sub in [Sub::Keyboard, Sub::Gamepad] {
                    // O clique cai na pintura da mesa (o controlo dispara no Up dentro do rect que o quadro repinta).
                    m.paint();
                    m.click(m.at(KbHit::Sub(sub).id()));
                    for a in map.actions() {
                        m.paint();
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
    });
}
