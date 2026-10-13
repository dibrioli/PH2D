//! ⭐ A cena 21 ARMA o que as lições dizem: o clique toca e o 2.º pausa; segurar e arrastar move o vídeo
//! (também a tocar); o pan com o Espaço não o pausa nem o desactiva; clicar fora deixa-o a tocar e só todo
//! fora da vista o pausa; a borda da cena fica À VOLTA do tocador (ele recua para dentro dela).

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use crate::smoke::w16::tests::Mouse;
use ph2d_board_render::{PlayerStatus, VideoCommand};
use ph2d_editor_core::screens::hero::board_keys::{BoardKey, key};

const STATUS: PlayerStatus = PlayerStatus {
    time: 12.0,
    duration: 596.0,
    playing: true,
    volume: 100,
    muted: false,
};

fn el(h: &HeroScreen, l: &Lessons) -> Element {
    h.documents
        .active_board()
        .unwrap()
        .doc
        .get(l.video)
        .unwrap()
        .clone()
}

fn live(h: &mut HeroScreen, l: &Lessons) -> bool {
    h.documents
        .videos_mut()
        .on_screen
        .get(&l.video)
        .is_some_and(|p| p.live)
}

fn space(h: &mut HeroScreen, down: bool) {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let none = ph2d_host::Modifiers::default();
    let _ = key(
        h,
        &mut ts,
        BoardKey::Space,
        none,
        down,
        down.then_some(" "),
        None,
    );
}

#[test]
fn the_w20_scene_does_what_its_lessons_say() {
    let mut h = hero("Ana");
    let l = scene_w20(&mut h);
    // A 1.ª pintura na janela da FOTO (1280×800): é ela que enquadra a cena.
    let photo = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1280.0, 800.0);
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    ph2d_editor_core::screens::hero::paint_hero_screen(
        &mut h,
        photo,
        &mut ph2d_vector::VectorScene::new(),
        &mut ts,
    );
    {
        let b = h.documents.active_board().unwrap();
        let c = h.last_canvas;
        let area = [c.x, c.y, c.w, c.h].map(f64::from);
        for id in l.lessons.iter().chain([&l.video]) {
            let e = b.doc.get(*id).unwrap();
            let p0 = b.camera.to_screen(area, [e.x, e.y]);
            let p1 = b.camera.to_screen(area, [e.x + e.w, e.y + e.h]);
            let inside = p0[0] >= area[0]
                && p0[1] >= area[1]
                && p1[0] <= area[0] + area[2]
                && p1[1] <= area[1] + area[3];
            assert!(inside, "{id:?} cortado: {p0:?}..{p1:?} fora de {area:?}");
        }
    }
    Hand::new().paint(&mut h);
    let v = el(&h, &l);
    let mid = |v: &Element| [v.x + v.w / 2.0, v.y + v.h / 2.0];
    // Lições 5 e 6: a borda da cena está À VOLTA do tocador — ele recua por ela.
    let z = h.documents.active_board().unwrap().camera.zoom;
    let size = h.documents.videos_mut().on_screen[&l.video].screen.size;
    let st = v.style().clone();
    assert!(
        st.stroke.is_some() && st.fill.is_some(),
        "a cena abre decorada"
    );
    let want = [
        (v.w - 2.0 * st.stroke_width) * z,
        (v.h - 2.0 * st.stroke_width) * z,
    ];
    assert!(
        (size[0] - want[0]).abs() < 1e-6 && (size[1] - want[1]).abs() < 1e-6,
        "o tocador {size:?} não recuou pela borda ({want:?})"
    );
    // Lição 1: o clique toca (activa); com o tocador a responder, o 2.º clique pausa.
    let mut m = Mouse::new();
    m.click(&mut h, mid(&v));
    h.documents.videos_mut().status = Some(STATUS);
    Hand::new().paint(&mut h);
    assert_eq!(h.documents.videos_mut().active, Some(l.video));
    assert!(live(&mut h, &l));
    h.documents.videos_mut().inbox.clear();
    m.click(&mut h, mid(&v));
    let inbox = std::mem::take(&mut h.documents.videos_mut().inbox);
    assert_eq!(
        inbox.len(),
        2,
        "o 2.º clique vai ao tocador (a pausa dele): {inbox:?}"
    );
    // Lição 2: segurar e arrastar move-o, a tocar — e nada vai ao tocador.
    let to = [mid(&v)[0] + 120.0, mid(&v)[1]];
    m.drag(&mut h, mid(&v), to);
    let moved = el(&h, &l);
    assert!(
        (moved.x - v.x - 120.0).abs() < 1e-6,
        "não moveu: {} -> {}",
        v.x,
        moved.x
    );
    assert!(h.documents.videos_mut().inbox.is_empty());
    assert_eq!(h.documents.videos_mut().active, Some(l.video));
    // Lição 3: o pan com o Espaço — a vista anda, o vídeo continua activo e vivo, nenhuma ordem.
    let cam = h.documents.active_board().unwrap().camera;
    space(&mut h, true);
    m.drag(&mut h, mid(&moved), [mid(&moved)[0] - 60.0, mid(&moved)[1]]);
    space(&mut h, false);
    Hand::new().paint(&mut h);
    assert_ne!(
        h.documents.active_board().unwrap().camera,
        cam,
        "a vista não andou"
    );
    assert_eq!(h.documents.videos_mut().active, Some(l.video));
    assert!(live(&mut h, &l));
    assert_eq!(
        std::mem::take(&mut h.documents.videos_mut().commands),
        Vec::<VideoCommand>::new()
    );
    assert_eq!(el(&h, &l).x, moved.x, "o pan moveu o vídeo");
    // Lição 4: clicar fora deixa-o a tocar; todo fora da vista pausa; de volta, toca.
    let away = [moved.x - 150.0, moved.y + moved.h + 40.0];
    m.click(&mut h, away);
    Hand::new().paint(&mut h);
    assert_eq!(h.documents.videos_mut().active, None);
    assert!(live(&mut h, &l), "clicar fora pausou");
    let back = h.documents.active_board().unwrap().camera;
    h.documents.active_board_mut().unwrap().camera.center_x += 5000.0;
    Hand::new().paint(&mut h);
    assert!(!live(&mut h, &l), "todo fora da vista e ainda vivo");
    h.documents.active_board_mut().unwrap().camera = back;
    Hand::new().paint(&mut h);
    assert!(live(&mut h, &l), "de volta e não voltou");
    let _ = l.board;
}
