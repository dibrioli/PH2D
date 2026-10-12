//! W18 — o vídeo pelo despacho REAL: colar um link do YouTube (Ctrl+V) faz nascer um vídeo; o clique no
//! play activa-o; activo e vivo, o rato e as teclas vão para o tocador e não para o quadro; clicar fora
//! desactiva; o botão «Video» pede o link à área de transferência; outro link continua texto.

use super::*;
use crate::screens::hero::board_video;
use ph2d_board_model::ElementId;
use ph2d_board_render::{PlayerInput, PlayerKey};

const LINK: &str = "https://www.youtube.com/watch?v=aqz-KE-bpKQ";

fn paste(t: &mut T, text: &str) -> bool {
    let out = board_keys::key(&mut t.hero, &mut t.ts, BoardKey::Char('v'), CTRL, true, None, Some(text.into()));
    board_keys::key(&mut t.hero, &mut t.ts, BoardKey::Char('v'), CTRL, false, None, None);
    t.paint();
    out.consumed
}

fn video(t: &T) -> Option<Element> {
    t.elements().into_iter().find(|e| e.embed().is_some())
}

/// O ponto de ecrã do local `(fx, fy)` (fracções da caixa) do vídeo, pelo que o desenho registou.
fn on_video(t: &mut T, id: ElementId, fx: f64, fy: f64) -> (f32, f32) {
    let el = video(t).unwrap();
    let p = t.hero.documents.videos_mut().on_screen[&id].to_screen
        * ph2d_vector::Point::new(el.w * fx, el.h * fy);
    (p.x as f32, p.y as f32)
}

fn click(t: &mut T, (x, y): (f32, f32)) {
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    let p = PointerButton::Primary;
    board_view::pointer(h, inp(ts, c), PointerKind::Down, p, x, y, true);
    board_view::pointer(h, inp(ts, c), PointerKind::Up, p, x, y, true);
    t.paint();
}

/// ⭐⭐ O caminho inteiro do vídeo no quadro.
#[test]
fn a_pasted_link_becomes_a_video_that_plays_on_its_play_and_takes_the_mouse_and_keys() {
    let mut t = T::new();
    assert!(paste(&mut t, LINK), "o link colado é do vídeo");
    let v = video(&t).expect("nasceu um vídeo");
    assert_eq!(v.embed().unwrap().link, LINK);
    assert_eq!([v.w, v.h], [560.0, 315.0], "o tamanho do oráculo");
    let id = v.id;
    assert!(t.hero.documents.videos_mut().on_screen.contains_key(&id), "o desenho registou-o");
    // Activar: o clique no play.
    let play = on_video(&mut t, id, 0.5, 0.5);
    click(&mut t, play);
    assert_eq!(t.hero.documents.videos_mut().active, Some(id));
    let pl = t.hero.documents.videos_mut().on_screen[&id];
    assert!(pl.live, "inteiro à vista, descoberto, ≥ 200 px: vivo ({:?})", pl.screen);
    // Activo: o clique dentro vai para o tocador (o vídeo não se mexe).
    t.hero.documents.videos_mut().inbox.clear();
    let inside = on_video(&mut t, id, 0.25, 0.75);
    click(&mut t, inside);
    let inbox = std::mem::take(&mut t.hero.documents.videos_mut().inbox);
    assert!(matches!(inbox.first(), Some(PlayerInput::Down { button: 0, .. })), "{inbox:?}");
    assert!(inbox.iter().any(|i| matches!(i, PlayerInput::Up { .. })), "{inbox:?}");
    let [x, y] = match inbox[0] {
        PlayerInput::Down { x, y, .. } => [x, y],
        _ => unreachable!(),
    };
    assert!((x - pl.screen.size[0] * 0.25).abs() < 1.0 && (y - pl.screen.size[1] * 0.75).abs() < 1.0, "{x},{y}");
    assert_eq!([video(&t).unwrap().x, video(&t).unwrap().y], [v.x, v.y], "o quadro não arrastou");
    // As teclas: o espaço é do tocador.
    assert!(t.key(BoardKey::Space, Modifiers::default(), None));
    let keys = std::mem::take(&mut t.hero.documents.videos_mut().inbox);
    assert!(keys.contains(&PlayerInput::Key { down: true, key: PlayerKey::Space }), "{keys:?}");
    // Fora: desactiva.
    let out = t.at(0.02, 0.98);
    click(&mut t, out);
    assert_eq!(t.hero.documents.videos_mut().active, None);
}

/// Um link que NÃO é um vídeo do YouTube continua o colar de texto (nenhum vídeo nasce).
#[test]
fn another_link_stays_text() {
    let mut t = T::new();
    let _ = paste(&mut t, "https://example.com/page");
    assert!(video(&t).is_none());
}

/// O botão «Video» está na barra curta, e o clique pede o link ao app.
#[test]
fn the_video_button_asks_the_app_for_the_clipboard_link() {
    let mut t = T::new();
    assert!(!t.hero.documents.take_video_ask());
    t.click_bar(Item::Video);
    assert!(t.hero.documents.take_video_ask());
    // E o app, com um link, põe o vídeo pelo mesmo caminho do colar.
    assert!(board_video::insert_link(&mut t.hero, LINK, None));
    t.paint();
    assert!(video(&t).is_some());
}
