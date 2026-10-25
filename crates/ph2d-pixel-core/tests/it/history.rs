//! O tecto do histórico é em bytes: o mais velho cai, o mais novo fica sempre.

use ph2d_pixel_core::{Background, ColorMode, History, NewSprite, Sprite, Stroke};

#[test]
fn the_byte_cap_drops_the_oldest_step_and_keeps_the_newest() {
    let mut s = Sprite::new(NewSprite {
        width: 64,
        height: 64,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let mut h = History::with_cap(2_000);
    for y in 0..20 {
        let mut st = Stroke::begin(&s, l, f, 0xff00_00ff + y as u32, 0, y);
        st.to(&s, 63, y);
        {
            let op = st.op(&s).expect("op");
            h.commit(&mut s, op)
        }
        .expect("commit");
        assert!(
            h.bytes() <= 2_000 || h.undo_len() == 1,
            "passo {y}: {} bytes",
            h.bytes()
        );
    }
    assert!(h.undo_len() < 20, "o tecto não cortou nada");
    assert!(h.undo_len() >= 1);
}
