use super::free_of;
use crate::zones::Rect;

/// A área do quadro a 1366×768 com os dois painéis laterais abertos (foto de 18/10, cena 22).
const AREA: Rect = Rect {
    x: 312.0,
    y: 22.0,
    w: 746.0,
    h: 718.0,
};
/// O cartão «Live» no canto de cima à direita.
const LIVE: Rect = Rect {
    x: 810.0,
    y: 30.0,
    w: 240.0,
    h: 365.0,
};
const M: f64 = 16.0;

#[test]
fn a_wide_scene_goes_below_the_card_and_a_tall_one_beside_it() {
    let wide = [0.0, 0.0, 1640.0, 880.0];
    assert_eq!(
        free_of(AREA, &[LIVE], wide, M),
        Rect::new(312.0, 395.0, 746.0, 345.0)
    );
    let tall = [0.0, 0.0, 400.0, 1600.0];
    assert_eq!(
        free_of(AREA, &[LIVE], tall, M),
        Rect::new(312.0, 22.0, 498.0, 718.0)
    );
}

#[test]
fn every_cover_cuts_and_none_outside_does() {
    let bx = [0.0, 0.0, 1000.0, 1000.0];
    assert_eq!(free_of(AREA, &[], bx, M), AREA);
    assert_eq!(
        free_of(AREA, &[Rect::new(0.0, 0.0, 300.0, 900.0)], bx, M),
        AREA
    );
    // A barra das ferramentas à esquerda e a fila do canto em baixo à direita: nada as toca.
    let bar = Rect::new(316.0, 30.0, 32.0, 515.0);
    let corner = Rect::new(940.0, 614.0, 114.0, 66.0);
    let free = free_of(AREA, &[bar, corner, LIVE], bx, M);
    for c in [bar, corner, LIVE] {
        let apart = free.x + free.w <= c.x
            || c.x + c.w <= free.x
            || free.y + free.h <= c.y
            || c.y + c.h <= free.y;
        assert!(apart, "{free:?} toca {c:?}");
    }
}
