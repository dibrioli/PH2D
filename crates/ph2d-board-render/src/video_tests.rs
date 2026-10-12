//! W18 — o vídeo no desenho: a miniatura pedida e desenhada, o símbolo de play, o tocador vivo só onde as
//! regras do YouTube deixam, e o ponto do ecrã em píxeis do tocador.

use std::sync::Arc;

use ph2d_board_model::{
    Board, BoardOp, BoardSet, Element, Embed, Rgba, Shape, ShapeType, Style, Video,
};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Brush, DrawOp, StableImage, VectorScene};

use super::*;
use crate::RenderCache;

fn video(id: &str) -> Video {
    Video::YouTube {
        id: id.into(),
        start: 0,
        vertical: false,
    }
}

/// Um vídeo 560×315 com a câmara a mostrá-lo inteiro; `note` = uma nota POR CIMA dele.
fn board(id: &str, note: bool) -> Board {
    let mut set = BoardSet::default();
    let bid = set.create("v".into());
    let b = set.get_mut(bid).unwrap();
    let e = Embed {
        link: format!("https://youtu.be/{id}"),
        video: video(id),
        style: Style::new(None, None, Rgba([0, 0, 0, 255])),
    };
    let el = Element::new_embed(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        e,
        [0.0, 0.0, 560.0, 315.0],
    );
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    if note {
        let s = Shape {
            kind: ShapeType::Sticky,
            style: Style::new(Some(Rgba([255, 230, 0, 255])), None, Rgba([0, 0, 0, 255])),
            text: ph2d_board_model::RichText::default(),
        };
        let n = Element::new_shape(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            s,
            [500.0, 280.0, 100.0, 100.0],
        );
        let _ = BoardOp::Put(n).apply(&mut b.doc);
    }
    let mut out = set.boards()[0].clone();
    out.camera.center_x = 280.0;
    out.camera.center_y = 157.5;
    out
}

fn ops(b: &Board, c: &mut RenderCache) -> Vec<DrawOp> {
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let mut scene = VectorScene::new();
    scene.record();
    let mut ts = TextSystem::without_system_fonts();
    crate::paint(
        b,
        [0.0, 0.0, 1280.0, 800.0],
        &mut scene,
        Theme::Forge,
        &mut ts,
        c,
        &routes,
        None,
    );
    scene.take_record()
}

fn has_play(o: &[DrawOp]) -> bool {
    let red = crate::token(ColorToken::Danger, Theme::Forge);
    o.iter()
        .any(|d| matches!(d, DrawOp::Fill { brush: Brush::Solid(k), .. } if *k == red))
}

fn image_colors(o: &[DrawOp]) -> Vec<[u8; 4]> {
    o.iter()
        .filter_map(|d| match d {
            DrawOp::Image(i) => Some(i.image.rgba()[..4].try_into().unwrap()),
            _ => None,
        })
        .collect()
}

fn flat(c: [u8; 4], w: u32, h: u32) -> StableImage {
    StableImage::from_rgba(Arc::new(c.repeat((w * h) as usize)), w, h).unwrap()
}

fn vid_of(b: &Board) -> ph2d_board_model::ElementId {
    b.doc.live().find(|e| e.embed().is_some()).unwrap().id
}

/// ⭐ Sem miniatura: o lugar vazio com o play, e a miniatura PEDIDA ao app; com ela, desenha-se.
#[test]
fn a_video_asks_for_its_thumbnail_and_then_draws_it() {
    let b = board("AAAAAAAAAA1", false);
    let mut c = RenderCache::default();
    let first = ops(&b, &mut c);
    assert!(has_play(&first));
    assert!(image_colors(&first).is_empty());
    assert!(c.videos.wanted.contains("youtube:AAAAAAAAAA1"));
    set_thumbnail(
        "youtube:AAAAAAAAAA1".into(),
        Thumb::Ready(flat([9, 8, 7, 255], 4, 3), [4, 3]),
    );
    let then = ops(&b, &mut c);
    assert_eq!(image_colors(&then), vec![[9, 8, 7, 255]]);
    assert!(has_play(&then), "parado: o play por cima da miniatura");
}

/// ⭐⭐ Activo e descoberto: o quadro do tocador e NENHUM play; com uma nota por cima (as regras do
/// YouTube), o tocador não está vivo — o último quadro fica, com o play.
#[test]
fn the_player_is_live_only_when_uncovered() {
    for (note, live) in [(false, true), (true, false)] {
        let b = board("AAAAAAAAAA2", note);
        let mut c = RenderCache::default();
        c.videos.active = Some(vid_of(&b));
        c.videos.frame = Some((flat([1, 2, 3, 255], 560, 315), [560, 315]));
        let o = ops(&b, &mut c);
        let p = c.videos.on_screen[&vid_of(&b)];
        assert_eq!(p.live, live, "nota por cima = {note}");
        assert_eq!(p.screen.covered, note);
        assert!(image_colors(&o).contains(&[1, 2, 3, 255]));
        assert_eq!(has_play(&o), !live);
    }
}

/// O ponto de ecrã em píxeis do tocador, e o play ao meio.
#[test]
fn a_screen_point_maps_into_the_player() {
    let b = board("AAAAAAAAAA3", false);
    let mut c = RenderCache::default();
    let _ = ops(&b, &mut c);
    let id = vid_of(&b);
    let el = b.doc.get(id).unwrap();
    let p = c.videos.on_screen[&id];
    let o = p.to_screen * ph2d_vector::Point::new(0.0, 0.0);
    let got = c
        .videos
        .player_point(id, el, [o.x + 56.0, o.y + 31.5])
        .unwrap();
    assert!(
        (got[0] - 56.0).abs() < 1e-6 && (got[1] - 31.5).abs() < 1e-6,
        "{got:?}"
    );
    assert!(c.videos.player_point(id, el, [o.x - 5.0, o.y]).is_none());
    let mid = p.to_screen * ph2d_vector::Point::new(280.0, 157.5);
    assert!(c.videos.on_play(id, el, [mid.x, mid.y]));
    assert!(!c.videos.on_play(id, el, [o.x + 5.0, o.y + 5.0]));
}
