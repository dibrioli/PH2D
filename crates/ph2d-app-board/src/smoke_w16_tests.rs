//! ⭐ A cena 17 ARMA o que as lições dizem: a fotografia inteira (que o `Enter` recorta, sem abrir
//! texto) ao lado de uma cópia recortada, as três do espelho, e um GIF que tem mesmo vários quadros.

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use ph2d_board_model::Image;

fn img(h: &HeroScreen, id: ElementId) -> Image {
    let b = h.documents.active_board().unwrap();
    b.doc.get(id).unwrap().image().unwrap().clone()
}

#[test]
fn the_w16_scene_does_what_its_lessons_say() {
    crate::images::install();
    let mut h = hero("Ana");
    let l = scene_w16(&mut h);
    Hand::new().paint(&mut h);
    let b = h.documents.active_board().unwrap();
    assert_eq!(b.id, l.board);
    assert_eq!(b.files.len(), 2, "a fotografia e o GIF, uma vez cada");
    assert_eq!(img(&h, l.whole).crop, None);
    assert!(img(&h, l.cropped).crop.is_some());
    let flips: Vec<[bool; 2]> = l.flips.iter().map(|id| img(&h, *id).flip).collect();
    assert_eq!(flips, [[false, false], [true, false], [false, true]]);
    let g = b.file(img(&h, l.gif).file).unwrap();
    assert_eq!(g.mime(), Some("image/gif"));
    let frames = crate::images::frames(&g.bytes.0).expect("um GIF").count();
    assert_eq!(frames, GIF_FRAMES as usize, "o GIF anima mesmo");
    assert_eq!(l.lessons.len(), 6);
    // A lição 1 pelo rato REAL: duplo-clique na fotografia inteira, e o canto de baixo à direita
    // arrastado para dentro recorta-a (antes o duplo-clique abria um texto vazio por cima dela).
    let whole = b.doc.get(l.whole).unwrap().clone();
    let mut hand = Mouse::new();
    let mid = [whole.x + whole.w / 2.0, whole.y + whole.h / 2.0];
    hand.click(&mut h, mid);
    hand.click(&mut h, mid);
    let corner = [whole.x + whole.w, whole.y + whole.h];
    hand.drag(
        &mut h,
        corner,
        [whole.x + whole.w * 0.75, whole.y + whole.h * 0.75],
    );
    let crop = img(&h, l.whole).crop.expect("o recorte pelo gesto");
    assert!(
        (crop.w - 480.0).abs() < 2.0 && (crop.h - 300.0).abs() < 2.0,
        "{crop:?}"
    );
}

/// O rato real sobre o quadro (cliques a 50 ms: um par é um duplo-clique); `mods` = as teclas em
/// baixo (W17: o `Shift` do recorte).
pub(crate) struct Mouse {
    ts: ph2d_text::TextSystem,
    clock: u128,
    pub(crate) mods: ph2d_host::Modifiers,
}

impl Mouse {
    pub(crate) fn new() -> Self {
        Self {
            ts: ph2d_text::TextSystem::without_system_fonts(),
            clock: 1_000_000_000_000,
            mods: ph2d_host::Modifiers::default(),
        }
    }

    fn at(&mut self, h: &mut HeroScreen, kind: ph2d_host::PointerKind, world: [f64; 2]) {
        use ph2d_editor_core::screens::hero::board_view::{Input, pointer};
        Hand::new().paint(h);
        let b = h.documents.active_board().unwrap();
        let c = h.last_canvas;
        let area = [c.x, c.y, c.w, c.h].map(f64::from);
        let p = b.camera.to_screen(area, world);
        self.clock += 50_000_000;
        let input = Input {
            text: &mut self.ts,
            mods: self.mods,
            now_ns: self.clock,
        };
        // Os movimentos chegam pela porta deles (a da janela).
        if kind == ph2d_host::PointerKind::Move {
            ph2d_editor_core::screens::hero::board_view::pointer_move(
                h,
                input,
                p[0] as f32,
                p[1] as f32,
            );
            return;
        }
        pointer(
            h,
            input,
            kind,
            ph2d_host::PointerButton::Primary,
            p[0] as f32,
            p[1] as f32,
            true,
        );
    }

    pub(crate) fn click(&mut self, h: &mut HeroScreen, world: [f64; 2]) {
        use ph2d_host::PointerKind::{Down, Up};
        self.at(h, Down, world);
        self.at(h, Up, world);
    }

    pub(crate) fn drag(&mut self, h: &mut HeroScreen, from: [f64; 2], to: [f64; 2]) {
        use ph2d_host::PointerKind::{Down, Move, Up};
        self.at(h, Down, from);
        let mid = [(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0];
        self.at(h, Move, mid);
        self.at(h, Move, to);
        self.at(h, Up, to);
    }
}
