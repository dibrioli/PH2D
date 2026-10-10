//! A área onde um enquadramento pode pôr o conteúdo (F8).

use super::*;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::layout::HeroLayout;

/// ⭐ **A pílula ⑤ do rodapé flutua por cima da área: um enquadramento acaba no topo dela** (medido na
/// foto da cena 14); sem pílula, a área é a visível inteira.
#[test]
fn a_frame_stops_at_the_top_of_the_footer_pill() {
    let viewport = Rect::new(0.0, 0.0, 1600.0, 1000.0);
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.last_content = Rect::new(300.0, 80.0, 1000.0, 900.0);
    let mut layout = HeroLayout::for_viewport(viewport);
    let e = hero.escala();
    let visivel = visible(&hero, viewport);
    layout.bottom_hud = Rect::new(500.0, 930.0, 600.0, 40.0);
    hero.last_layout = Some(layout);
    let r = frameable(&hero, viewport);
    assert_eq!((r.x, r.y, r.w), (visivel.x, visivel.y, visivel.w));
    let topo = e.rect_to_physical(layout.bottom_hud).y;
    assert!(
        (r.y + r.h - topo).abs() < 1e-3,
        "{r:?} não acaba no topo da pílula ({topo})"
    );
    layout.bottom_hud.h = 0.0;
    hero.last_layout = Some(layout);
    assert_eq!(frameable(&hero, viewport), visivel);
}
