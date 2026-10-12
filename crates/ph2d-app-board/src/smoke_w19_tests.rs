//! ⭐ A cena 20 ARMA o que as lições dizem: o play activa o vídeo e os NOSSOS controles aparecem por
//! baixo dele (nunca à frente); um zoom que deixa metade fora da vista não o tira de vivo; `F` põe-no no
//! nosso ecrã inteiro com a proporção dele e `Esc` sai; na apresentação o vídeo do diapositivo é
//! clicável. (O canal que abre fora corre no navegador real: `ph2d-board-video`, testes `#[ignore]`.)

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use crate::smoke::w16::tests::Mouse;
use ph2d_board_render::PlayerStatus;
use ph2d_editor_core::screens::hero::board_keys::{BoardKey, key};
use ph2d_editor_core::screens::hero::board_nav::team::show;
use ph2d_editor_core::screens::hero::board_video_bar::{self, Ctl};
use ph2d_vector::Point;

/// O centro de ecrã do que o pintor registou com `id`.
fn find(h: &HeroScreen, id: ph2d_a11y::NodeId) -> Option<(f32, f32)> {
    let vp = h.last_viewport;
    let (mut n, mut sx, mut sy) = (0.0, 0.0, 0.0);
    let mut y = vp.y;
    while y < vp.y + vp.h {
        let mut x = vp.x;
        while x < vp.x + vp.w {
            if h.hit_index.hit(x, y) == Some(id) {
                (n, sx, sy) = (n + 1.0, sx + x, sy + y);
            }
            x += 3.0;
        }
        y += 3.0;
    }
    (n > 0.0).then(|| (sx / n, sy / n))
}

/// O rectângulo de ecrã do vídeo, pela colocação que o desenho registou.
fn rect(h: &mut HeroScreen, l: &Lessons) -> [f64; 4] {
    let el = h
        .documents
        .active_board()
        .unwrap()
        .doc
        .get(l.video)
        .unwrap()
        .clone();
    let pl = h.documents.videos_mut().on_screen[&l.video];
    let (a, b) = (
        pl.to_screen * Point::new(0.0, 0.0),
        pl.to_screen * Point::new(el.w, el.h),
    );
    [a.x, a.y, b.x - a.x, b.y - a.y]
}

fn press(h: &mut HeroScreen, k: BoardKey) {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let none = ph2d_host::Modifiers::default();
    for down in [true, false] {
        let _ = key(h, &mut ts, k, none, down, None, None);
    }
    Hand::new().paint(h);
}

#[test]
fn the_w19_scene_does_what_its_lessons_say() {
    let mut h = hero("Ana");
    let l = scene_w19(&mut h);
    // A 1.ª pintura na janela da FOTO (1280×800, os dois painéis abertos): é ela que enquadra a cena.
    let photo = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1280.0, 800.0);
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    ph2d_editor_core::screens::hero::paint_hero_screen(
        &mut h,
        photo,
        &mut ph2d_vector::VectorScene::new(),
        &mut ts,
    );
    let el = h
        .documents
        .active_board()
        .unwrap()
        .doc
        .get(l.video)
        .unwrap()
        .clone();
    // Cada lição e o vídeo inteiros dentro da área com que a cena abre, nessa janela. ⚠️ A área do teste não
    // tem os painéis laterais (1268 px); na janela real eles TAPAM os lados do quadro e o enquadramento não os
    // desconta — o corte da 1.ª foto de 12/10 (aberto no handoff W19; a cena 19 tem o mesmo).
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
    // Lição 1: o play activa-o; com o tocador a responder, a barra fica POR BAIXO.
    Mouse::new().click(&mut h, [el.x + el.w / 2.0, el.y + el.h / 2.0]);
    h.documents.videos_mut().status = Some(PlayerStatus {
        time: 12.0,
        duration: 596.0,
        playing: true,
        volume: 100,
        muted: false,
    });
    Hand::new().paint(&mut h);
    assert_eq!(h.documents.videos_mut().active, Some(l.video));
    assert!(h.documents.videos_mut().on_screen[&l.video].live);
    let [_, vy, _, vh] = rect(&mut h, &l);
    for c in [Ctl::Play, Ctl::Seek, Ctl::Fullscreen, Ctl::Outside] {
        let (_, y) = find(&h, board_video_bar::node(c)).unwrap_or_else(|| panic!("{c:?} não está"));
        assert!(f64::from(y) > vy + vh, "{c:?} à frente do vídeo");
    }
    // Lição 2: o zoom (2×, o canto do vídeo no meio da vista) deixa 3/4 fora — continua vivo.
    {
        let b = h.documents.active_board_mut().unwrap();
        b.camera.zoom *= 2.0;
        (b.camera.center_x, b.camera.center_y) = (el.x, el.y);
    }
    Hand::new().paint(&mut h);
    let pl = h.documents.videos_mut().on_screen[&l.video];
    assert!(pl.live, "o zoom tirou-o de vivo: {:?}", pl.screen);
    assert!(
        pl.screen.visible[0] < pl.screen.size[0],
        "a régua: metade fora da vista"
    );
    // Lição 4: `F` — o nosso ecrã inteiro, com a proporção do vídeo; `Esc` sai.
    press(&mut h, BoardKey::Char('f'));
    assert!(h.documents.videos_mut().fullscreen);
    let [_, _, w, hh] = rect(&mut h, &l);
    assert!((w / hh - el.w / el.h).abs() < 1e-3, "esticado: {w}×{hh}");
    press(&mut h, BoardKey::Escape);
    assert!(!h.documents.videos_mut().fullscreen);
    // Lição 5: na apresentação o frame é um diapositivo e o vídeo dele é clicável.
    show::start(&mut h);
    Hand::new().paint(&mut h);
    assert!(show::showing(&h));
    assert!(
        find(&h, board_video_bar::show_node(l.video)).is_some(),
        "o vídeo do diapositivo é clicável na apresentação"
    );
    assert_eq!(l.lessons.len(), 5);
    let _ = (l.board, l.frame);
}
