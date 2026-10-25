//! O traço: acumula entre premir e largar, e vira UM passo de desfazer.

use ph2d_pixel_core::{
    Background, ColorMode, FrameId, History, LayerId, NewSprite, Sample, Sprite, Stroke,
    compose_rgba, pick,
};

fn sprite(mode: ColorMode, background: Background) -> Sprite {
    Sprite::new(NewSprite {
        width: 32,
        height: 32,
        mode,
        background,
    })
    .expect("sprite")
}

fn ids(s: &Sprite) -> (LayerId, FrameId) {
    (s.layers()[0].id, s.frames()[0].id)
}

#[test]
fn one_stroke_is_one_undo_step_and_undo_gives_the_document_back_to_the_byte() {
    for mode in [ColorMode::Rgba, ColorMode::Gray, ColorMode::Indexed] {
        let mut s = sprite(mode, Background::Transparent);
        let before = s.clone();
        let (l, f) = ids(&s);
        let red = s.value_of_rgba([0xac, 0x32, 0x32, 0xff]);
        let mut st = Stroke::begin(&s, l, f, red, 2, 2);
        st.to(&s, 20, 9);
        st.to(&s, 5, 30);
        let mut h = History::default();
        {
            let op = st.op(&s).expect("mudou");
            h.commit(&mut s, op)
        }
        .expect("commit");
        assert_eq!(h.undo_len(), 1, "{mode:?}: um traço = um passo");
        assert_ne!(s, before);
        assert_eq!(s.value_at((l, f), 20, 9), red);
        let after = s.clone();
        assert!(h.undo(&mut s));
        assert_eq!(s, before, "{mode:?}: desfazer devolve o documento ao byte");
        assert!(h.redo(&mut s));
        assert_eq!(s, after, "{mode:?}: refazer devolve o traço");
        assert!(!h.redo(&mut s));
    }
}

#[test]
fn undo_shrinks_the_cel_back_to_the_box_it_had() {
    let mut s = sprite(ColorMode::Rgba, Background::Transparent);
    let (l, f) = ids(&s);
    let mut h = History::default();
    let a = Stroke::begin(&s, l, f, 0xff00_00ff, 1, 1);
    {
        let op = a.op(&s).expect("op");
        h.commit(&mut s, op)
    }
    .expect("commit");
    let one = s.clone();
    let mut b = Stroke::begin(&s, l, f, 0xff00_00ff, 30, 30);
    b.to(&s, 28, 31);
    {
        let op = b.op(&s).expect("op");
        h.commit(&mut s, op)
    }
    .expect("commit");
    assert_eq!(s.cel((l, f)).expect("cel").bbox.w, 30);
    assert!(h.undo(&mut s));
    assert_eq!(s, one);
    assert_eq!(s.cel((l, f)).expect("cel").bbox.w, 1);
}

#[test]
fn a_click_that_changes_nothing_spends_no_step() {
    let s = sprite(ColorMode::Rgba, Background::White);
    let (l, f) = ids(&s);
    let white = s.value_of_rgba([255, 255, 255, 255]);
    let st = Stroke::begin(&s, l, f, white, 3, 3);
    assert!(st.op(&s).is_none());
}

#[test]
fn the_pending_stroke_is_seen_over_its_layer_without_touching_the_document() {
    let s = sprite(ColorMode::Indexed, Background::Transparent);
    let (l, f) = ids(&s);
    let st = Stroke::begin(&s, l, f, 21, 4, 5);
    let mut out = Vec::new();
    compose_rgba(&s, f, Some(&st), &mut out);
    let i = (5 * 32 + 4) * 4;
    assert_eq!(&out[i..i + 4], &[255, 255, 255, 255]);
    compose_rgba(&s, f, None, &mut out);
    assert_eq!(&out[i..i + 4], &[0, 0, 0, 0], "o documento continua vazio");
}

#[test]
fn the_eraser_writes_transparency_and_the_eyedropper_reads_it_back() {
    let mut s = sprite(ColorMode::Rgba, Background::Transparent);
    let (l, f) = ids(&s);
    let green = s.value_of_rgba([0x6a, 0xbe, 0x30, 0xff]);
    let mut h = History::default();
    let mut st = Stroke::begin(&s, l, f, green, 0, 0);
    st.to(&s, 31, 0);
    {
        let op = st.op(&s).expect("op");
        h.commit(&mut s, op)
    }
    .expect("commit");
    assert_eq!(pick(&s, f, 10, 0, Sample::AllLayers), Some(green));
    let er = Stroke::begin(&s, l, f, s.transparent_value(), 10, 0);
    {
        let op = er.op(&s).expect("op");
        h.commit(&mut s, op)
    }
    .expect("commit");
    assert_eq!(
        pick(&s, f, 10, 0, Sample::AllLayers),
        Some(s.transparent_value())
    );
    assert_eq!(pick(&s, f, 11, 0, Sample::Layer(l)), Some(green));
    assert_eq!(
        pick(&s, f, 32, 0, Sample::AllLayers),
        None,
        "fora do sprite"
    );
}

#[test]
fn a_stroke_that_leaves_the_canvas_is_clipped_to_it() {
    let s = sprite(ColorMode::Rgba, Background::Transparent);
    let (l, f) = ids(&s);
    let mut st = Stroke::begin(&s, l, f, 0xff00_00ff, -5, 3);
    st.to(&s, 40, 3);
    let r = st.rect();
    assert_eq!((r.x, r.w), (0, 32));
}

#[test]
fn a_new_white_sprite_has_an_opaque_background_layer_in_every_mode() {
    for mode in [ColorMode::Rgba, ColorMode::Gray, ColorMode::Indexed] {
        let s = sprite(mode, Background::White);
        assert!(s.layers()[0].background);
        assert_eq!(s.layers()[0].name, "Background");
        let (_, f) = ids(&s);
        let mut out = Vec::new();
        compose_rgba(&s, f, None, &mut out);
        assert!(
            out.chunks_exact(4).all(|p| p == [255, 255, 255, 255]),
            "{mode:?}"
        );
    }
    let t = sprite(ColorMode::Rgba, Background::Transparent);
    assert_eq!(t.layers()[0].name, "Layer 1");
    assert!(t.cel(ids(&t)).is_none(), "transparente = nenhum cel");
}

#[test]
fn a_replacing_stroke_only_touches_the_cells_of_the_from_color() {
    let mut s = sprite(ColorMode::Rgba, Background::Transparent);
    let (l, f) = ids(&s);
    let (red, blue, green) = (0xff00_00ffu32, 0xffff_0000u32, 0xff00_ff00u32);
    let mut h = History::default();
    let mut a = Stroke::begin(&s, l, f, red, 0, 0);
    a.to(&s, 9, 0);
    let op = a.op(&s).expect("op");
    h.commit(&mut s, op).expect("commit");
    let b = Stroke::begin(&s, l, f, blue, 4, 0);
    let op = b.op(&s).expect("op");
    h.commit(&mut s, op).expect("commit");
    let mut r = Stroke::begin_replacing(&s, l, f, red, green, 0, 0);
    r.to(&s, 9, 0);
    let op = r.op(&s).expect("op");
    h.commit(&mut s, op).expect("commit");
    assert_eq!(s.value_at((l, f), 3, 0), green);
    assert_eq!(s.value_at((l, f), 4, 0), blue, "a célula de outra cor fica");
    let none = Stroke::begin_replacing(&s, l, f, red, green, 0, 1);
    assert!(none.op(&s).is_none(), "sem a cor de origem, nada muda");
}
