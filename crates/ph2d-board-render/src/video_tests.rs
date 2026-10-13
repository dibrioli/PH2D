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

/// ⭐⭐ W21 (o dono, 13/10: «se alguma forma sobrepõe parcialmente o vídeo, o vídeo não toca. Corrija») —
/// o vídeo desenha-se por cima do que o sobrepõe (nada à frente do tocador: as regras do YouTube; o oráculo
/// esconde atrás do vídeo o que se desenha por cima dele, `sonda_w21.mjs` F_*). Uma nota mais acima no z
/// pinta-se ANTES do quadro do tocador e ele continua vivo, sem play; só outro VÍDEO por cima o tapa.
#[test]
fn a_video_draws_over_what_overlaps_it_and_only_another_video_covers_it() {
    let yellow = Brush::Solid(crate::doc_color(Rgba([255, 230, 0, 255])));
    for note in [false, true] {
        let b = board("AAAAAAAAAA2", note);
        let mut c = RenderCache::default();
        c.videos.activate(vid_of(&b));
        c.videos.status = Some(PlayerStatus::default());
        c.videos.frame = Some((flat([1, 2, 3, 255], 560, 315), [560, 315]));
        let o = ops(&b, &mut c);
        let p = c.videos.on_screen[&vid_of(&b)];
        assert!(p.live, "nota por cima = {note}: o tocador parou");
        assert!(!p.screen.covered);
        assert!(!has_play(&o));
        let player = o
            .iter()
            .position(|d| matches!(d, DrawOp::Image(i) if i.image.rgba()[..4] == [1, 2, 3, 255]))
            .expect("o quadro do tocador");
        if note {
            let n = o
                .iter()
                .position(|d| matches!(d, DrawOp::Fill { brush, .. } if *brush == yellow))
                .expect("a nota");
            assert!(
                n < player,
                "a nota pintou-se por cima do tocador ({n} > {player})"
            );
        }
    }
    // Outro vídeo mais acima no z, a sobrepô-lo: esse sim tapa-o.
    let mut b = board("AAAAAAAAAA2", false);
    let e = Embed {
        link: "https://youtu.be/AAAAAAAAAA3".into(),
        video: video("AAAAAAAAAA3"),
        style: Style::new(None, None, Rgba([0, 0, 0, 255])),
    };
    let over = Element::new_embed(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        e,
        [400.0, 200.0, 560.0, 315.0],
    );
    let _ = BoardOp::Put(over).apply(&mut b.doc);
    let first = b.doc.live_in_z_order()[0].id;
    let mut c = RenderCache::default();
    c.videos.activate(first);
    let _ = ops(&b, &mut c);
    let p = c.videos.on_screen[&first];
    assert!(
        p.screen.covered && !p.live,
        "outro vídeo por cima e ainda vivo"
    );
}

/// W21 — com o tocador a ARRANCAR (vai tocar sozinho), o vídeo mostra «a carregar» no lugar do play; quando
/// o tocador começou, nada.
#[test]
fn a_starting_player_shows_it_is_loading() {
    let scrim = Brush::Solid(crate::token(ColorToken::BgScrim, Theme::Forge));
    let loading = |o: &[DrawOp]| {
        o.iter()
            .any(|d| matches!(d, DrawOp::Fill { brush, .. } if *brush == scrim))
    };
    let b = board("AAAAAAAAAA4", false);
    for (status, want) in [
        (None, true),
        (
            Some(PlayerStatus {
                starting: true,
                ..PlayerStatus::default()
            }),
            true,
        ),
        (Some(PlayerStatus::default()), false),
    ] {
        let mut c = RenderCache::default();
        c.videos.activate(vid_of(&b));
        c.videos.status = status;
        let o = ops(&b, &mut c);
        assert_eq!(loading(&o), want, "{status:?}");
        assert!(!has_play(&o), "vivo: sem o play");
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

/// O retângulo de ecrã `[x, y, w, h]` da imagem do quadro do tocador.
fn frame_rect(o: &[DrawOp], color: [u8; 4]) -> [f64; 4] {
    let i = o
        .iter()
        .find_map(|d| match d {
            DrawOp::Image(i) if i.image.rgba()[..4] == color => Some(i),
            _ => None,
        })
        .expect("o quadro do tocador não se desenhou");
    let (w, h) = (f64::from(i.image.width()), f64::from(i.image.height()));
    let a = i.transform * ph2d_vector::Point::new(0.0, 0.0);
    let b = i.transform * ph2d_vector::Point::new(w, h);
    [a.x, a.y, b.x - a.x, b.y - a.y]
}

/// ⭐⭐ W19 (smoke do dono, defeito 1): um quadro do tocador com OUTRA proporção que a caixa (o ecrã
/// inteiro do tocador mandava 420×315 para uma caixa 560×315) desenha-se INTEIRO e com faixas, como o
/// tocador faz — nunca esticado.
#[test]
fn a_frame_of_another_shape_draws_with_bands_never_stretched() {
    let b = board("AAAAAAAAAA4", false);
    let mut c = RenderCache::default();
    c.videos.activate(vid_of(&b));
    c.videos.frame = Some((flat([4, 5, 6, 255], 420, 315), [420, 315]));
    let o = ops(&b, &mut c);
    let box_ = c.videos.on_screen[&vid_of(&b)].to_screen * ph2d_vector::Point::new(0.0, 0.0);
    let [x, y, w, h] = frame_rect(&o, [4, 5, 6, 255]);
    assert!(
        (w / h - 420.0 / 315.0).abs() < 1e-9,
        "esticado: {w}×{h} (a proporção do quadro é 4:3)"
    );
    assert!(
        (h - 315.0).abs() < 1e-9 && (w - 420.0).abs() < 1e-9,
        "{w}×{h}"
    );
    assert!(
        (x - box_.x - 70.0).abs() < 1e-9 && (y - box_.y).abs() < 1e-9,
        "as faixas de 70 px dos dois lados: {x},{y}"
    );
    // Da mesma forma, um quadro mais pequeno que a caixa (o tocador ainda não seguiu o zoom) ENCHE-a.
    c.videos.frame = Some((flat([4, 5, 6, 255], 320, 180), [320, 180]));
    let o = ops(&b, &mut c);
    let r = frame_rect(&o, [4, 5, 6, 255]);
    assert!(
        (r[2] - 560.0).abs() < 1e-9 && (r[3] - 315.0).abs() < 1e-9,
        "{r:?}"
    );
}

/// ⭐⭐ W20 (o dono: «a janela de vídeo deve ter as opções de decoração como um retângulo») — com borda e
/// fundo: o fundo pinta-se ANTES do quadro do tocador (por trás), a borda DEPOIS e no contorno da caixa, e
/// o tocador recua pela borda — nenhum píxel do traço (metade dele para dentro do contorno) cai no quadro
/// do tocador (as regras do YouTube); o tocador no ecrã tem o tamanho do rectângulo recuado. O oráculo
/// (`sonda_w20.mjs` E2): o iframe recua pela largura do traço.
#[test]
fn a_framed_video_keeps_its_border_and_fill_around_the_player_never_on_it() {
    let mut b = board("AAAAAAAAAA6", false);
    let id = vid_of(&b);
    let (red, yellow) = (Rgba([224, 49, 49, 255]), Rgba([255, 236, 153, 255]));
    let mut el = b.doc.get(id).unwrap().clone();
    let st = el.style_mut();
    (st.stroke, st.stroke_width, st.fill) = (Some(red), 4.0, Some(yellow));
    st.dash = ph2d_board_model::Dash::Dashed;
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    let mut c = RenderCache::default();
    c.videos.activate(id);
    c.videos.status = Some(PlayerStatus::default());
    c.videos.frame = Some((flat([7, 7, 7, 255], 560, 315), [560, 315]));
    let o = ops(&b, &mut c);
    let at = |want: &dyn Fn(&DrawOp) -> bool, what: &str| {
        o.iter()
            .position(want)
            .unwrap_or_else(|| panic!("{what} não se desenhou"))
    };
    let fill = at(
        &|d| matches!(d, DrawOp::Fill { brush: Brush::Solid(k), .. } if *k == crate::doc_color(yellow)),
        "o fundo",
    );
    let frame = at(
        &|d| matches!(d, DrawOp::Image(i) if i.image.rgba()[..4] == [7, 7, 7, 255]),
        "o quadro do tocador",
    );
    let border = at(&|d| matches!(d, DrawOp::Stroke { .. }), "a borda");
    assert!(
        fill < frame && frame < border,
        "a ordem: fundo {fill}, tocador {frame}, borda {border}"
    );
    // A caixa no ecrã: zoom 1, o meio dela no meio da vista de 1280×800.
    let (x0, y0) = (640.0 - 280.0, 400.0 - 157.5);
    let [x, y, w, h] = frame_rect(&o, [7, 7, 7, 255]);
    let eps = 1e-6;
    assert!(
        x >= x0 + 4.0 - eps
            && y >= y0 + 4.0 - eps
            && x + w <= x0 + 556.0 + eps
            && y + h <= y0 + 311.0 + eps,
        "o quadro do tocador {x},{y} {w}×{h} não recuou pela borda de 4 (a caixa em {x0},{y0})"
    );
    let p = c.videos.on_screen[&id];
    assert!(
        (p.screen.size[0] - 552.0).abs() < eps && (p.screen.size[1] - 307.0).abs() < eps,
        "o tocador é o rectângulo recuado: {:?}",
        p.screen.size
    );
    assert!(p.live, "decorado continua vivo");
    let DrawOp::Stroke { style, .. } = &o[border] else {
        unreachable!()
    };
    assert!(
        (style.width - 4.0).abs() < eps && !style.dash_pattern.is_empty(),
        "{style:?}"
    );
}

/// ⭐⭐ W19 (smoke do dono, defeito 2): com o zoom, o vídeo MEIO FORA da vista continua vivo (W20: até
/// sair todo), e a parte visível é a medida.
#[test]
fn a_video_half_out_of_view_stays_live() {
    let mut b = board("AAAAAAAAAA5", false);
    let mut c = RenderCache::default();
    let id = vid_of(&b);
    c.videos.activate(id);
    // O meio do vídeo no canto de cima à esquerda da vista de 1280×800, ao zoom 2: só um quarto à vista.
    b.camera.zoom = 2.0;
    b.camera.center_x = 280.0 + 640.0 / 2.0;
    b.camera.center_y = 157.5 + 400.0 / 2.0;
    let _ = ops(&b, &mut c);
    let p = c.videos.on_screen[&id];
    assert_eq!(p.screen.size, [1120.0, 630.0]);
    assert!(
        (p.screen.visible[0] - 560.0).abs() < 1e-6 && (p.screen.visible[1] - 315.0).abs() < 1e-6,
        "{:?}",
        p.screen.visible
    );
    assert!(
        p.live,
        "meio fora da vista com 560×315 à vista pausava o vídeo"
    );
}
