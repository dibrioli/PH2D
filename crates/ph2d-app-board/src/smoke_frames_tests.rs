//! ⭐ A cena 16 ARMA o que as lições dizem: três molduras (contínua, tracejada, à mão e redonda), a
//! MESMA fotografia a cores e a preto e branco, o mesmo SVG pequeno e grande; e o desenho da cena leva
//! o traço de cada moldura, os píxeis cinzentos da cinzenta e o SVG rasterizado ao tamanho de cada um.

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use ph2d_board_model::Image;

fn img(h: &HeroScreen, id: ElementId) -> Image {
    let b = h.documents.active_board().unwrap();
    b.doc.get(id).unwrap().image().unwrap().clone()
}

#[test]
fn the_frames_scene_does_what_its_lessons_say() {
    crate::images::install();
    let mut h = hero("Ana");
    let l = scene_frames(&mut h);
    Hand::new().paint(&mut h);
    let b = h.documents.active_board().unwrap();
    assert_eq!(b.id, l.board);
    assert_eq!(b.files.len(), 2, "a fotografia e o logótipo, uma vez cada");
    let dashes: Vec<Dash> = l.framed.iter().map(|id| img(&h, *id).style.dash).collect();
    assert_eq!(dashes, [Dash::Solid, Dash::Dashed, Dash::Solid]);
    for id in l.framed {
        assert!(img(&h, id).style.stroke.is_some(), "uma moldura sem cor");
    }
    let hand = img(&h, l.framed[2]).style;
    assert!(hand.sketch && hand.round);
    let (c, g) = (img(&h, l.color), img(&h, l.gray));
    assert_eq!(c.file, g.file, "a MESMA fotografia");
    assert_eq!((c.filter, g.filter), (ImageFilter::Original, ImageFilter::Gray));
    let (s, big) = (img(&h, l.svg_small), img(&h, l.svg_big));
    assert_eq!(s.file, big.file);
    let f = b.file(s.file).unwrap();
    assert_eq!((f.mime(), f.size), (Some("image/svg+xml"), logo().size));
    assert_eq!(l.lessons.len(), 6);
    let lesson5 = b.doc.get(l.lessons[4]).unwrap().shape().unwrap().text.as_str().to_owned();
    assert!(lesson5.contains("Send to board…"), "{lesson5}");
}
