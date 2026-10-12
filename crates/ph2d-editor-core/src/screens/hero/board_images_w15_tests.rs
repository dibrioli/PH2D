//! W15 — a moldura e o preto e branco de uma imagem pelo clique REAL na barra dela: os grupos que
//! aparecem, o contorno que o clique escreve no estilo da imagem, o botão que acende e o desfazer.

use super::*;
use crate::screens::hero::board_images::insert_images;
use ph2d_board_model::{ImageFile, ImageFilter};

fn put(t: &mut T, n: u8) {
    assert_eq!(
        insert_images(
            &mut t.hero,
            vec![ImageFile::new(vec![n; 4], [120, 80])],
            None
        ),
        1
    );
    t.paint();
}

fn image(t: &T) -> ph2d_board_model::Image {
    t.elements()
        .into_iter()
        .find_map(|e| e.image().cloned())
        .expect("uma imagem")
}

/// ⭐ A barra de uma imagem oferece a moldura (cor, espessura, traço) e o preto e branco; o clique na
/// cor põe o contorno (sem ele, a imagem nasce sem moldura), a espessura e o tracejado mudam-no — cada
/// um UM passo de desfazer.
#[test]
fn the_image_bar_frames_the_picture() {
    let mut t = T::new();
    put(&mut t, 1);
    assert_eq!(image(&t).style.stroke, None, "nasce sem moldura");
    for it in [
        Item::Stroke(None),
        Item::Width(0),
        Item::Dash(Dash::Dashed),
        Item::Gray,
    ] {
        assert!(t.find(node(it)).is_some(), "{it:?} na barra da imagem");
    }
    t.click_bar(Item::Stroke(Some(Some(BLACK))));
    assert_eq!(image(&t).style.stroke, Some(palette(BLACK)));
    let thick = WIDTHS.len() - 1;
    t.click_bar(Item::Width(thick));
    t.click_bar(Item::Dash(Dash::Dotted));
    let st = image(&t).style;
    assert_eq!((st.stroke_width, st.dash), (WIDTHS[thick], Dash::Dotted));
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert_eq!(image(&t).style.dash, Dash::Solid);
    t.click_bar(Item::Stroke(None));
    assert_eq!(
        image(&t).style.stroke,
        None,
        "«sem contorno» tira a moldura"
    );
}

/// ⭐ O botão preto e branco: um clique põe a imagem cinzenta (e o botão acende), outro devolve as
/// cores; o desfazer volta um passo.
#[test]
fn the_gray_button_toggles_and_lights() {
    let mut t = T::new();
    put(&mut t, 2);
    let lit = |t: &T| {
        let d = &t.hero.documents;
        d.live.editor.as_ref()?.images_gray(&d.active_board()?.doc)
    };
    assert_eq!(lit(&t), Some(false));
    t.click_bar(Item::Gray);
    assert_eq!(image(&t).filter, ImageFilter::Gray);
    assert_eq!(lit(&t), Some(true));
    t.click_bar(Item::Gray);
    assert_eq!(image(&t).filter, ImageFilter::Original);
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert_eq!(image(&t).filter, ImageFilter::Gray);
}

/// O «Rascunho ↔ Final» com a imagem seleccionada põe a moldura à mão (W15: a imagem é «sketchable»).
#[test]
fn the_sketch_button_reaches_the_frame() {
    let mut t = T::new();
    put(&mut t, 3);
    assert!(!image(&t).style.sketch);
    t.click_bar(Item::Sketch);
    assert!(image(&t).style.sketch);
}

/// Os ids dos controlos que já existiam não se movem com o preto e branco (vai depois do «Forma»;
/// o que veio a seguir, W16, vai depois dele).
#[test]
fn the_gray_item_is_last_and_moves_no_id() {
    let v = items();
    let r = v
        .iter()
        .position(|i| *i == Item::Reshape)
        .expect("o «Forma»");
    assert_eq!(v[r..r + 2], [Item::Reshape, Item::Gray]);
}
