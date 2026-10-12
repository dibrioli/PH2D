//! ⭐ A cena 15 ARMA o que as seis lições dizem, pelo caminho real: a imagem que se larga num ponto
//! nasce lá (a porta da shell), a barra da imagem mostra os cantos, colar uma imagem da área de
//! transferência põe-na no meio da vista, copiar e colar noutro quadro leva os bytes, exportar leva
//! as imagens, e a seta da nota acaba NA fotografia (não no centro dela).

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use ph2d_board_route::RouteCache;
use ph2d_editor_core::ids::DOC_TAB_NEW;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::board_bar::{Item, node};
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_host::Modifiers;

const CTRL: Modifiers = Modifiers {
    shift: false,
    ctrl: true,
    alt: false,
    meta: false,
};

fn images(h: &HeroScreen) -> Vec<Element> {
    h.documents
        .active_board()
        .map(|b| {
            b.doc
                .live()
                .filter(|e| e.image().is_some())
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn on_screen(h: &HeroScreen, id: ph2d_editor_core::NodeId) -> bool {
    let vp = h.last_viewport;
    let mut y = vp.y;
    while y < vp.y + vp.h {
        let mut x = vp.x;
        while x < vp.x + vp.w {
            if h.hit_index.hit(x, y) == Some(id) {
                return true;
            }
            x += 3.0;
        }
        y += 3.0;
    }
    false
}

fn key(h: &mut HeroScreen, k: BoardKey) {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    board_keys::key(h, &mut ts, k, CTRL, true, None, None);
    board_keys::key(h, &mut ts, k, CTRL, false, None, None);
}

#[test]
fn the_images_scene_does_what_its_lessons_say() {
    crate::images::install();
    let mut h = hero("Ana");
    let l = scene_images(&mut h);
    let mut hand = Hand::new();
    hand.paint(&mut h);
    let b = h.documents.active_board().unwrap();
    assert_eq!(b.id, l.board);
    let imgs = images(&h);
    assert_eq!(imgs.len(), 4, "a fotografia, o sol, a redonda e o ícone");
    for e in &imgs {
        assert!(
            b.file(e.image().unwrap().file).is_some(),
            "uma imagem sem bytes"
        );
    }
    assert_eq!(b.files.len(), 2, "a fotografia uma vez para as três");
    let sun = b.doc.get(l.sun).unwrap().image().unwrap();
    assert!(sun.crop.is_some() && sun.flip == [true, false]);
    let icon = b.doc.get(l.icon).unwrap().image().unwrap();
    let px = crate::images::decode(&b.file(icon.file).unwrap().bytes.0, None).unwrap();
    assert_eq!(px.rgba[3], 0, "o canto do ícone é transparente");
    let round = b.doc.get(l.round).unwrap().image().unwrap();
    assert!(round.style.round && round.style.opacity == 75);
    for (i, id) in l.lessons.iter().enumerate() {
        let t = b
            .doc
            .get(*id)
            .unwrap()
            .shape()
            .unwrap()
            .text
            .as_str()
            .to_owned();
        assert_eq!(t, ph2d_i18n::tr(&format!("board.smoke.images.l{}", i + 1)));
    }

    // 2 — a seta acaba na BORDA da fotografia (presa a ela), não no centro.
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let end = *routes
        .get(l.arrow)
        .expect("a seta tem rota")
        .stations
        .last()
        .unwrap();
    let p = b.doc.get(l.photo).unwrap();
    let c = p.center();
    assert!(
        (end[0] - c[0]).hypot(end[1] - c[1]) > 100.0,
        "a seta vai ao centro: {end:?}"
    );
    assert!(
        end[0] >= p.x - 20.0
            && end[0] <= p.x + p.w + 20.0
            && end[1] >= p.y - 20.0
            && end[1] <= p.y + p.h + 20.0
    );

    // 6 — exportar leva as quatro imagens (o SVG: os bytes, nenhum píxel — W16).
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let page = ph2d_board_export::board_page(b, &routes).unwrap();
    let set: std::collections::BTreeSet<ElementId> = Default::default();
    let (mut s, size) = ph2d_board_export::paint_page(
        b,
        &page,
        1.0,
        None,
        ph2d_board_export::ExportTarget::Svg,
        &set,
        &mut ts,
    );
    let ops = s.take_record();
    let n = ops
        .iter()
        .filter(|o| matches!(o, ph2d_vector::DrawOp::Embed(_)))
        .count();
    assert_eq!(n, 4);
    assert!(
        !ops.iter()
            .any(|o| matches!(o, ph2d_vector::DrawOp::Image(_)))
    );
    let svg = ph2d_board_export::svg::write(&ops, size);
    assert_eq!(
        svg.matches("<image id=").count(),
        2,
        "cada ficheiro uma vez"
    );

    // 1 — largar um ficheiro num ponto do ecrã: nasce centrado lá, seleccionado.
    let png = crate::images::encode_png(&photo()).unwrap();
    let path = std::env::temp_dir().join(format!("ph2d_w14_{}.png", std::process::id()));
    std::fs::write(&path, &png).unwrap();
    let c = h.last_canvas;
    let view = [c.x, c.y, c.w, c.h].map(f64::from);
    let at = [view[0] + view[2] * 0.25, view[1] + view[3] * 0.75];
    assert!(
        crate::images::drop_files(&mut h, &[path.clone(), path.with_extension("txt")], at)
            .is_some(),
        "o que não é imagem avisa"
    );
    let _ = std::fs::remove_file(&path);
    let dropped = images(&h)
        .into_iter()
        .find(|e| !imgs.iter().any(|o| o.id == e.id))
        .unwrap();
    let b = h.documents.active_board().unwrap();
    let s = b.camera.to_screen(view, dropped.center());
    assert!(
        (s[0] - at[0]).abs() < 1e-6 && (s[1] - at[1]).abs() < 1e-6,
        "{s:?} ≠ {at:?}"
    );

    // 3 — a imagem largada fica seleccionada: a barra dela mostra os cantos e a opacidade.
    hand.paint(&mut h);
    assert!(
        on_screen(&h, node(Item::Round(true))),
        "a barra da imagem não tem os cantos"
    );
    assert!(on_screen(&h, node(Item::Opacity(0))));

    // 4 — colar uma imagem da área de transferência: nasce no meio da vista.
    let px = photo();
    assert!(crate::images::paste(&mut h, None, px.size, px.rgba));
    let b = h.documents.active_board().unwrap();
    let pasted = images(&h).into_iter().max_by_key(|e| e.id).unwrap();
    let c = pasted.center();
    assert!((c[0] - b.camera.center_x).abs() < 1e-6 && (c[1] - b.camera.center_y).abs() < 1e-6);

    // 5 — Ctrl+C na imagem colada, outro quadro, Ctrl+V: a cópia leva os bytes.
    key(&mut h, BoardKey::Char('c'));
    h.apply_event(WidgetEvent::Click(DOC_TAB_NEW));
    hand.paint(&mut h);
    assert!(images(&h).is_empty());
    key(&mut h, BoardKey::Char('v'));
    let copy = images(&h);
    assert_eq!(copy.len(), 1, "a colagem noutro quadro");
    let b = h.documents.active_board().unwrap();
    assert!(
        b.file(copy[0].image().unwrap().file).is_some(),
        "colou sem os bytes"
    );
}
