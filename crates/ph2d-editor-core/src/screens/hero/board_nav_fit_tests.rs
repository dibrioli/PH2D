use super::free_of;
use crate::zones::Rect;

const AREA: Rect = Rect {
    x: 312.0,
    y: 22.0,
    w: 1310.0,
    h: 990.0,
};

#[test]
fn each_cover_cuts_the_free_rect_on_the_side_that_keeps_more() {
    // A cena 22 a 1930×1040 (foto de 18/10): a barra à esquerda, o cartão «Live» em cima à direita,
    // a fila do canto em baixo à direita e o HUD em baixo ao meio.
    let bar = Rect::new(316.0, 30.0, 32.0, 515.0);
    let live = Rect::new(1374.0, 30.0, 240.0, 365.0);
    let corner = Rect::new(1474.0, 942.0, 144.0, 68.0);
    let hud = Rect::new(462.0, 962.0, 1006.0, 30.0);
    let free = free_of(AREA, &[corner, bar, hud, live]);
    assert_eq!(free, Rect::new(348.0, 22.0, 1026.0, 940.0));
    // Nada a tapar: a área inteira; uma coberta fora dela não corta nada.
    assert_eq!(free_of(AREA, &[]), AREA);
    assert_eq!(free_of(AREA, &[Rect::new(0.0, 0.0, 300.0, 900.0)]), AREA);
}

#[test]
fn a_tall_cover_on_the_right_keeps_the_left_and_a_wide_one_at_the_bottom_keeps_the_top() {
    let right = Rect::new(1500.0, 22.0, 122.0, 990.0);
    assert_eq!(
        free_of(AREA, &[right]),
        Rect::new(312.0, 22.0, 1188.0, 990.0)
    );
    let bottom = Rect::new(312.0, 900.0, 1310.0, 112.0);
    assert_eq!(
        free_of(AREA, &[bottom]),
        Rect::new(312.0, 22.0, 1310.0, 878.0)
    );
}
